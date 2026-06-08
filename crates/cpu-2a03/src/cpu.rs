// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::mem::transmute;
use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::instruction::Instruction;
use crate::cpu::table::InstructionTable;

mod addressing;
mod instruction;
mod operation;
mod status;
mod table;

pub use status::CpuStatus;

/// Tracks which stage of the CPU pipeline is active.
enum Phase {
    /// At an instruction boundary: the next tick will fetch the opcode.
    Fetch,

    /// An instruction is in flight; the stored handler drives one T-state per [`Cpu2A03::tick`].
    Execute { instruction: fn() },

    /// The 7-cycle hardware reset sequence is in progress.
    Reset,

    /// The CPU has jammed on a KIL instruction; only a hardware RESET can escape.
    ///
    /// After the opcode fetch the CPU reads from `address` (= PC+1), then cycles through
    /// `$FFFF`/`$FFFE` reads indefinitely. `t` is repurposed as a halt-cycle counter.
    Halted { address: u16 },
}

/// The 2A03 NES CPU core.
/// Driven one clock cycle at a time via [`Cpu2A03::tick`].
pub struct Cpu2A03 {
    /// Accumulator (A).
    ///
    /// The main register for arithmetic and logic operations.
    /// Unlike the X and Y registers, it has a direct connection to the Arithmetic and Logic Unit (ALU).
    pub a: u8,

    /// X index register.
    pub x: u8,

    /// Y index register.
    pub y: u8,

    /// Program counter (PC).
    ///
    /// This register points the address from which the next instruction
    /// byte (opcode or parameter) will be fetched.
    pub pc: u16,

    /// Stack pointer (SP).
    ///
    /// The NMOS 65xx processors have 256 bytes of stack memory, ranging from `$0100` to `$01FF`.
    /// The S register is a 8-bit offset to the stack page.
    pub sp: u8,

    /// Processor (P) status register.
    pub p: CpuStatus,

    /// The current T-state: 0 during the opcode fetch cycle, incrementing by one each subsequent
    /// clock cycle. Also used as the step counter within the reset sequence (T0–T6).
    t: u8,

    /// Current pipeline phase.
    phase: Phase,

    /// Scratch register used during indirect addressing to hold the zero-page pointer
    /// byte before it is expanded into a full 16-bit address.
    ptr: u8,

    /// High byte of the effective address (ADH). Written once the second address byte
    /// is fetched; for zero-page modes it stays `$00` for the duration of the instruction.
    adh: u8,

    /// Low byte of the effective address (ADL). Always the first byte fetched during
    /// address resolution; mutated in-place for zero-page indexed wrap-around.
    adl: u8,

    /// Single-byte data latch used to pass a value between the read and write cycles
    /// of a read-modify-write instruction.
    data: u8,

    /// Total clock cycles elapsed since construction, incremented on every [`Cpu2A03::tick`].
    pub cycles: u64,

    /// The "magic" byte OR'd into A before the AND in unstable immediate-mode opcodes (`LXA`, `XAA`).
    ///
    /// On real hardware this value is non-deterministic, it depends on analog bus capacitance,
    /// chip revision, and temperature. Common values: `0xFF` (nestest), `0xEE` (SingleStepTests /
    /// visual6502). Defaults to `0xFF`.
    pub magic: u8,
}

impl Default for Cpu2A03 {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu2A03 {
    /// Returns a CPU with registers zeroed and ready to fetch. Call [`Cpu2A03::reset`] to run the
    /// 7-cycle hardware reset sequence and load PC from the reset vector before executing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0x00,
            p: CpuStatus(0),
            t: 0,
            phase: Phase::Fetch,
            adh: 0,
            adl: 0,
            ptr: 0,
            data: 0,
            cycles: 0,
            magic: 0xFF,
        }
    }

    /// Triggers the 7-cycle hardware reset sequence.
    ///
    /// The sequence runs through [`Cpu2A03::tick`]: SP is decremented three times, the I flag is
    /// set, and PC is loaded from the `$FFFC`/`$FFFD` reset vector. Normal instruction execution
    /// resumes on the eighth tick.
    pub fn reset(&mut self) {
        self.sp = 0x00;
        self.p = CpuStatus(0);
        self.t = 0;
        self.phase = Phase::Reset;
    }

    /// Advances the CPU by one clock cycle.
    pub fn tick<B: Bus>(&mut self, bus: &mut B) {
        self.cycles += 1;

        match self.phase {
            // KIL halt loop: T0=PC+1, T1=$FFFF, T2-T3=$FFFE, T4+=$FFFF (matches visual6502).
            #[allow(clippy::wildcard_in_or_patterns)]
            Phase::Halted { address } => {
                let address = match self.t {
                    0 => address,
                    2 | 3 => 0xFFFE,
                    1 | _ => 0xFFFF,
                };

                bus.read(address);
                self.t = self.t.saturating_add(1);
            }

            // Drive the 7-cycle hardware reset sequence one step forward.
            Phase::Reset => {
                if reset(self, bus).is_ready() {
                    self.t = 0;
                    self.phase = Phase::Fetch;
                } else {
                    self.t += 1;
                }
            }

            // T0: fetch the opcode and cache its handler. T advances to 1 so the first
            // execution T-state enters at T1.
            Phase::Fetch => {
                let opcode = self.fetch(bus);

                let Some(instruction) = InstructionTable::<B>::dispatch(opcode) else {
                    tracing::error!(
                        opcode = format!("{opcode:02X}"),
                        pc = format!("{:04X}", self.pc.wrapping_sub(1)),
                        "illegal opcode; halting CPU"
                    );

                    // Store current PC (= original_pc + 1) as the first halt-loop read address.
                    self.phase = Phase::Halted { address: self.pc };

                    return;
                };

                self.t += 1;

                // SAFETY: fn pointers are pointer-sized; B is the same type recovered in Execute.
                self.phase = Phase::Execute {
                    instruction: unsafe { transmute::<Instruction<B>, fn()>(instruction) },
                };
            }

            // Drive the in-flight instruction one T-state forward; return to Fetch when done.
            Phase::Execute { instruction } => {
                // SAFETY: instruction was stored as Instruction<B> in the Fetch arm of this same tick<B>.
                let instruction: Instruction<B> = unsafe { transmute(instruction) };

                if instruction(self, bus).is_ready() {
                    self.t = 0;
                    self.phase = Phase::Fetch;
                } else {
                    self.t += 1;
                }
            }
        }
    }

    /// Returns `true` while the 7-cycle hardware reset sequence is in progress.
    #[must_use]
    pub const fn resetting(&self) -> bool {
        matches!(self.phase, Phase::Reset)
    }

    /// Returns `true` when the CPU has jammed on a KIL opcode. Cleared by [`Cpu2A03::reset`].
    #[must_use]
    pub const fn halted(&self) -> bool {
        matches!(self.phase, Phase::Halted { .. })
    }

    /// Returns the current T-state; 0 (T0) indicates the SYNC cycle where the next
    /// opcode will be fetched.
    #[must_use]
    pub const fn t(&self) -> u8 {
        self.t
    }

    /// Reads the byte at PC, and advances PC.
    fn fetch<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let value = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        value
    }

    /// Reconstructs the full 16-bit effective address from `adl` and `adh`.
    /// Used at bus read/write sites once address resolution is complete.
    #[must_use]
    #[inline(always)]
    const fn address(&self) -> u16 {
        u16::from_le_bytes([self.adl, self.adh])
    }

    /// Sets both address bytes together, used when a full 16-bit address is known at once
    /// (e.g. zero-page fetch where `adh` is always `$00`, or indirect target assembly).
    #[inline(always)]
    const fn set_address(&mut self, adl: u8, adh: u8) {
        self.adl = adl;
        self.adh = adh;
    }

    /// Returns the full 16-bit address of the current stack top: `$0100 | SP`.
    #[inline(always)]
    #[must_use]
    const fn stack_address(&self) -> u16 {
        0x0100 | self.sp as u16
    }

    /// Writes `value` to `$0100 + SP`, then decrements SP.
    #[inline]
    fn stack_push<B: Bus>(&mut self, bus: &mut B, value: u8) {
        bus.write(self.stack_address(), value);
        self.sp = self.sp.wrapping_sub(1);
    }
}

/// Drives one cycle of the 7-cycle hardware reset sequence (T0–T6).
///
/// Returns `Poll::Pending` while the sequence is in progress and `Poll::Ready(())` on T6 once
/// PC has been loaded from the reset vector and execution can resume.
fn reset<B: Bus>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
    match cpu.t {
        // T0–T1: internal pipeline cycles; the bus is read but the result is discarded.
        0 => {
            bus.read(cpu.pc);

            Poll::Pending
        }

        1 => {
            bus.read(cpu.pc.wrapping_add(1));

            Poll::Pending
        }

        // T2–T4: phantom stack accesses. On a live reset the R/W line is forced high so three
        // reads are issued at the stack address instead of writes; SP still decrements each cycle.
        2 => {
            bus.read(cpu.stack_address());

            cpu.sp = cpu.sp.wrapping_sub(1);

            Poll::Pending
        }

        3 => {
            bus.read(cpu.stack_address());

            cpu.sp = cpu.sp.wrapping_sub(1);

            Poll::Pending
        }

        4 => {
            bus.read(cpu.stack_address());

            cpu.sp = cpu.sp.wrapping_sub(1);
            cpu.p.insert(CpuStatus::I);

            Poll::Pending
        }

        // T5–T6: fetch the reset vector from $FFFC/$FFFD and load PC.
        5 => {
            cpu.adl = bus.read(0xFFFC);

            Poll::Pending
        }

        _ => {
            cpu.adh = bus.read(0xFFFD);
            cpu.pc = cpu.address();

            Poll::Ready(())
        }
    }
}

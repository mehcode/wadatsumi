// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::mem::transmute;
use std::task::Poll;

use crate::bus::{CpuBus, CpuReadWrite};
use crate::instruction::Instruction;
use crate::interrupt::{IRQ, InterruptKind, NMI, interrupt};
use crate::reset::reset;
use crate::status::CpuStatus;
use crate::table::InstructionTable;

/// Tracks which stage of the CPU pipeline is active.
#[expect(clippy::upper_case_acronyms)]
enum Phase {
    /// At an instruction boundary: the next tick will fetch the opcode (or service a
    /// pending interrupt in its place).
    Fetch,

    /// An instruction is in flight; the stored handler drives one T-state per [`Cpu2A03::tick`].
    Execute { instruction: fn() },

    /// The 7-cycle hardware reset sequence is in progress.
    Reset,

    /// A hardware IRQ sequence is in-flight.
    IRQ,

    /// A hardware NMI sequence is in-flight.
    NMI,

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
    /// clock cycle. Also used as the step counter within the reset sequence (`T0..=T6`).
    pub(crate) t: u8,

    /// Current pipeline phase.
    phase: Phase,

    /// Scratch register used during indirect addressing to hold the zero-page pointer
    /// byte before it is expanded into a full 16-bit address.
    pub(crate) ptr: u8,

    /// High byte of the effective address (ADH). Written once the second address byte
    /// is fetched; for zero-page modes it stays `$00` for the duration of the instruction.
    pub(crate) adh: u8,

    /// Low byte of the effective address (ADL). Always the first byte fetched during
    /// address resolution; mutated in-place for zero-page indexed wrap-around.
    pub(crate) adl: u8,

    /// Single-byte data latch used to pass a value between the read and write cycles
    /// of a read-modify-write instruction.
    pub(crate) data: u8,

    /// Total clock cycles elapsed since construction, incremented on every [`Cpu2A03::tick`].
    pub cycles: u64,

    /// The "magic" byte OR'd into A before the AND in unstable immediate-mode opcodes (`LXA`, `XAA`).
    ///
    /// On real hardware this value is non-deterministic, it depends on analog bus capacitance,
    /// chip revision, and temperature. Common values: `0xFF` (nestest), `0xEE` (`SingleStepTests` /
    /// visual6502). Defaults to `0xFF`.
    pub magic: u8,

    /// Level of `/NMI` sampled on the previous [`Cpu2A03::tick`]. Used to detect the
    /// low→high edge that latches a pending NMI: the chip fires exactly one NMI per
    /// rising edge regardless of how long the line stays asserted, so the previous
    /// level is what tells a fresh assertion apart from one we've already latched.
    nmi_previous: bool,

    /// Latched rising edge on `/NMI`, waiting to be serviced at the next fetch boundary.
    /// Set by the edge sampler in [`Cpu2A03::tick`] and cleared either when `Phase::Fetch`
    /// enters NMI service or when `interrupt`'s T5 arm consumes it as a vector hijack.
    /// Kept separate from the live bus level so a one-cycle pulse still fires exactly
    /// one NMI even if `/NMI` drops back low before the boundary.
    pub(crate) nmi_latch: bool,

    /// Frozen interrupt decision from the previous instruction or interrupt sequence,
    /// captured at its second-to-last cycle. Set by [`Cpu2A03::advance`] when a step
    /// returns `Ready`, consumed by `Phase::Fetch` on the next tick. Mirrors the
    /// chip's internal interrupt-grant flip-flop.
    ///
    /// Only a bool; NMI-vs-IRQ kind is re-derived at `Phase::Fetch` from `nmi_latch`
    /// (NMI wins priority if latched). This lets an NMI that arrived after the
    /// second-to-last cycle but before fetch upgrade a pending IRQ to NMI, matching
    /// chip behavior.
    interrupt_pending: bool,
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
            nmi_previous: false,
            nmi_latch: false,
            interrupt_pending: false,
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
        self.nmi_previous = false;
        self.nmi_latch = false;
        self.interrupt_pending = false;
    }

    /// Advances the CPU by one clock cycle.
    pub fn tick<B: CpuBus>(&mut self, bus: &mut B) {
        self.cycles += 1;

        // Sample `/NMI` every cycle and latch a rising edge into `nmi_latch`. The 2A03
        // samples NMI on every clock; a one-cycle pulse is enough to fire exactly one NMI,
        // and holding the line asserted indefinitely still only fires once per assertion.
        // Sampling unconditionally (including during Reset/Halted) matches the chip and
        // also keeps `nmi` aligned with the bus across phase transitions.
        let nmi = bus.nmi();
        self.nmi_latch |= nmi && !self.nmi_previous;
        self.nmi_previous = nmi;

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

            // Advance the 7-cycle hardware reset sequence one step forward.
            Phase::Reset => self.advance(bus, reset),

            // Advance the hardware IRQ/NMI sequence one step forward. Each kind monomorphizes
            // to its own copy of `interrupt` with the vector and pushed-P mask folded in.
            Phase::IRQ => self.advance(bus, IRQ),
            Phase::NMI => self.advance(bus, NMI),

            Phase::Fetch => {
                // The previous instruction's second-to-last-cycle interrupt
                // decision said "service something." NMI wins at dispatch time
                // if it's latched, since a fresh edge that arrived after the
                // decision can upgrade IRQ to NMI here. `advance` hands off to
                // `interrupt`'s T0 arm; the current cycle is spent there as the
                // dummy PC read.
                if self.interrupt_pending {
                    self.interrupt_pending = false;

                    #[expect(clippy::semicolon_if_nothing_returned)]
                    return if self.nmi_latch {
                        self.nmi_latch = false;
                        self.phase = Phase::NMI;

                        self.advance(bus, NMI)
                    } else {
                        self.phase = Phase::IRQ;

                        self.advance(bus, IRQ)
                    };
                }

                // T0: fetch the opcode and cache its handler. T advances to 1 so the first
                // execution T-state enters at T1.
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
                self.advance(bus, instruction);
            }
        }
    }

    /// Advances a multi-cycle pipeline phase by one cycle, and on the last cycle
    /// latches the interrupt decision into `interrupt_pending`.
    ///
    /// Reset, hardware IRQ/NMI, and instruction execution all share the same shape:
    /// call a `Poll<()>`-returning step that consumes one bus cycle, bump `t` while
    /// it returns `Pending`, and on `Ready` snap back to the fetch boundary while
    /// freezing the interrupt decision. The decision uses the I flag captured before
    /// the step runs, so a CLI/SEI/PLP that writes I on its last cycle still lands
    /// its delay-by-one-instruction behavior.
    #[inline]
    fn advance<B: CpuBus>(&mut self, bus: &mut B, step: fn(&mut Self, &mut B) -> Poll<()>) {
        let i = self.p.contains(CpuStatus::I);

        if step(self, bus).is_ready() {
            self.interrupt_pending = self.nmi_latch || (bus.irq() && !i);

            self.t = 0;
            self.phase = Phase::Fetch;
        } else {
            self.t += 1;
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
    pub(crate) fn fetch<B: CpuReadWrite>(&mut self, bus: &mut B) -> u8 {
        let value = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        value
    }

    /// Reconstructs the full 16-bit effective address from `adl` and `adh`.
    /// Used at bus read/write sites once address resolution is complete.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn address(&self) -> u16 {
        u16::from_le_bytes([self.adl, self.adh])
    }

    /// Sets both address bytes together, used when a full 16-bit address is known at once
    /// (e.g. zero-page fetch where `adh` is always `$00`, or indirect target assembly).
    #[inline(always)]
    pub(crate) const fn set_address(&mut self, adl: u8, adh: u8) {
        self.adl = adl;
        self.adh = adh;
    }

    /// Returns the full 16-bit address of the current stack top: `$0100 | SP`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn stack_address(&self) -> u16 {
        0x0100 | self.sp as u16
    }

    /// Writes `value` to `$0100 + SP`, then decrements SP.
    #[inline]
    pub(crate) fn stack_push<B: CpuReadWrite>(&mut self, bus: &mut B, value: u8) {
        bus.write(self.stack_address(), value);
        self.sp = self.sp.wrapping_sub(1);
    }
}

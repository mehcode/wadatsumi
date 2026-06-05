// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bus::Bus;
use crate::cpu::instruction::Instruction;
use crate::cpu::table::InstructionTable;

mod addressing;
mod instruction;
mod operation;
mod status;
mod table;

pub use status::CpuStatus;

/// The 2A03 NES CPU core.
/// Driven one clock cycle at a time via [`Cpu::tick`].
pub struct Cpu2A03<B: Bus> {
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

    /// The current T-state of the in-flight instruction: 0 during the opcode fetch cycle,
    /// incrementing by one each subsequent clock cycle until the instruction completes.
    t: u8,

    /// The instruction fetched at cycle 0, held for the duration of execution.
    /// None when `cycle == 0`.
    instruction: Option<Instruction<B>>,

    /// Scratch register used during indirect addressing to hold the zero-page pointer
    /// byte before it is expanded into a full 16-bit address.
    ptr: u8,

    /// Effective address being assembled across addressing cycles. Holds the final
    /// target address once addressing is complete.
    address: u16,

    /// Single-byte data latch used to pass a value between the read and write cycles
    /// of a read-modify-write instruction.
    data: u8,

    /// Set when the CPU fetches an opcode with no handler. Like the NMOS 6502 KIL/JAM
    /// opcodes, the core then locks up: [`Cpu::tick`] becomes a no-op until [`Cpu::reset`].
    halted: bool,
}

impl<B: Bus> Default for Cpu2A03<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Bus> Cpu2A03<B> {
    const TABLE: InstructionTable<B> = InstructionTable::new();

    #[must_use]
    pub const fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,

            // After the reset sequence the 6502 performs three phantom stack writes,
            // decrementing S from 0xFF to 0xFD.
            sp: 0xFD,

            // I is set by the reset sequence.
            p: CpuStatus::I,

            instruction: None,
            t: 0,
            address: 0,
            ptr: 0,
            data: 0,
            halted: false,
        }
    }

    /// Reads the `/RESET` vector at `$fffc` and `$fffd` and sets the PC to the result.
    /// Call once after the cartridge is attached and the bus is ready.
    pub fn reset(&mut self, bus: &mut B) {
        let lo = u16::from(bus.read(0xfffc));
        let hi = u16::from(bus.read(0xfffd));

        self.pc = (hi << 8) | lo;
        self.halted = false;
    }

    /// Advances the CPU by one clock cycle.
    ///
    /// Fetching an opcode with no handler jams the core (see [`Cpu::halted`]); once
    /// halted, this is a no-op until [`Cpu::reset`] clears the condition.
    pub fn tick(&mut self, bus: &mut B) {
        if self.halted {
            return;
        }

        let Some(instruction) = self.instruction else {
            // T0. Opcode fetch. T advances to 1 so the first execution T-state enters at T1.
            let opcode = self.fetch(bus);

            let Some(handler) = Self::TABLE.get(opcode) else {
                // Illegal/unimplemented opcode: lock up like a KIL/JAM instruction.
                tracing::error!(
                    opcode = format!("{opcode:02X}"),
                    pc = format!("{:04X}", self.pc.wrapping_sub(1)),
                    "illegal opcode; halting CPU"
                );

                self.halted = true;
                return;
            };

            self.t += 1;
            self.instruction = Some(handler);

            return;
        };

        // Execute one T-state of the in-flight instruction; clear it when the handler signals done.
        if instruction(self, bus).is_ready() {
            self.instruction = None;
            self.t = 0;
        } else {
            self.t += 1;
        }
    }

    /// Returns `true` once the CPU has jammed on an illegal opcode. Cleared by [`Cpu::reset`].
    #[must_use]
    pub const fn halted(&self) -> bool {
        self.halted
    }

    /// Returns the current T-state; 0 (T0) indicates the SYNC cycle where the next
    /// opcode will be fetched.
    #[must_use]
    pub const fn t(&self) -> u8 {
        self.t
    }

    /// Reads the byte at PC, and advances PC.
    fn fetch(&mut self, bus: &mut B) -> u8 {
        let value = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        value
    }

    /// Returns the full 16-bit address of the current stack top: `$0100 | SP`.
    #[inline(always)]
    #[must_use]
    const fn stack_address(&self) -> u16 {
        0x0100 | self.sp as u16
    }

    /// Writes `value` to `$0100 + SP`, then decrements SP.
    #[inline]
    fn stack_push(&mut self, bus: &mut B, value: u8) {
        bus.write(self.stack_address(), value);
        self.sp = self.sp.wrapping_sub(1);
    }
}

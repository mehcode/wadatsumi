// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::Error;
use crate::bus::Bus;
use crate::cpu::instruction::Instruction;
use crate::cpu::state::CpuState;
use crate::cpu::table::InstructionTable;

mod addressing;
mod instruction;
mod operation;
mod state;
mod table;

/// The 2A03 NES CPU core.
/// Driven one clock cycle at a time via [`Cpu::tick`].
pub struct Cpu<B: Bus> {
    /// Architectural register state (A, X, Y, S, P, PC). Separated so it can be
    /// snapshotted or inspected independently of micro-architecture scratch.
    pub state: CpuState,

    /// Which micro-operation cycle the current instruction is on. Zero means the
    /// CPU is idle and will fetch the next opcode on the next tick.
    cycle: u8,

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

    /// Set to `true` once the in-flight instruction's addressing mode has resolved and any
    /// operand prefetch has been committed to `cpu.data`; prevents `execute` from re-running
    /// address resolution and the framework bus read on subsequent execution cycles.
    executing: bool,
}

impl<B: Bus> Cpu<B> {
    const TABLE: InstructionTable<B> = InstructionTable::new();

    pub fn new() -> Self {
        Self {
            state: CpuState::new(),
            instruction: None,
            cycle: 0,
            address: 0,
            ptr: 0,
            data: 0,
            executing: false,
        }
    }

    /// Reads the `/RESET` vector at `$fffc` and `$fffd` and sets the PC to the result.
    /// Call once after the [`Pak`] is loaded.
    pub fn reset(&mut self, bus: &mut B) {
        let lo = u16::from(bus.read(0xfffc));
        let hi = u16::from(bus.read(0xfffd));

        self.state.pc = (hi << 8) | lo;
    }

    /// Advances the CPU by one clock cycle.
    pub fn tick(&mut self, bus: &mut B) -> crate::Result<()> {
        let Some(instruction) = self.instruction else {
            // Decode. No instruction is in-flight, so consume this cycle reading the opcode at
            // PC and resolving its handler. `cycle` is reset to 0 so addressing modes see a clean
            // counter starting from 1 on their first post-fetch cycle.
            let opcode = self.fetch(bus);

            self.cycle = 0;
            self.instruction = Some(Self::TABLE.get(opcode).ok_or_else(|| {
                Error::UnknownOpcode { opcode, pc: self.state.pc.wrapping_sub(1) }
            })?);

            return Ok(());
        };

        // Increment before calling so addressing modes see the correct cycle index (1 = first
        // post-fetch cycle, matching the `match cpu.cycle` arms in each AddressingMode impl).
        self.cycle += 1;

        // Execute one cycle of the in-flight instruction; clear it when the handler signals done.
        if instruction(self, bus).is_ready() {
            self.instruction = None;
            self.executing = false;
        }

        Ok(())
    }

    /// Reads the byte at PC, and advances PC.
    fn fetch(&mut self, bus: &mut B) -> u8 {
        let value = bus.read(self.state.pc);
        self.state.pc = self.state.pc.wrapping_add(1);

        value
    }

    /// Writes `value` to `$0100 + SP`, then decrements SP.
    fn stack_push(&mut self, bus: &mut B, value: u8) {
        bus.write(self.state.stack_address(), value);
        self.state.sp = self.state.sp.wrapping_sub(1);
    }
}

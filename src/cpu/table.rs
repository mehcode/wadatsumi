// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bus::Bus;
use crate::cpu::addressing::{
    Absolute, AbsoluteX, AbsoluteY, AddressingMode, Immediate, Implied, Indirect, IndirectX,
    IndirectY, Relative, ZeroPage, ZeroPageX, ZeroPageY,
};
use crate::cpu::instruction::{Instruction, execute};
use crate::cpu::operation::{
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS, JMP, JSR, LDA, LDX, LDY, Operation, RTS, STA, STX, STY,
};
use crate::cpu::state::Register;

/// Dispatch table mapping all 256 6502/2A03 opcodes to their [`Instruction`] handlers.
///
/// Slots for unimplemented opcodes are `None`; the CPU should treat those as
/// illegal instructions.
pub struct InstructionTable<B: Bus> {
    instructions: [Option<Instruction<B>>; 256],
}

impl<B: Bus> InstructionTable<B> {
    /// Builds the table with every implemented opcode wired to its handler.
    pub const fn new() -> Self {
        use Register::*;

        let mut table = Self { instructions: [None; 256] };

        // Load Register from Memory [LDA, LDX, LDY]
        table.insert::<LDA, Immediate>(0xa9);
        table.insert::<LDA, ZeroPage>(0xa5);
        table.insert::<LDA, ZeroPageX>(0xb5);
        table.insert::<LDA, Absolute>(0xad);
        table.insert::<LDA, AbsoluteX>(0xbd);
        table.insert::<LDA, AbsoluteY>(0xb9);
        table.insert::<LDA, IndirectX>(0xa1);
        table.insert::<LDA, IndirectY>(0xb1);
        table.insert::<LDX, Immediate>(0xa2);
        table.insert::<LDX, ZeroPage>(0xa6);
        table.insert::<LDX, ZeroPageY>(0xb6);
        table.insert::<LDX, Absolute>(0xae);
        table.insert::<LDX, AbsoluteY>(0xbe);
        table.insert::<LDY, Immediate>(0xa0);
        table.insert::<LDY, ZeroPage>(0xa4);
        table.insert::<LDY, ZeroPageX>(0xb4);
        table.insert::<LDY, Absolute>(0xac);
        table.insert::<LDY, AbsoluteX>(0xbc);

        // Store Register in Memory [STA, STX, STY]
        table.insert::<STA, ZeroPage>(0x85);
        table.insert::<STA, ZeroPageX>(0x95);
        table.insert::<STA, Absolute>(0x8d);
        table.insert::<STA, AbsoluteX>(0x9d);
        table.insert::<STA, AbsoluteY>(0x99);
        table.insert::<STA, IndirectX>(0x81);
        table.insert::<STA, IndirectY>(0x91);
        table.insert::<STX, ZeroPage>(0x86);
        table.insert::<STX, ZeroPageY>(0x96);
        table.insert::<STX, Absolute>(0x8e);
        table.insert::<STY, ZeroPage>(0x84);
        table.insert::<STY, ZeroPageX>(0x94);
        table.insert::<STY, Absolute>(0x8c);

        // Jumps, Calls, Returns
        table.insert::<JMP, Absolute>(0x4c);
        table.insert::<JMP, Indirect>(0x6c);
        table.insert::<JSR, Implied>(0x20);
        table.insert::<RTS, Implied>(0x60);

        // Conditional Branches
        table.insert::<BPL, Relative>(0x10);
        table.insert::<BMI, Relative>(0x30);
        table.insert::<BVC, Relative>(0x50);
        table.insert::<BVS, Relative>(0x70);
        table.insert::<BCC, Relative>(0x90);
        table.insert::<BCS, Relative>(0xb0);
        table.insert::<BNE, Relative>(0xd0);
        table.insert::<BEQ, Relative>(0xf0);

        table
    }

    /// Registers `execute::<B, O, A>` as the handler for `opcode`.
    const fn insert<O: Operation, A: AddressingMode>(&mut self, opcode: u8) {
        self.instructions[opcode as usize] = Some(execute::<B, O, A>);
    }

    /// Returns the handler for `opcode`, or `None` if the opcode is not implemented.
    pub const fn get(&self, opcode: u8) -> Option<Instruction<B>> {
        self.instructions[opcode as usize]
    }
}

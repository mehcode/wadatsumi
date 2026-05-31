// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bus::Bus;
use crate::cpu::addressing::{
    Absolute, AbsoluteX, AbsoluteY, AddressingMode, Immediate, Implied, Indirect, IndirectX,
    IndirectY, Relative, ZeroPage, ZeroPageX, ZeroPageY,
};
use crate::cpu::instruction::{Instruction, execute};
use crate::cpu::operation::{
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS, CLC, CLD, CLI, CLV, DEC, DEX, DEY, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, NOP, Operation, PHA, PHP, PLA, PLP, RTS, SEC, SED, SEI, STA, STX, STY, TAX,
    TAY, TSX, TXA, TXS, TYA,
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

        // Register to Register Transfer [TAX, TAY, TSX, TXA, TYA, TXS]
        table.insert::<TAY, Implied>(0xa8);
        table.insert::<TAX, Implied>(0xaa);
        table.insert::<TSX, Implied>(0xba);
        table.insert::<TXA, Implied>(0x8a);
        table.insert::<TYA, Implied>(0x98);
        table.insert::<TXS, Implied>(0x9a);

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

        // Push, Pull
        table.insert::<PHA, Implied>(0x48);
        table.insert::<PHP, Implied>(0x08);
        table.insert::<PLA, Implied>(0x68);
        table.insert::<PLP, Implied>(0x28);

        // Increment by one
        table.insert::<INC, ZeroPage>(0xe6);
        table.insert::<INC, ZeroPageX>(0xf6);
        table.insert::<INC, Absolute>(0xee);
        table.insert::<INC, AbsoluteX>(0xfe);
        table.insert::<INX, Implied>(0xe8);
        table.insert::<INY, Implied>(0xc8);

        // Decrement by one
        table.insert::<DEC, ZeroPage>(0xc6);
        table.insert::<DEC, ZeroPageX>(0xd6);
        table.insert::<DEC, Absolute>(0xce);
        table.insert::<DEC, AbsoluteX>(0xde);
        table.insert::<DEX, Implied>(0xca);
        table.insert::<DEY, Implied>(0x88);

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

        // CPU Control
        table.insert::<CLC, Implied>(0x18);
        table.insert::<CLI, Implied>(0x58);
        table.insert::<CLD, Implied>(0xd8);
        table.insert::<CLV, Implied>(0xb8);
        table.insert::<SEC, Implied>(0x38);
        table.insert::<SEI, Implied>(0x78);
        table.insert::<SED, Implied>(0xf8);

        // No Operation
        table.insert::<NOP, Implied>(0xea);

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

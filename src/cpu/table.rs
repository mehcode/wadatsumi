// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bus::Bus;
use crate::cpu::addressing::{
    Absolute, AbsoluteX, AbsoluteY, AddressingMode, Immediate, Implied, Indirect, IndirectX,
    IndirectY, Relative, ZeroPage, ZeroPageX, ZeroPageY,
};
use crate::cpu::instruction::{Instruction, execute};
use crate::cpu::operation::{
    ADC, ALR, ANC, AND, ARR, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC, CLD, CLI,
    CLV, CMP, CPX, CPY, DCP, DEC, DEX, DEY, EOR, INC, INX, INY, ISC, JMP, JSR, LAX, LDA, LDX, LDY,
    LSR, LXA, NOP, ORA, Operation, PHA, PHP, PLA, PLP, RLA, ROL, ROR, RRA, RTI, RTS, SAX, SBC, SBX,
    SEC, SED, SEI, SHA, SHX, SHY, SLO, SRE, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
};

/// Dispatch table mapping all 256 6502/2A03 opcodes to their [`Instruction`] handlers.
///
/// Slots for unimplemented opcodes are `None`; the CPU should treat those as
/// illegal instructions.
pub struct InstructionTable<B: Bus> {
    instructions: [Option<Instruction<B>>; 256],
}

impl<B: Bus> InstructionTable<B> {
    /// Builds the table with every implemented opcode wired to its handler.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub const fn new() -> Self {
        use crate::cpu::operation::Operand::{Memory, Register};
        use crate::cpu::operation::Register::A;

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

        // Push, Pull [PHA, PHP, PLA, PLP]
        table.insert::<PHA, Implied>(0x48);
        table.insert::<PHP, Implied>(0x08);
        table.insert::<PLA, Implied>(0x68);
        table.insert::<PLP, Implied>(0x28);

        // Add memory to accumulator with carry [ADC]
        table.insert::<ADC, Immediate>(0x69);
        table.insert::<ADC, ZeroPage>(0x65);
        table.insert::<ADC, ZeroPageX>(0x75);
        table.insert::<ADC, Absolute>(0x6d);
        table.insert::<ADC, AbsoluteX>(0x7d);
        table.insert::<ADC, AbsoluteY>(0x79);
        table.insert::<ADC, IndirectX>(0x61);
        table.insert::<ADC, IndirectY>(0x71);

        // Subtract memory from accumulator with borrow [SBC]
        table.insert::<SBC, Immediate>(0xe9);
        table.insert::<SBC, ZeroPage>(0xe5);
        table.insert::<SBC, ZeroPageX>(0xf5);
        table.insert::<SBC, Absolute>(0xed);
        table.insert::<SBC, AbsoluteX>(0xfd);
        table.insert::<SBC, AbsoluteY>(0xf9);
        table.insert::<SBC, IndirectX>(0xe1);
        table.insert::<SBC, IndirectY>(0xf1);

        // Logical AND memory with accumulator [AND]
        table.insert::<AND, Immediate>(0x29);
        table.insert::<AND, ZeroPage>(0x25);
        table.insert::<AND, ZeroPageX>(0x35);
        table.insert::<AND, Absolute>(0x2d);
        table.insert::<AND, AbsoluteX>(0x3d);
        table.insert::<AND, AbsoluteY>(0x39);
        table.insert::<AND, IndirectX>(0x21);
        table.insert::<AND, IndirectY>(0x31);

        // Exclusive OR memory with accumulator [EOR]
        table.insert::<EOR, Immediate>(0x49);
        table.insert::<EOR, ZeroPage>(0x45);
        table.insert::<EOR, ZeroPageX>(0x55);
        table.insert::<EOR, Absolute>(0x4d);
        table.insert::<EOR, AbsoluteX>(0x5d);
        table.insert::<EOR, AbsoluteY>(0x59);
        table.insert::<EOR, IndirectX>(0x41);
        table.insert::<EOR, IndirectY>(0x51);

        // Logical OR memory with accumulator [ORA]
        table.insert::<ORA, Immediate>(0x09);
        table.insert::<ORA, ZeroPage>(0x05);
        table.insert::<ORA, ZeroPageX>(0x15);
        table.insert::<ORA, Absolute>(0x0d);
        table.insert::<ORA, AbsoluteX>(0x1d);
        table.insert::<ORA, AbsoluteY>(0x19);
        table.insert::<ORA, IndirectX>(0x01);
        table.insert::<ORA, IndirectY>(0x11);

        // Compare [CMP, CPX, CPY]
        table.insert::<CMP, Immediate>(0xc9);
        table.insert::<CMP, ZeroPage>(0xc5);
        table.insert::<CMP, ZeroPageX>(0xd5);
        table.insert::<CMP, Absolute>(0xcd);
        table.insert::<CMP, AbsoluteX>(0xdd);
        table.insert::<CMP, AbsoluteY>(0xd9);
        table.insert::<CMP, IndirectX>(0xc1);
        table.insert::<CMP, IndirectY>(0xd1);
        table.insert::<CPX, Immediate>(0xe0);
        table.insert::<CPX, ZeroPage>(0xe4);
        table.insert::<CPX, Absolute>(0xec);
        table.insert::<CPY, Immediate>(0xc0);
        table.insert::<CPY, ZeroPage>(0xc4);
        table.insert::<CPY, Absolute>(0xcc);

        // Bit Test [BIT]
        table.insert::<BIT, ZeroPage>(0x24);
        table.insert::<BIT, Absolute>(0x2c);

        // Increment by one [INC, INX, INY]
        table.insert::<INC, ZeroPage>(0xe6);
        table.insert::<INC, ZeroPageX>(0xf6);
        table.insert::<INC, Absolute>(0xee);
        table.insert::<INC, AbsoluteX>(0xfe);
        table.insert::<INX, Implied>(0xe8);
        table.insert::<INY, Implied>(0xc8);

        // Decrement by one [DEC, DEX, DEY]
        table.insert::<DEC, ZeroPage>(0xc6);
        table.insert::<DEC, ZeroPageX>(0xd6);
        table.insert::<DEC, Absolute>(0xce);
        table.insert::<DEC, AbsoluteX>(0xde);
        table.insert::<DEX, Implied>(0xca);
        table.insert::<DEY, Implied>(0x88);

        // Arithmetic Shift Left [ASL]
        table.insert::<ASL<{ Register(A) }>, Implied>(0x0a);
        table.insert::<ASL<{ Memory }>, ZeroPage>(0x06);
        table.insert::<ASL<{ Memory }>, ZeroPageX>(0x16);
        table.insert::<ASL<{ Memory }>, Absolute>(0x0e);
        table.insert::<ASL<{ Memory }>, AbsoluteX>(0x1e);

        // Shift Right Logical [LSR]
        table.insert::<LSR<{ Register(A) }>, Implied>(0x4a);
        table.insert::<LSR<{ Memory }>, ZeroPage>(0x46);
        table.insert::<LSR<{ Memory }>, ZeroPageX>(0x56);
        table.insert::<LSR<{ Memory }>, Absolute>(0x4e);
        table.insert::<LSR<{ Memory }>, AbsoluteX>(0x5e);

        // Rotate Left through Carry [ROL]
        table.insert::<ROL<{ Register(A) }>, Implied>(0x2a);
        table.insert::<ROL<{ Memory }>, ZeroPage>(0x26);
        table.insert::<ROL<{ Memory }>, ZeroPageX>(0x36);
        table.insert::<ROL<{ Memory }>, Absolute>(0x2e);
        table.insert::<ROL<{ Memory }>, AbsoluteX>(0x3e);

        // Rotate Right through Carry [ROR]
        table.insert::<ROR<{ Register(A) }>, Implied>(0x6a);
        table.insert::<ROR<{ Memory }>, ZeroPage>(0x66);
        table.insert::<ROR<{ Memory }>, ZeroPageX>(0x76);
        table.insert::<ROR<{ Memory }>, Absolute>(0x6e);
        table.insert::<ROR<{ Memory }>, AbsoluteX>(0x7e);

        // Jumps, Calls, Returns [JMP, JSR, RTS, RTI]
        table.insert::<JMP, Absolute>(0x4c);
        table.insert::<JMP, Indirect>(0x6c);
        table.insert::<JSR, Implied>(0x20);
        table.insert::<RTS, Implied>(0x60);
        table.insert::<RTI, Implied>(0x40);

        // Conditional Branches [BPL, BMI, BVC, BVS, BCC, BCS, BNE, BEQ]
        table.insert::<BPL, Relative>(0x10);
        table.insert::<BMI, Relative>(0x30);
        table.insert::<BVC, Relative>(0x50);
        table.insert::<BVS, Relative>(0x70);
        table.insert::<BCC, Relative>(0x90);
        table.insert::<BCS, Relative>(0xb0);
        table.insert::<BNE, Relative>(0xd0);
        table.insert::<BEQ, Relative>(0xf0);

        // Breakpoint [BRK]
        table.insert::<BRK, Implied>(0x00);

        // CPU Control [CLC, CLD, CLI, CLV, SEC, SED, SEI]
        table.insert::<CLC, Implied>(0x18);
        table.insert::<CLI, Implied>(0x58);
        table.insert::<CLD, Implied>(0xd8);
        table.insert::<CLV, Implied>(0xb8);
        table.insert::<SEC, Implied>(0x38);
        table.insert::<SEI, Implied>(0x78);
        table.insert::<SED, Implied>(0xf8);

        // No Operation [NOP]
        table.insert::<NOP, Implied>(0xea);

        // LDA + TAX combined (unofficial) [LAX]
        table.insert::<LAX, IndirectX>(0xa3);
        table.insert::<LAX, ZeroPage>(0xa7);
        table.insert::<LAX, Absolute>(0xaf);
        table.insert::<LAX, IndirectY>(0xb3);
        table.insert::<LAX, ZeroPageY>(0xb7);
        table.insert::<LAX, AbsoluteY>(0xbf);

        // AND (A | MAGIC) with immediate, then load A and X (unofficial) [LXA]
        table.insert::<LXA, Immediate>(0xab);

        // Store A & X in memory (unofficial) [SAX]
        table.insert::<SAX, ZeroPage>(0x87);
        table.insert::<SAX, ZeroPageY>(0x97);
        table.insert::<SAX, Absolute>(0x8f);
        table.insert::<SAX, IndirectX>(0x83);

        // Store AND (baseAddrHigh + 1) into memory (unofficial) [SHA, SHX, SHY]
        table.insert::<SHA, AbsoluteY>(0x9f);
        table.insert::<SHA, IndirectY>(0x93);
        table.insert::<SHX, AbsoluteY>(0x9e);
        table.insert::<SHY, AbsoluteX>(0x9c);

        // Subtract memory from accumulator with borrow (unofficial) [SBC]
        table.insert::<SBC, Immediate>(0xeb);

        // AND A,X then subtract immediate; result → X (unofficial) [SBX]
        table.insert::<SBX, Immediate>(0xcb);

        // AND accumulator with carry (unofficial) [ANC]
        table.insert::<ANC, Immediate>(0x0b);
        table.insert::<ANC, Immediate>(0x2b);

        // ASL operand + ORA operand (unofficial) [SLO]
        table.insert::<SLO, ZeroPage>(0x07);
        table.insert::<SLO, ZeroPageX>(0x17);
        table.insert::<SLO, Absolute>(0x0f);
        table.insert::<SLO, AbsoluteX>(0x1f);
        table.insert::<SLO, AbsoluteY>(0x1b);
        table.insert::<SLO, IndirectX>(0x03);
        table.insert::<SLO, IndirectY>(0x13);

        // LSR operand + EOR accumulator (unofficial) [SRE]
        table.insert::<SRE, ZeroPage>(0x47);
        table.insert::<SRE, ZeroPageX>(0x57);
        table.insert::<SRE, Absolute>(0x4f);
        table.insert::<SRE, AbsoluteX>(0x5f);
        table.insert::<SRE, AbsoluteY>(0x5b);
        table.insert::<SRE, IndirectX>(0x43);
        table.insert::<SRE, IndirectY>(0x53);

        // AND immediate + LSR accumulator (unofficial) [ALR]
        table.insert::<ALR, Immediate>(0x4b);

        // AND immediate + ROR accumulator (unofficial) [ARR]
        table.insert::<ARR, Immediate>(0x6b);

        // ROL operand + AND accumulator (unofficial) [RLA]
        table.insert::<RLA, ZeroPage>(0x27);
        table.insert::<RLA, ZeroPageX>(0x37);
        table.insert::<RLA, Absolute>(0x2f);
        table.insert::<RLA, AbsoluteX>(0x3f);
        table.insert::<RLA, AbsoluteY>(0x3b);
        table.insert::<RLA, IndirectX>(0x23);
        table.insert::<RLA, IndirectY>(0x33);

        // ROR operand + ADC operand (unofficial) [RRA]
        table.insert::<RRA, ZeroPage>(0x67);
        table.insert::<RRA, ZeroPageX>(0x77);
        table.insert::<RRA, Absolute>(0x6f);
        table.insert::<RRA, AbsoluteX>(0x7f);
        table.insert::<RRA, AbsoluteY>(0x7b);
        table.insert::<RRA, IndirectX>(0x63);
        table.insert::<RRA, IndirectY>(0x73);

        // DEC memory + CMP accumulator (unofficial) [DCP]
        table.insert::<DCP, IndirectX>(0xc3);
        table.insert::<DCP, ZeroPage>(0xc7);
        table.insert::<DCP, ZeroPageX>(0xd7);
        table.insert::<DCP, Absolute>(0xcf);
        table.insert::<DCP, AbsoluteX>(0xdf);
        table.insert::<DCP, AbsoluteY>(0xdb);
        table.insert::<DCP, IndirectY>(0xd3);

        // INC memory + SBC accumulator (unofficial) [ISC]
        table.insert::<ISC, IndirectX>(0xe3);
        table.insert::<ISC, ZeroPage>(0xe7);
        table.insert::<ISC, ZeroPageX>(0xf7);
        table.insert::<ISC, Absolute>(0xef);
        table.insert::<ISC, AbsoluteX>(0xff);
        table.insert::<ISC, AbsoluteY>(0xfb);
        table.insert::<ISC, IndirectY>(0xf3);

        // No Operation (unofficial) [NOP]
        table.insert::<NOP, Implied>(0x1a);
        table.insert::<NOP, Implied>(0x3a);
        table.insert::<NOP, Implied>(0x5a);
        table.insert::<NOP, Implied>(0x7a);
        table.insert::<NOP, Implied>(0xda);
        table.insert::<NOP, Implied>(0xfa);
        table.insert::<NOP, Immediate>(0x80);
        table.insert::<NOP, Immediate>(0x82);
        table.insert::<NOP, Immediate>(0x89);
        table.insert::<NOP, Immediate>(0xc2);
        table.insert::<NOP, Immediate>(0xe2);
        table.insert::<NOP, ZeroPage>(0x04);
        table.insert::<NOP, ZeroPage>(0x44);
        table.insert::<NOP, ZeroPage>(0x64);
        table.insert::<NOP, ZeroPageX>(0x14);
        table.insert::<NOP, ZeroPageX>(0x34);
        table.insert::<NOP, ZeroPageX>(0x54);
        table.insert::<NOP, ZeroPageX>(0x74);
        table.insert::<NOP, ZeroPageX>(0xd4);
        table.insert::<NOP, ZeroPageX>(0xf4);
        table.insert::<NOP, Absolute>(0x0c);
        table.insert::<NOP, AbsoluteX>(0x1c);
        table.insert::<NOP, AbsoluteX>(0x3c);
        table.insert::<NOP, AbsoluteX>(0x5c);
        table.insert::<NOP, AbsoluteX>(0x7c);
        table.insert::<NOP, AbsoluteX>(0xdc);
        table.insert::<NOP, AbsoluteX>(0xfc);

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

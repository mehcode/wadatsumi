use std::vec;
use std::ops::Index;
use std::fmt::Write;
use std::string::String;
use strfmt;

use ::op;
use ::bus::Bus;
use ::cpu::Context;

#[derive(Default)]
pub struct Operation {
    // Function to handle the operation
    pub handle: Option<fn(&mut Context, &mut Bus) -> ()>,

    // String format of operation for disassembly
    pub disassembly: &'static str,

    // Number of bytes (incl. opcode); used for disassembly
    pub size: u8,
}

impl Operation {
    fn empty() -> Self {
        Default::default()
    }

    fn new(handle: fn(&mut Context, &mut Bus) -> (), disassembly: &'static str, size: u8) -> Self {
        Operation {
            handle: Some(handle),
            disassembly: disassembly,
            size: size,
        }
    }

    pub fn format(&self, _: &Context, _: &mut Bus) -> Result<String, strfmt::FmtError> {
        strfmt::strfmt_map(self.disassembly,
                           &|mut fmt: strfmt::Formatter| {
            // TODO(rust): This library seems to want me to use unwrap here which smells
            if fmt.key == "0" {
                fmt.write_str("?").unwrap()
            } else {
                fmt.write_str("??").unwrap()
            }

            Ok(())
        })
    }
}

pub struct Table {
    // Operation table
    //  + 0x000 - 0x00 - 0xFF
    //  + 0x100 - 0xCB00 - 0xCBFF
    operations: vec::Vec<Operation>,
}

impl Default for Table {
    fn default() -> Self {
        let mut operations = vec![Operation::new(op::_00, "NOP", 1),
                                  Operation::new(op::_01, "LD BC, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_02, "LD (BC), A", 1),
                                  Operation::new(op::_03, "INC BC", 1),
                                  Operation::new(op::_04, "INC B", 1),
                                  Operation::new(op::_05, "DEC B", 1),
                                  Operation::new(op::_06, "LD B, 0x{0:X}", 2),
                                  Operation::new(op::_07, "RLCA", 1),
                                  Operation::new(op::_08, "LD (0x{1:X}{0:X}), SP", 3),
                                  Operation::new(op::_09, "ADD HL, BC", 1),
                                  Operation::new(op::_0A, "LD A, (BC)", 1),
                                  Operation::new(op::_0B, "DEC BC", 1),
                                  Operation::new(op::_0C, "INC C", 1),
                                  Operation::new(op::_0D, "DEC C", 1),
                                  Operation::new(op::_0E, "LD C, 0x{0:X}", 2),
                                  Operation::new(op::_0F, "RRCA", 1),

                                  Operation::new(op::_10, "STOP", 1),
                                  Operation::new(op::_11, "LD DE, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_12, "LD (DE), A", 1),
                                  Operation::new(op::_13, "INC DE", 1),
                                  Operation::new(op::_14, "INC D", 1),
                                  Operation::new(op::_15, "DEC D", 1),
                                  Operation::new(op::_16, "LD D, 0x{0:X}", 2),
                                  Operation::new(op::_17, "RLA", 1),
                                  Operation::new(op::_18, "JR 0x{0:X}", 2),
                                  Operation::new(op::_19, "ADD HL, DE", 1),
                                  Operation::new(op::_1A, "LD A, (DE)", 1),
                                  Operation::new(op::_1B, "DEC DE", 1),
                                  Operation::new(op::_1C, "INC E", 1),
                                  Operation::new(op::_1D, "DEC E", 1),
                                  Operation::new(op::_1E, "LD E, 0x{0:X}", 2),
                                  Operation::new(op::_1F, "RRA", 1),

                                  Operation::new(op::_20, "JR NZ, 0x{0:X}", 2),
                                  Operation::new(op::_21, "LD HL, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_22, "LDI (HL), A", 1),
                                  Operation::new(op::_23, "INC HL", 1),
                                  Operation::new(op::_24, "INC H", 1),
                                  Operation::new(op::_25, "DEC H", 1),
                                  Operation::new(op::_26, "LD H, 0x{0:X}", 2),
                                  Operation::new(op::_27, "DAA", 1),
                                  Operation::new(op::_28, "JR Z, 0x{0:X}", 2),
                                  Operation::new(op::_29, "ADD HL, HL", 1),
                                  Operation::new(op::_2A, "LDI A, (HL)", 1),
                                  Operation::new(op::_2B, "DEC HL", 1),
                                  Operation::new(op::_2C, "INC L", 1),
                                  Operation::new(op::_2D, "DEC L", 1),
                                  Operation::new(op::_2E, "LD L, 0x{0:X}", 2),
                                  Operation::new(op::_2F, "CPL", 1),

                                  Operation::new(op::_30, "JR NC, 0x{0:X}", 2),
                                  Operation::new(op::_31, "LD SP, (0x{1:X}{0:X})", 3),
                                  Operation::new(op::_32, "LDD (HL), A", 1),
                                  Operation::new(op::_33, "INC SP", 1),
                                  Operation::new(op::_34, "INC (HL)", 1),
                                  Operation::new(op::_35, "DEC (HL)", 1),
                                  Operation::new(op::_36, "LD (HL), 0x{0:X}", 2),
                                  Operation::new(op::_37, "SCF", 1),
                                  Operation::new(op::_38, "JR C, 0x{0:X}", 2),
                                  Operation::new(op::_39, "ADD HL, SP", 1),
                                  Operation::new(op::_3A, "LDD A, (HL)", 1),
                                  Operation::new(op::_3B, "DEC SP", 1),
                                  Operation::new(op::_3C, "INC A", 1),
                                  Operation::new(op::_3D, "DEC A", 1),
                                  Operation::new(op::_3E, "LD A, 0x{0:X}", 2),
                                  Operation::new(op::_3F, "CCF", 1),

                                  Operation::new(op::_40, "LD B, B", 1),
                                  Operation::new(op::_41, "LD B, C", 1),
                                  Operation::new(op::_42, "LD B, D", 1),
                                  Operation::new(op::_43, "LD B, E", 1),
                                  Operation::new(op::_44, "LD B, H", 1),
                                  Operation::new(op::_45, "LD B, L", 1),
                                  Operation::new(op::_46, "LD B, (HL)", 1),
                                  Operation::new(op::_47, "LD B, A", 1),
                                  Operation::new(op::_48, "LD C, B", 1),
                                  Operation::new(op::_49, "LD C, C", 1),
                                  Operation::new(op::_4A, "LD C, D", 1),
                                  Operation::new(op::_4B, "LD C, E", 1),
                                  Operation::new(op::_4C, "LD C, H", 1),
                                  Operation::new(op::_4D, "LD C, L", 1),
                                  Operation::new(op::_4E, "LD C, (HL)", 1),
                                  Operation::new(op::_4F, "LD C, A", 1),

                                  Operation::new(op::_50, "LD D, B", 1),
                                  Operation::new(op::_51, "LD D, C", 1),
                                  Operation::new(op::_52, "LD D, D", 1),
                                  Operation::new(op::_53, "LD D, E", 1),
                                  Operation::new(op::_54, "LD D, H", 1),
                                  Operation::new(op::_55, "LD D, L", 1),
                                  Operation::new(op::_56, "LD D, (HL)", 1),
                                  Operation::new(op::_57, "LD D, A", 1),
                                  Operation::new(op::_58, "LD E, B", 1),
                                  Operation::new(op::_59, "LD E, C", 1),
                                  Operation::new(op::_5A, "LD E, D", 1),
                                  Operation::new(op::_5B, "LD E, E", 1),
                                  Operation::new(op::_5C, "LD E, H", 1),
                                  Operation::new(op::_5D, "LD E, L", 1),
                                  Operation::new(op::_5E, "LD E, (HL)", 1),
                                  Operation::new(op::_5F, "LD E, A", 1),

                                  Operation::new(op::_60, "LD H, B", 1),
                                  Operation::new(op::_61, "LD H, C", 1),
                                  Operation::new(op::_62, "LD H, D", 1),
                                  Operation::new(op::_63, "LD H, E", 1),
                                  Operation::new(op::_64, "LD H, H", 1),
                                  Operation::new(op::_65, "LD H, L", 1),
                                  Operation::new(op::_66, "LD H, (HL)", 1),
                                  Operation::new(op::_67, "LD H, A", 1),
                                  Operation::new(op::_68, "LD L, B", 1),
                                  Operation::new(op::_69, "LD L, C", 1),
                                  Operation::new(op::_6A, "LD L, D", 1),
                                  Operation::new(op::_6B, "LD L, E", 1),
                                  Operation::new(op::_6C, "LD L, H", 1),
                                  Operation::new(op::_6D, "LD L, L", 1),
                                  Operation::new(op::_6E, "LD L, (HL)", 1),
                                  Operation::new(op::_6F, "LD L, A", 1),

                                  Operation::new(op::_70, "LD (HL), B", 1),
                                  Operation::new(op::_71, "LD (HL), C", 1),
                                  Operation::new(op::_72, "LD (HL), D", 1),
                                  Operation::new(op::_73, "LD (HL), E", 1),
                                  Operation::new(op::_74, "LD (HL), H", 1),
                                  Operation::new(op::_75, "LD (HL), L", 1),
                                  Operation::new(op::_76, "HALT", 1),
                                  Operation::new(op::_77, "LD (HL), A", 1),
                                  Operation::new(op::_78, "LD A, B", 1),
                                  Operation::new(op::_79, "LD A, C", 1),
                                  Operation::new(op::_7A, "LD A, D", 1),
                                  Operation::new(op::_7B, "LD A, E", 1),
                                  Operation::new(op::_7C, "LD A, H", 1),
                                  Operation::new(op::_7D, "LD A, L", 1),
                                  Operation::new(op::_7E, "LD A, (HL)", 1),
                                  Operation::new(op::_7F, "LD A, A", 1),

                                  Operation::new(op::_80, "ADD A, B", 1),
                                  Operation::new(op::_81, "ADD A, C", 1),
                                  Operation::new(op::_82, "ADD A, D", 1),
                                  Operation::new(op::_83, "ADD A, E", 1),
                                  Operation::new(op::_84, "ADD A, H", 1),
                                  Operation::new(op::_85, "ADD A, L", 1),
                                  Operation::new(op::_86, "ADD A, (HL)", 1),
                                  Operation::new(op::_87, "ADD A, A", 1),
                                  Operation::new(op::_88, "ADC A, B", 1),
                                  Operation::new(op::_89, "ADC A, C", 1),
                                  Operation::new(op::_8A, "ADC A, D", 1),
                                  Operation::new(op::_8B, "ADC A, E", 1),
                                  Operation::new(op::_8C, "ADC A, H", 1),
                                  Operation::new(op::_8D, "ADC A, L", 1),
                                  Operation::new(op::_8E, "ADC A, (HL)", 1),
                                  Operation::new(op::_8F, "ADC A, A", 1),

                                  Operation::new(op::_90, "SUB A, B", 1),
                                  Operation::new(op::_91, "SUB A, C", 1),
                                  Operation::new(op::_92, "SUB A, D", 1),
                                  Operation::new(op::_93, "SUB A, E", 1),
                                  Operation::new(op::_94, "SUB A, H", 1),
                                  Operation::new(op::_95, "SUB A, L", 1),
                                  Operation::new(op::_96, "SUB A, (HL)", 1),
                                  Operation::new(op::_97, "SUB A, A", 1),
                                  Operation::new(op::_98, "SBC A, B", 1),
                                  Operation::new(op::_99, "SBC A, C", 1),
                                  Operation::new(op::_9A, "SBC A, D", 1),
                                  Operation::new(op::_9B, "SBC A, E", 1),
                                  Operation::new(op::_9C, "SBC A, H", 1),
                                  Operation::new(op::_9D, "SBC A, L", 1),
                                  Operation::new(op::_9E, "SBC A, (HL)", 1),
                                  Operation::new(op::_9F, "SBC A, A", 1),

                                  Operation::new(op::_A0, "AND A, B", 1),
                                  Operation::new(op::_A1, "AND A, C", 1),
                                  Operation::new(op::_A2, "AND A, D", 1),
                                  Operation::new(op::_A3, "AND A, E", 1),
                                  Operation::new(op::_A4, "AND A, H", 1),
                                  Operation::new(op::_A5, "AND A, L", 1),
                                  Operation::new(op::_A6, "AND A, (HL)", 1),
                                  Operation::new(op::_A7, "AND A, A", 1),
                                  Operation::new(op::_A8, "XOR A, B", 1),
                                  Operation::new(op::_A9, "XOR A, C", 1),
                                  Operation::new(op::_AA, "XOR A, D", 1),
                                  Operation::new(op::_AB, "XOR A, E", 1),
                                  Operation::new(op::_AC, "XOR A, H", 1),
                                  Operation::new(op::_AD, "XOR A, L", 1),
                                  Operation::new(op::_AE, "XOR A, (HL)", 1),
                                  Operation::new(op::_AF, "XOR A, A", 1),

                                  Operation::new(op::_B0, "OR A, B", 1),
                                  Operation::new(op::_B1, "OR A, C", 1),
                                  Operation::new(op::_B2, "OR A, D", 1),
                                  Operation::new(op::_B3, "OR A, E", 1),
                                  Operation::new(op::_B4, "OR A, H", 1),
                                  Operation::new(op::_B5, "OR A, L", 1),
                                  Operation::new(op::_B6, "OR A, (HL)", 1),
                                  Operation::new(op::_B7, "OR A, A", 1),
                                  Operation::new(op::_B8, "CP A, B", 1),
                                  Operation::new(op::_B9, "CP A, C", 1),
                                  Operation::new(op::_BA, "CP A, D", 1),
                                  Operation::new(op::_BB, "CP A, E", 1),
                                  Operation::new(op::_BC, "CP A, H", 1),
                                  Operation::new(op::_BD, "CP A, L", 1),
                                  Operation::new(op::_BE, "CP A, (HL)", 1),
                                  Operation::new(op::_BF, "CP A, A", 1),

                                  Operation::new(op::_C0, "RET NZ", 1),
                                  Operation::new(op::_C1, "POP BC", 1),
                                  Operation::new(op::_C2, "JP NZ, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_C3, "JP 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_C4, "CALL NZ, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_C5, "PUSH BC", 1),
                                  Operation::new(op::_C6, "ADD A, 0x{0:X}", 2),
                                  Operation::new(op::_C7, "RST 0x00", 1),
                                  Operation::new(op::_C8, "RET Z", 1),
                                  Operation::new(op::_C9, "RET", 1),
                                  Operation::new(op::_CA, "JP Z, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_CC, "CALL Z, 0x{1:X}{0:X}", 3),
                                  Operation::empty(),
                                  Operation::new(op::_CD, "CALL 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_CE, "ADC A, 0x{0:X}", 2),
                                  Operation::new(op::_CF, "RST 0x08", 1),

                                  Operation::new(op::_D0, "RET NC", 1),
                                  Operation::new(op::_D1, "POP DE", 1),
                                  Operation::new(op::_D2, "JP NC, 0x{1:X}{0:X}", 3),
                                  Operation::empty(),
                                  Operation::new(op::_D4, "CALL NC, 0x{1:X}{0:X}", 3),
                                  Operation::new(op::_D5, "PUSH DE", 1),
                                  Operation::new(op::_D6, "SUB A, 0x{0:X}", 2),
                                  Operation::new(op::_D7, "RST 0x10", 1),
                                  Operation::new(op::_D8, "RET C", 1),
                                  Operation::new(op::_D9, "RETI", 1),
                                  Operation::new(op::_DA, "JP C, 0x{1:X}{0:X}", 3),
                                  Operation::empty(),
                                  Operation::new(op::_DC, "CALL C, 0x{1:X}{0:X}", 3),
                                  Operation::empty(),
                                  Operation::new(op::_DE, "SBC A, 0x{0:X}", 2),
                                  Operation::new(op::_DF, "RST 0x18", 1),

                                  Operation::new(op::_E0, "LD (0xFF00 + 0x{0:X}), A", 2),
                                  Operation::new(op::_E1, "POP HL", 1),
                                  Operation::new(op::_E2, "LD (C), A", 1),
                                  Operation::empty(),
                                  Operation::empty(),
                                  Operation::new(op::_E5, "PUSH HL", 1),
                                  Operation::new(op::_E6, "AND A, 0x{0:X}", 2),
                                  Operation::new(op::_E7, "RST 0x20", 1),
                                  Operation::new(op::_E8, "ADD SP, {0:X}", 2),
                                  Operation::new(op::_E9, "JP HL", 1),
                                  Operation::new(op::_EA, "LD (0x{1:X}{0:X}), A", 3),
                                  Operation::empty(),
                                  Operation::empty(),
                                  Operation::empty(),
                                  Operation::new(op::_EE, "XOR A, 0x{0:X}", 2),
                                  Operation::new(op::_EF, "RST 0x28", 1),

                                  Operation::new(op::_F0, "LD A, (0xFF00 + 0x{0:X})", 2),
                                  Operation::new(op::_F1, "POP AF", 1),
                                  Operation::new(op::_F2, "LD A, (C)", 3),
                                  Operation::new(op::_F3, "DI", 1),
                                  Operation::empty(),
                                  Operation::new(op::_F5, "PUSH AF", 1),
                                  Operation::new(op::_F6, "OR A, 0x{0:X}", 2),
                                  Operation::new(op::_F7, "RST 0x30", 1),
                                  Operation::new(op::_F8, "LD HL, SP + 0x{0:X}", 2),
                                  Operation::new(op::_F9, "LD SP, HL", 1),
                                  Operation::new(op::_FA, "LD A, (0x{1:X}{0:X})", 3),
                                  Operation::new(op::_FB, "EI", 1),
                                  Operation::empty(),
                                  Operation::empty(),
                                  Operation::new(op::_FE, "CP A, 0x{0:X}", 2),
                                  Operation::new(op::_FF, "RST 0x38", 1)];

        // DEBUG: Fill operations table with empty operations
        while operations.len() < 0x200 {
            operations.push(Operation::empty());
        }

        Table { operations: operations }
    }
}

impl Index<usize> for Table {
    type Output = Operation;

    fn index(&self, index: usize) -> &Operation {
        &self.operations[index]
    }
}

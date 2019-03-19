use crate::instruction::{
    Condition, DataProcessingOperation, Instruction, Operand2, Register, Shift, ShiftType, Unit,
};
use bitintr::{Bextr, Pext};
use byteorder::{ByteOrder, BE, LE};

#[inline]
fn decode_data_proc_or_psr(cond: Condition, opcode: u32) -> Instruction {
    let set_cond = opcode.bextr(20, 1) == 1;
    let immediate = opcode.bextr(25, 1) == 1;

    if opcode.bextr(23, 2) == 0b10 && !set_cond {
        let current = opcode.bextr(22, 1) == 0;
        if opcode.bextr(21, 1) == 1 {
            Instruction::StatusTransferFrom {
                cond,
                current,
                write_flags: opcode.bextr(19, 1) == 1,
                write_control: opcode.bextr(16, 1) == 1,
                o: Operand2::decode(opcode, immediate),
            }
        } else {
            Instruction::StatusTransferTo {
                cond,
                current,
                d: Register::decode(opcode.bextr(12, 4)),
            }
        }
    } else {
        Instruction::DataProcessing {
            cond,
            set_cond,
            op: DataProcessingOperation::decode(opcode.bextr(21, 4)),
            d: Register::decode(opcode.bextr(12, 4)),
            n: Register::decode(opcode.bextr(16, 4)),
            o: Operand2::decode(opcode, immediate),
        }
    }
}

pub fn decode(opcode: u32) -> Instruction {
    let cond = Condition::decode(opcode.bextr(28, 4));
    match opcode.bextr(25, 3) {
        0b000 => {
            if opcode.bextr(8, 20) == 0b0001_0010_1111_1111_1111 {
                Instruction::BranchExchange {
                    cond,
                    n: Register::decode(opcode.bextr(0, 4)),
                }
            } else if opcode.bextr(4, 4) == 0b1001 {
                match opcode.bextr(23, 2) {
                    0b00 => Instruction::Multiply {
                        cond,
                        set_cond: opcode.bextr(20, 1) == 1,
                        accumulate: opcode.bextr(21, 1) == 1,
                        d: Register::decode(opcode.bextr(16, 4)),
                        n: Register::decode(opcode.bextr(12, 4)),
                        s: Register::decode(opcode.bextr(8, 4)),
                        m: Register::decode(opcode.bextr(0, 4)),
                    },

                    0b01 => Instruction::MultiplyLong {
                        cond,
                        set_cond: opcode.bextr(20, 1) == 1,
                        accumulate: opcode.bextr(21, 1) == 1,
                        signed: opcode.bextr(22, 1) == 1,
                        dh: Register::decode(opcode.bextr(16, 4)),
                        dl: Register::decode(opcode.bextr(12, 4)),
                        s: Register::decode(opcode.bextr(8, 4)),
                        m: Register::decode(opcode.bextr(0, 4)),
                    },

                    0b10 => Instruction::SingleDataSwap {
                        cond,
                        unit: if opcode.bextr(22, 1) == 1 {
                            Unit::Byte
                        } else {
                            Unit::Word
                        },
                        d: Register::decode(opcode.bextr(16, 4)),
                        n: Register::decode(opcode.bextr(12, 4)),
                        m: Register::decode(opcode.bextr(0, 4)),
                    },

                    _ => unreachable!(),
                }
            } else if opcode.pext(0b1001_0000) == 0b11 {
                let offset = if opcode.bextr(25, 1) == 1 {
                    Operand2::Immediate {
                        rotate: 0,
                        value: opcode.pext(0b1111_0000_1111) as u16,
                    }
                } else {
                    Operand2::Register {
                        shift: Shift::Immediate {
                            type_: ShiftType::LogicalLeft,
                            amount: 0,
                        },
                        m: Register::decode(opcode.bextr(0, 4)),
                    }
                };

                Instruction::SingleDataTransfer {
                    cond,
                    pre_post_indexing: opcode.bextr(24, 1) == 1,
                    up_down: opcode.bextr(23, 1) == 1,
                    signed: opcode.bextr(6, 1) == 1,
                    store_load: opcode.bextr(20, 1) == 1,
                    write_back: opcode.bextr(21, 1) == 1,
                    unit: if opcode.bextr(5, 1) == 1 {
                        Unit::HalfWord
                    } else {
                        Unit::Byte
                    },
                    n: Register::decode(opcode.bextr(16, 4)),
                    d: Register::decode(opcode.bextr(12, 4)),
                    offset,
                }
            } else {
                decode_data_proc_or_psr(cond, opcode)
            }
        }

        0b001 => decode_data_proc_or_psr(cond, opcode),

        0b011 if opcode.bextr(4, 1) == 1 => {
            // |_Cond__|0_1_1|________________xxx____________________|1|__xxx__| Undefined
            unimplemented!("Undefined")
        }

        0b011 | 0b010 => {
            let offset = if opcode.bextr(25, 1) == 0 {
                Operand2::Immediate {
                    rotate: 0,
                    value: opcode.bextr(0, 12) as u16,
                }
            } else {
                Operand2::Register {
                    m: Register::decode(opcode.bextr(0, 4)),
                    shift: Shift::Immediate {
                        type_: ShiftType::decode(opcode.bextr(5, 2)),
                        amount: opcode.bextr(7, 12) as u8,
                    },
                }
            };

            Instruction::SingleDataTransfer {
                cond,
                pre_post_indexing: opcode.bextr(24, 1) == 1,
                up_down: opcode.bextr(23, 1) == 1,
                unit: if opcode.bextr(22, 1) == 1 {
                    Unit::Byte
                } else {
                    Unit::Word
                },
                signed: false,
                write_back: opcode.bextr(21, 1) == 1,
                store_load: opcode.bextr(20, 1) == 1,
                n: Register::decode(opcode.bextr(16, 4)),
                d: Register::decode(opcode.bextr(12, 4)),
                offset,
            }
        }

        0b100 => {
            // |_Cond__|1_0_0|P|U|S|W|L|__Rn___|__________Register_List________| BlockTrans
            unimplemented!("BlockTrans")
        }

        0b101 => {
            let mut offset = opcode.bextr(0, 24) << 2;
            if offset.bextr(25, 1) == 1 {
                // sign-extend to 32-bits if needed
                offset |= 0xfc_00_00_00;
            }

            Instruction::Branch {
                cond,
                link: opcode.bextr(24, 1) == 1,
                offset,
            }
        }

        0b111 => {
            // |_Cond__|1_1_1_1|_____________Ignored_by_Processor______________| SWI
            unimplemented!("SWI")
        }

        _ => unimplemented!("unknown: 0x{:x}", opcode),
    }
}

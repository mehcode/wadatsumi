mod condition;
mod data_processing_operation;
mod operand2;
mod register;
mod shift;
mod unit;

pub use self::{
    condition::Condition,
    data_processing_operation::DataProcessingOperation,
    operand2::Operand2,
    register::Register,
    shift::{Shift, ShiftType},
    unit::Unit,
};
use bitintr::{Bextr, Pext};
use core::fmt::Write;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{
    convert::TryInto,
    fmt::{self, Display, Formatter},
};
use unchecked_unwrap::UncheckedUnwrap;

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Branch {
        cond: Condition,
        link: bool,
        offset: u32,
    },

    BranchExchange {
        cond: Condition,
        n: Register,
    },

    Multiply {
        cond: Condition,
        set_cond: bool,
        accumulate: bool,
        d: Register,
        n: Register,
        s: Register,
        m: Register,
    },

    MultiplyLong {
        cond: Condition,
        set_cond: bool,
        signed: bool,
        accumulate: bool,
        dh: Register,
        dl: Register,
        s: Register,
        m: Register,
    },

    SingleDataSwap {
        cond: Condition,
        unit: Unit,
        d: Register,
        n: Register,
        m: Register,
    },

    SingleDataTransfer {
        cond: Condition,
        unit: Unit,
        signed: bool,
        write_back: bool,
        pre_post_indexing: bool,
        store_load: bool,
        up_down: bool,
        offset: Operand2,
        d: Register,
        n: Register,
    },

    DataProcessing {
        cond: Condition,
        set_cond: bool,
        op: DataProcessingOperation,
        n: Register,
        d: Register,
        o: Operand2,
    },

    /// Transfer xPSR contents to a register.
    StatusTransferTo {
        cond: Condition,
        /// True to transfer the _current_ status register (`CPSR`); false, to transfer the
        /// _saved_ status register (`SPSR`) for the current mode.
        current: bool,

        /// Destination register.
        d: Register,
    },

    /// Transfer register contents or immediate value to xPSR.
    StatusTransferFrom {
        cond: Condition,
        /// True to transfer the _current_ status register (`CPSR`); false, to transfer the
        /// _saved_ status register (`SPSR`) for the current mode.
        current: bool,

        write_flags: bool,
        write_control: bool,

        /// Source operand (may be un-shifted register or rotated immediate).
        o: Operand2,
    },

    SoftwareInterrupt {
        cond: Condition,
        comment: u32,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Instruction::BranchExchange { cond, n } => write!(f, "BX{} {}", cond, n),

            Instruction::Branch { cond, link, offset } => {
                let link_s = if *link { "L" } else { "" };
                write!(f, "B{}{} #{}", link_s, cond, (*offset as i32))
            }

            Instruction::DataProcessing {
                cond,
                op,
                d,
                n,
                o,
                set_cond,
            } => {
                let set_cond_s = if *set_cond { "S" } else { "" };
                write!(f, "{}{}{}", op, cond, set_cond_s)?;

                match op {
                    DataProcessingOperation::Compare
                    | DataProcessingOperation::CompareNegative
                    | DataProcessingOperation::TestBits
                    | DataProcessingOperation::TestBitwiseEquality => write!(f, " {}, {}", n, o),

                    DataProcessingOperation::Move | DataProcessingOperation::MoveNegative => {
                        write!(f, " {}, {}", d, o)
                    }

                    _ => write!(f, " {}, {}, {}", d, n, o),
                }
            }

            Instruction::StatusTransferTo { cond, current, d } => {
                let current_s = if *current { "C" } else { "S" };
                write!(f, "MRS{} {}, {}PSR", cond, d, current_s)
            }

            Instruction::StatusTransferFrom {
                cond,
                current,
                o,
                write_flags,
                write_control,
            } => {
                let current_s = if *current { "C" } else { "S" };
                let mask_s = if *write_flags && *write_control {
                    ""
                } else if *write_flags {
                    "_flg"
                } else {
                    "_ctl"
                };

                write!(f, "MSR{} {}PSR{}, {}", cond, current_s, mask_s, o)
            }

            Instruction::SingleDataTransfer {
                cond,
                unit,
                signed,
                write_back,
                pre_post_indexing,
                store_load,
                up_down,
                offset,
                d,
                n,
            } => {
                let op = if *store_load { "LDR" } else { "STR" };
                let signed_s = if *signed { "S" } else { "" };
                let up_down_s = if *up_down { "+" } else { "-" };

                write!(f, "{}{}{}{}", cond, op, signed_s, unit)?;

                if !*pre_post_indexing && *write_back {
                    f.write_char('T')?;
                }

                write!(f, " {}, ", d);

                if *pre_post_indexing {
                    write!(f, "[{}, {}{}]", n, up_down_s, offset)?;

                    if *write_back {
                        f.write_char('!')?;
                    }
                } else {
                    write!(f, "[{}], {}{}", n, up_down_s, offset)?;
                }

                Ok(())
            }

            _ => unimplemented!("{:?}", self),
        }
    }
}

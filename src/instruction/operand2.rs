use super::{Register, Shift};
use bitintr::Bextr;
use std::fmt::{self, Display, Formatter};

// todo: better name than Operand2?
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Operand2 {
    Immediate { rotate: u8, value: u16 },
    Register { shift: Shift, m: Register },
}

impl Operand2 {
    pub fn decode(code: u32, immediate: bool) -> Self {
        if immediate {
            Operand2::Immediate {
                rotate: code.bextr(8, 4) as u8,
                value: (code & 0xff) as u16,
            }
        } else {
            Operand2::Register {
                shift: Shift::decode(code.bextr(4, 8)),
                m: Register::decode(code.bextr(0, 4)),
            }
        }
    }
}

impl Display for Operand2 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Operand2::Immediate { rotate, value } => write!(
                f,
                "#0x{:x}",
                (*value as u32).rotate_right((*rotate as u32) * 2)
            ),

            Operand2::Register { shift, m } => write!(f, "{}{}", m, shift),
        }
    }
}

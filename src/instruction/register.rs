use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::{self, Display, Formatter};
use unchecked_unwrap::UncheckedUnwrap;

/// A reference to a register in an instruction.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    #[inline]
    pub fn decode(code: u32) -> Self {
        unsafe { Register::from_u32(code).unchecked_unwrap() }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "R{}", *self as u8)
    }
}

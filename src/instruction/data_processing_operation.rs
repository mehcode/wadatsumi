use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::{self, Display, Formatter};
use unchecked_unwrap::UncheckedUnwrap;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum DataProcessingOperation {
    And = 0b0000,
    ExclusiveOr = 0b0001,
    Subtract = 0b0010,
    ReverseSubtract = 0b0011,
    Add = 0b0100,
    AddWithCarry = 0b0101,
    SubtractWithCarry = 0b0110,
    ReverseSubtractWithCarry = 0b0111,
    TestBits = 0b1000,
    TestBitwiseEquality = 0b1001,
    Compare = 0b1010,
    CompareNegative = 0b1011,
    Or = 0b1100,
    Move = 0b1101,
    BitClear = 0b1110,
    MoveNegative = 0b1111,
}

impl Display for DataProcessingOperation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            DataProcessingOperation::And => "AND",
            DataProcessingOperation::ExclusiveOr => "EOR",
            DataProcessingOperation::Subtract => "SUB",
            DataProcessingOperation::ReverseSubtract => "RSB",
            DataProcessingOperation::Add => "ADD",
            DataProcessingOperation::AddWithCarry => "ADC",
            DataProcessingOperation::SubtractWithCarry => "SBC",
            DataProcessingOperation::ReverseSubtractWithCarry => "RSC",
            DataProcessingOperation::TestBits => "TST",
            DataProcessingOperation::TestBitwiseEquality => "TEQ",
            DataProcessingOperation::Compare => "CMP",
            DataProcessingOperation::CompareNegative => "CMN",
            DataProcessingOperation::Or => "ORR",
            DataProcessingOperation::Move => "MOV",
            DataProcessingOperation::BitClear => "BIC",
            DataProcessingOperation::MoveNegative => "MVN",
        })
    }
}

impl DataProcessingOperation {
    #[inline]
    pub fn decode(code: u32) -> Self {
        unsafe { DataProcessingOperation::from_u32(code).unchecked_unwrap() }
    }
}

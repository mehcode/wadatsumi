use std::fmt::{self, Display, Formatter, Write};

/// Unit of data being swapped or transferred from a memory address.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum Unit {
    Byte,
    HalfWord,
    Word,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Unit::Byte => f.write_char('B'),
            Unit::HalfWord => f.write_char('H'),

            // Word quantities are the default when no size suffix is specified
            _ => Ok(()),
        }
    }
}

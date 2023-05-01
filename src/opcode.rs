use std::fmt::{self, Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct Opcode {
    pub hi: u8,
    pub lo: u8,
}

impl Opcode {
    pub fn fetch(ram: &[u8], pc: &mut usize) -> Self {
        let hi = ram[*pc];
        let lo = ram[*pc + 1];

        *pc += 2;

        Self { hi, lo }
    }

    /// Get the 4 nibbles of this opcode as 4 separate `u8` values.
    pub fn digits(self) -> (u8, u8, u8, u8) {
        (self.hi >> 4, self.hi & 0xf, self.lo >> 4, self.lo & 0xf)
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}{:02X}", self.hi, self.lo)
    }
}

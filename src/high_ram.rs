use super::bus::Bus;

pub struct HighRam {
    ram: Box<[u8; 127]>,
}

impl HighRam {
    pub fn new() -> Self {
        Self {
            ram: box [0; 127],
        }
    }
}

impl Bus for HighRam {
    #[inline]
    fn contains(&self, address: u16) -> bool {
        (0xff80...0xfffe).contains(address)
    }

    fn read8(&self, address: u16) -> u8 {
        self.ram[(address as usize) - 0xff80]
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.ram[(address as usize) - 0xff80] = value;
    }
}

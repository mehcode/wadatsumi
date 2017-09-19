use super::bus::Bus;

pub struct WorkRam {
    // 0xC000...0xCFFF | 4 KiB Work RAM Bank 0
    // 0xD000...0xDFFF | 4 KiB Work RAM Bank 1
    // 0xE000...0xFDFF | Mirror of 0xC000 ... 0xDDFF
    /// 8 KiB of Work RAM
    ram: Box<[u8]>,
}

impl WorkRam {
    pub fn new() -> Self {
        // TODO: 32 KiB in CGB mode (switchable bank 1)

        Self {
            ram: vec![0; 0x2000].into_boxed_slice(),
        }
    }
}

impl Bus for WorkRam {
    #[inline]
    fn contains(&self, address: u16) -> bool {
        (0xC000...0xFDFF).contains(address)
    }

    fn read8(&self, address: u16) -> u8 {
        self.ram[(address as usize & 0x1FFF)]
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.ram[(address as usize & 0x1FFF)] = value;
    }
}

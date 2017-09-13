pub trait Bus {
    fn read8(&self, address: u16) -> u8;
    fn write8(&mut self, address: u16, value: u8);

    #[inline]
    fn read16(&self, address: u16) -> u16 {
        let l = self.read8(address);
        let h = self.read8(address.wrapping_add(1));

        ((h as u16) << 8) | (l as u16)
    }

    #[inline]
    fn write16(&mut self, address: u16, value: u16) {
        self.write8(address.wrapping_add(1), (value >> 8) as u8);
        self.write8(address, value as u8);
    }
}

pub trait Bus {
    fn read8(&self, address: u16) -> u8;
    fn write8(&mut self, address: u16, value: u8);
}

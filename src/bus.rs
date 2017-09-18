pub trait Bus {
    fn contains(&self, address: u16) -> bool;

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

impl<T, U> Bus for (T, U)
where
    T: Bus,
    U: Bus,
{
    #[inline]
    fn contains(&self, address: u16) -> bool {
        self.0.contains(address) || self.1.contains(address)
    }

    #[inline]
    fn read8(&self, address: u16) -> u8 {
        if self.0.contains(address) {
            self.0.read8(address)
        } else if self.1.contains(address) {
            self.1.read8(address)
        } else {
            if address == 0xFF01 || address == 0xFF00 {
                warn!("unhandled read: {:04x}", address);
            }

            0xff
        }
    }

    #[inline]
    fn write8(&mut self, address: u16, value: u8) {
        if self.0.contains(address) {
            self.0.write8(address, value);
        } else if self.1.contains(address) {
            self.1.write8(address, value);
        } else {
            if address == 0xFF01 || address == 0xFF00 {
                warn!("unhandled write: {:04x} <- {:02x}", address, value);
            }
        }
    }
}

/// Describes a system "bus" that is intended to be implemented on its corresponding component.
///
/// A tuple of `Bus` can be used as an "Interconnect".
pub trait Bus {
    /// Returns `true` if this instance can handle the given `address`.
    #[inline]
    fn contains(&self, _: u16) -> bool {
        false
    }

    /// Returns the 8-bit value corresponding to the `address`.
    #[inline]
    fn read8(&self, _: u16) -> u8 {
        // Return an "unconnected" value
        0xff
    }

    /// Writes the 8-bit value to the given `address`.
    #[inline]
    fn write8(&mut self, _: u16, _: u8) {
        // Do nothing
    }

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

macro_rules! impl_bus_tuple {
    ( $($name:ident)+) => (
        impl<$($name: Bus),*> Bus for ($($name,)*) {
            #[allow(non_snake_case)]
            #[inline]
            fn contains(&self, address: u16) -> bool {
                let ($(ref $name,)*) = *self;
                $(if $name.contains(address) { return true; })*

                false
            }

            #[allow(non_snake_case)]
            #[inline]
            fn read8(&self, address: u16) -> u8 {
                let ($(ref $name,)*) = *self;
                $(if $name.contains(address) { return $name.read8(address); })*

                warn!("unhandled read: {:04x}", address);

                0xff
            }

            #[allow(non_snake_case)]
            #[inline]
            fn write8(&mut self, address: u16, value: u8) {
                let ($(ref mut $name,)*) = *self;
                $(if $name.contains(address) { return $name.write8(address, value); })*

                warn!("unhandled write: {:04x} <- {:02x}", address, value);
            }
        }
    );
}

impl_bus_tuple!{ A }
impl_bus_tuple!{ A B }

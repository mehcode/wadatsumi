use ::bits;

#[derive(Default)]
pub struct Channel3 {
    /// Sound On/Off
    pub enable: bool,

    /// Sound Length
    pub length: u8,

    /// Counter / Consecutive selection (Length Enable)
    pub length_enable: bool,

    /// Volume
    pub volume: u8,

    /// Frequency - 11-bits
    pub frequency: u16,
}

impl Channel3 {
    pub fn reset(&mut self) {
        self.enable = false;

        self.length = 0;
        self.length_enable = false;

        self.volume = 0;
        self.frequency = 0;
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // Channel 3 Sound On/Off
            // [E--- ----] DAC Power
            0xFF1A => bits::bit(self.enable, 7) | 0x7F,

            // Channel 3 Volume
            // [-VV- ----] Volume
            0xFF1C => (self.volume << 5) | 0x9F,

            // Channel 3 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF1E => bits::bit(self.length_enable, 6) | 0xBF,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // Channel 3 Sound On/Off
            // [E--- ----] DAC Power
            0xFF1A => {
                self.enable = bits::test(value, 7);
            }

            // Channel 3 Sound Length
            // [LLLL LLLL] Length load (256-L)
            0xFF1B => {
                self.length = (256 as u16 - value as u16) as u8;
            }

            // Channel 3 Volume
            // [-VV- ----] Volume
            0xFF1C => {
                self.volume = (value & 0b11) << 5;
            }

            // Channel 2 Frequency (lo)
            // [FFFF FFFF] Frequency LSB
            0xFF1D => {
                self.frequency &= !0xFF;
                self.frequency |= value as u16;
            }

            // Channel 2 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF1E => {
                self.frequency &= !0x700;
                self.frequency |= ((value & 0b111) as u16) << 8;

                self.length_enable = bits::test(value, 6);
            }

            _ => {}
        }
    }
}

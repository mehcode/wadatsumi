use ::bits;

#[derive(Default)]
pub struct Channel4 {
    /// Sound Length
    pub length: u8,

    /// Counter / Consecutive selection (Length Enable)
    pub length_enable: bool,

    /// Initial Volume of envelope
    pub volume_envl_initial: u8,

    /// Volume Envelope Direction (0=decrease, 1=increase)
    pub volume_envl_direction: bool,

    /// Volume Envelope Period
    ///     A period of 0 is treated as 8.
    pub volume_envl_period: u8,

    /// Shift Clock Frequency - 4-bits
    pub shift: u8,

    /// Counter Step/Width (0=15 bits, 1=7 bits)
    pub width: bool,

    /// Dividing Ratio of Frequencies
    pub divisor: u8,
}

impl Channel4 {
    pub fn reset(&mut self) {
        self.length = 0;
        self.length_enable = false;

        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;

        self.shift = 0;
        self.width = false;
        self.divisor = 0;
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // Channel 4 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF21 => {
                (self.volume_envl_initial << 4) | bits::bit(self.volume_envl_direction, 3) |
                self.volume_envl_period
            }

            // Channel 4 Polynomial Counter
            // [SSSS WDDD] Clock shift, Width mode of LFSR, Divisor code
            0xFF22 => (self.shift << 4) | bits::bit(self.width, 3) | self.divisor,

            // Channel 2 Misc.
            // [TL-- ----] Trigger, Length enable
            0xFF23 => bits::bit(self.length_enable, 6) | 0xBF,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // Channel 4 Sound Length
            // [--LL LLLL] Length load (64-L)
            0xFF20 => {
                self.length = 64 - (value & 0b11_1111);
            }

            // Channel 4 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF21 => {
                self.volume_envl_initial = (value >> 4) & 0b1111;
                self.volume_envl_direction = bits::test(value, 3);
                self.volume_envl_period = value & 0b111;
            }

            // Channel 4 Polynomial Counter
            // [SSSS WDDD] Clock shift, Width mode of LFSR, Divisor code
            0xFF22 => {
                self.shift = (value >> 4) & 0b1111;
                self.width = bits::test(value, 3);
                self.divisor = value & 0b111;
            }

            // Channel 4 Misc.
            // [TL-- ----] Trigger, Length enable
            0xFF23 => {
                self.length_enable = bits::test(value, 6);
            }

            _ => {}
        }
    }
}

use ::bits;

#[derive(Default)]
pub struct Channel2 {
    /// Enable
    pub enable: bool,

    /// Wave Pattern Duty
    ///     0      00000001    12.5%
    ///     1      10000001    25%
    ///     2      10000111    50%
    ///     3      01111110    75%
    pub wave_pattern_duty: u8,

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

    /// Frequency - 11-bits
    pub frequency: u16,
}

impl Channel2 {
    pub fn is_enabled(&self) -> bool {
        self.enable && (!self.length_enable || self.length > 0)
    }

    pub fn reset(&mut self) {
        self.enable = false;

        self.wave_pattern_duty = 0;

        self.length = 0;
        self.length_enable = false;

        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;

        self.frequency = 0;
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // Channel 2 Sound Length/Wave Pattern Duty
            // [DDLL LLLL] Duty, Length load (64-L)
            0xFF16 => (self.wave_pattern_duty << 6) | 0x3F,

            // Channel 2 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF17 => {
                (self.volume_envl_initial << 4) | bits::bit(self.volume_envl_direction, 3) |
                self.volume_envl_period
            }

            // Channel 2 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF19 => bits::bit(self.length_enable, 6) | 0xBF,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // Channel 2 Sound Length/Wave Pattern Duty
            // [DDLL LLLL] Duty, Length load (64-L)
            0xFF16 => {
                self.wave_pattern_duty = (value >> 6) & 0b11;
                self.length = 64 - (value & 0b11_1111);
            }

            // Channel 2 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF17 => {
                self.volume_envl_initial = (value >> 4) & 0b1111;
                self.volume_envl_direction = bits::test(value, 3);
                self.volume_envl_period = value & 0b111;
            }

            // Channel 2 Frequency (lo)
            // [FFFF FFFF] Frequency LSB
            0xFF18 => {
                self.frequency &= !0xFF;
                self.frequency |= value as u16;
            }

            // Channel 2 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF19 => {
                self.frequency &= !0x700;
                self.frequency |= ((value & 0b111) as u16) << 8;

                self.length_enable = bits::test(value, 6);
            }

            _ => {}
        }
    }
}

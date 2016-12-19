use ::bits;

#[derive(Default)]
pub struct Channel1 {
    /// Enable
    pub enable: bool,

    /// Sweep Period
    pub sweep_period: u8,

    /// Sweep Direction
    pub sweep_direction: bool,

    /// Sweep Shift
    pub sweep_shift: u8,

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

impl Channel1 {
    pub fn is_enabled(&self) -> bool {
        self.enable && (self.volume_envl_initial > 0 || self.volume_envl_direction)
    }

    pub fn clear(&mut self) {
        self.enable = false;

        self.sweep_period = 0;
        self.sweep_direction = false;
        self.sweep_shift = 0;

        self.wave_pattern_duty = 0;

        self.length = 0;
        self.length_enable = false;

        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;

        self.frequency = 0;
    }

    pub fn reset(&mut self) {
        self.clear();
    }

    pub fn trigger(&mut self, frame_seq_step: u8) {
        // Channel is enabled
        self.enable = true;

        // If length counter is zero; set to max
        if self.length == 0 {
            self.length = if self.length_enable && (frame_seq_step % 2 == 1) {
                // If the length counter is being unfrozen when the frame sequencer's next
                // step would not clock the length counter (and its enabled); clock it
                63
            } else {
                64
            };
        }

        // TODO: Frequency timer is reloaded with period
        // TODO: Volume envelope timer is reloaded with period

        // Sweep
        // TODO: Square 1's frequency is copied to the shadow register.
        // TODO: The sweep timer is reloaded.
        // TODO: The internal enabled flag is set if either the sweep period or shift are non-zero, cleared otherwise.
        // TODO: If the sweep shift is non-zero, frequency calculation and the overflow check are performed immediately.
    }

    pub fn step_length(&mut self) {
        if self.length_enable && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enable = false;
            }
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // Channel 1 Sweep
            // [-PPP NSSS] Sweep period, negate, shift
            0xFF10 => {
                (self.sweep_period << 4) | bits::bit(self.sweep_direction, 3) | self.sweep_shift |
                0x80
            }

            // Channel 1 Sound Length/Wave Pattern Duty
            // [DDLL LLLL] Duty, Length load (64-L)
            0xFF11 => (self.wave_pattern_duty << 6) | 0x3F,

            // Channel 1 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF12 => {
                (self.volume_envl_initial << 4) | bits::bit(self.volume_envl_direction, 3) |
                self.volume_envl_period
            }

            // Channel 1 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF14 => bits::bit(self.length_enable, 6) | 0xBF,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8, frame_seq_step: u8) {
        match address {
            // Channel 1 Sweep
            // [-PPP NSSS] Sweep period, negate, shift
            0xFF10 => {
                self.sweep_period = (value >> 4) & 0b111;
                self.sweep_direction = bits::test(value, 3);
                self.sweep_shift = value & 0b111;
            }

            // Channel 1 Sound Length/Wave Pattern Duty
            // [DDLL LLLL] Duty, Length load (64-L)
            0xFF11 => {
                self.wave_pattern_duty = (value >> 6) & 0b11;
                self.length = 64 - (value & 0b11_1111);
            }

            // Channel 1 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF12 => {
                self.volume_envl_initial = (value >> 4) & 0b1111;
                self.volume_envl_direction = bits::test(value, 3);
                self.volume_envl_period = value & 0b111;

                // Setting the volume envelope to 0 with a decrease direction will disable
                // the channel
                if self.volume_envl_initial == 0 && !self.volume_envl_direction {
                    self.enable = false;
                }
            }

            // Channel 1 Frequency (lo)
            // [FFFF FFFF] Frequency LSB
            0xFF13 => {
                self.frequency &= !0xFF;
                self.frequency |= value as u16;
            }

            // Channel 1 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF14 => {
                self.frequency &= !0x700;
                self.frequency |= ((value & 0b111) as u16) << 8;

                let prev_length_enable = self.length_enable;
                self.length_enable = bits::test(value, 6);

                // Enabling the length counter when the next step of the frame sequencer
                // would not clock the length counter; should clock the length counter
                if !prev_length_enable && self.length_enable && (frame_seq_step % 2 == 1) {
                    if self.length > 0 {
                        self.length -= 1;
                    }
                }

                if bits::test(value, 7) {
                    self.trigger(frame_seq_step);
                } else if self.length == 0 {
                    // If the extra length clock brought our length to 0 and we weren't triggered;
                    // disable
                    self.enable = false;
                }
            }

            _ => {}
        }
    }
}

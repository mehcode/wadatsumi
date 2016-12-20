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

    /// Current index into the wave pattern
    pub wave_pattern_index: u8,

    /// Sound Length
    pub length: u8,

    /// Counter / Consecutive selection (Length Enable)
    pub length_enable: bool,

    /// Current volume
    pub volume: u8,

    /// Volume envelope timer
    pub volume_envl_timer: u8,

    /// Initial Volume of envelope
    pub volume_envl_initial: u8,

    /// Volume Envelope Direction (0=decrease, 1=increase)
    pub volume_envl_direction: bool,

    /// Volume Envelope Period
    ///     A period of 0 is treated as 8.
    pub volume_envl_period: u8,

    /// Frequency - 11-bits
    pub frequency: u16,

    /// Timer (Frequency)
    pub timer: u16,
}

impl Channel2 {
    pub fn is_enabled(&self) -> bool {
        self.enable && (!self.length_enable || self.length > 0) &&
        (self.volume_envl_initial > 0 || self.volume_envl_direction)
    }

    pub fn clear(&mut self) {
        self.enable = false;

        self.wave_pattern_duty = 0;
        self.wave_pattern_index = 0;

        self.length_enable = false;

        self.volume = 0;
        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;
        self.volume_envl_timer = 0;

        self.frequency = 0;
        self.timer = 0;
    }

    pub fn reset(&mut self) {
        self.clear();
        self.length = 0;
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

        // Frequency timer is reloaded with period
        self.timer = (2048 - self.frequency) * 4;

        // Volume envelope timer is reloaded with period
        self.volume = self.volume_envl_initial;
        self.volume_envl_timer = if self.volume_envl_period == 0 {
            8
        } else {
            self.volume_envl_period
        };
    }

    pub fn sample(&mut self) -> i16 {
        if !self.is_enabled() {
            return 0;
        }

        let pattern = if self.wave_pattern_duty == 0b11 {
            0b01111110
        } else if self.wave_pattern_duty == 0b10 {
            0b10000111
        } else if self.wave_pattern_duty == 0b01 {
            0b10000001
        } else {
            0b00000001
        };

        let bit = bits::test(pattern, (7 - self.wave_pattern_index));

        if bit { self.volume as i16 } else { 0 }
    }

    pub fn step(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.wave_pattern_index += 1;
            if self.wave_pattern_index == 8 {
                self.wave_pattern_index = 0;
            }

            self.timer = (2048 - self.frequency) * 4;
        }
    }

    pub fn step_length(&mut self) {
        if self.length_enable && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enable = false;
            }
        }
    }

    pub fn step_volume(&mut self) {
        if self.volume_envl_timer > 0 {
            self.volume_envl_timer -= 1;
        }

        if self.volume_envl_period > 0 && self.volume_envl_timer == 0 {
            if self.volume_envl_direction {
                if self.volume < 0xF {
                    self.volume += 1;
                }
            } else if self.volume > 0 {
                self.volume -= 1;
            }
        }

        if self.volume_envl_timer == 0 {
            self.volume_envl_timer = if self.volume_envl_period == 0 {
                8
            } else {
                self.volume_envl_period
            };
        }
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

    pub fn write(&mut self, address: u16, value: u8, frame_seq_step: u8, master_enable: bool) {
        match address {
            // Channel 2 Sound Length/Wave Pattern Duty
            // [DDLL LLLL] Duty, Length load (64-L)
            0xFF16 => {
                if master_enable {
                    self.wave_pattern_duty = (value >> 6) & 0b11;
                }

                self.length = 64 - (value & 0b11_1111);
            }

            // Channel 2 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF17 if master_enable => {
                self.volume_envl_initial = (value >> 4) & 0b1111;
                self.volume_envl_direction = bits::test(value, 3);
                self.volume_envl_period = value & 0b111;

                // Setting the volume envelope to 0 with a decrease direction will disable
                // the channel
                if self.volume_envl_initial == 0 && !self.volume_envl_direction {
                    self.enable = false;
                }
            }

            // Channel 2 Frequency (lo)
            // [FFFF FFFF] Frequency LSB
            0xFF18 if master_enable => {
                self.frequency &= !0xFF;
                self.frequency |= value as u16;
            }

            // Channel 2 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF19 if master_enable => {
                self.frequency &= !0x700;
                self.frequency |= ((value & 0b111) as u16) << 8;

                let prev_length_enable = self.length_enable;
                self.length_enable = bits::test(value, 6);

                // Enabling the length counter when the next step of the frame sequencer
                // would not clock the length counter; should clock the length counter
                if !prev_length_enable && self.length_enable && (frame_seq_step % 2 == 1) &&
                   self.length > 0 {
                    self.length -= 1;
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

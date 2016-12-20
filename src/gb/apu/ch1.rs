use ::bits;

#[derive(Default)]
pub struct Channel1 {
    /// Enable
    pub enable: bool,

    /// Sweep Enabled
    pub sweep_enable: bool,

    /// Sweep Timer
    pub sweep_timer: u8,

    /// # of Sweep Calculations since Trigger
    pub sweep_negate_calcd: bool,

    /// Sweep Shadow Frequency
    pub frequency_sh: u16,

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

impl Channel1 {
    pub fn is_enabled(&self) -> bool {
        self.enable && (self.volume_envl_initial > 0 || self.volume_envl_direction)
    }

    pub fn clear(&mut self) {
        self.enable = false;

        self.sweep_enable = false;
        self.sweep_timer = 0;
        self.frequency_sh = 0;
        self.sweep_period = 0;
        self.sweep_direction = false;
        self.sweep_shift = 0;
        self.sweep_negate_calcd = false;

        self.wave_pattern_duty = 0;
        self.wave_pattern_index = 0;

        self.length_enable = false;

        self.volume = 0;
        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;
        self.volume_envl_timer = 0;

        self.frequency = 0;

        // When triggering a square channel, the low two bits of the
        // frequency timer are NOT modified.
        self.timer &= 3;
    }

    pub fn reset(&mut self) {
        self.clear();

        self.enable = true;

        self.length = 0;

        self.wave_pattern_duty = 0b10;

        self.volume_envl_period = 0b11;
        self.volume_envl_initial = 0xF;

        self.timer = 0;
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

        // If a channel is triggered when the frame sequencer's next
        // step will clock the volume envelope, the envelope's timer is
        // reloaded with one greater than it would have been.
        if frame_seq_step == 7 {
            self.volume_envl_timer += 1;
        }

        // [Sweep] Square 1's frequency is copied to the shadow register.
        self.frequency_sh = self.frequency;

        // [Sweep] The sweep timer is reloaded.
        self.sweep_timer = if self.sweep_period == 0 {
            8
        } else {
            self.sweep_period
        };

        // The internal enabled flag is set if either the sweep period or shift are non-zero,
        // cleared otherwise.
        self.sweep_enable = self.sweep_period > 0 || self.sweep_shift > 0;
        self.sweep_negate_calcd = false;

        // If the sweep shift is non-zero, frequency calculation and the overflow check
        // are performed immediately.
        if self.sweep_enable && self.sweep_shift > 0 {
            self.calc_sweep();
        }
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

    pub fn step_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }

        if self.sweep_period > 0 && self.sweep_enable && self.sweep_timer == 0 {
            let freq = self.calc_sweep();
            if freq <= 2047 && self.sweep_shift > 0 {
                self.frequency_sh = freq;
                self.frequency = freq;

                self.calc_sweep();
            }
        }

        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period == 0 {
                8
            } else {
                self.sweep_period
            };
        }
    }

    pub fn calc_sweep(&mut self) -> u16 {
        // Calculate new frequency using sweep
        let mut freq = self.frequency_sh;
        let r = self.frequency_sh >> self.sweep_shift;
        if self.sweep_direction {
            freq -= r;
        } else {
            freq += r;
        }

        // Disable channel if overflow
        if freq > 2047 {
            self.enable = false;
            self.sweep_enable = false;
        }

        if self.sweep_direction {
            self.sweep_negate_calcd = true;
        }

        freq
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

    pub fn write(&mut self, address: u16, value: u8, frame_seq_step: u8, master_enable: bool) {
        match address {
            // Channel 1 Sweep
            // [-PPP NSSS] Sweep period, negate, shift
            0xFF10 if master_enable => {
                self.sweep_period = (value >> 4) & 0b111;
                self.sweep_shift = value & 0b111;

                // Clearing the sweep negate mode bit in NR10 after at least one
                // sweep calculation has been made using the negate mode since
                // the last trigger causes the channel to be immediately disabled.
                if self.sweep_direction && !bits::test(value, 3) && self.sweep_negate_calcd {
                    self.enable = false;
                    self.sweep_enable = false;
                }

                self.sweep_direction = bits::test(value, 3);
            }

            // Channel 1 Sound Length/Wave Pattern Duty
            // [DDLL LLLL] Duty, Length load (64-L)
            0xFF11 => {
                if master_enable {
                    self.wave_pattern_duty = (value >> 6) & 0b11;
                }
                self.length = 64 - (value & 0b11_1111);
            }

            // Channel 1 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF12 if master_enable => {
                // If the old envelope period was zero and the envelope is
                // still doing automatic updates, volume is incremented by 1,
                // otherwise if the envelope was in subtract mode, volume is
                // incremented by 2.
                if self.volume_envl_period == 0 && (self.volume > 0 || self.volume < 0xF) {
                    self.volume += 1;
                    if self.volume_envl_direction {
                        self.volume += 1;
                    }

                    self.volume &= 0xF;
                }

                self.volume_envl_initial = (value >> 4) & 0b1111;

                // If the mode was changed (add to subtract or subtract to add),
                // volume is set to 16-volume.
                if self.volume_envl_direction != bits::test(value, 3) {
                    self.volume = 16 - self.volume;
                    self.volume &= 0xF;
                }

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
            0xFF13 if master_enable => {
                self.frequency &= !0xFF;
                self.frequency |= value as u16;
            }

            // Channel 1 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF14 if master_enable => {
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

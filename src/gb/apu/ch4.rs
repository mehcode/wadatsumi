use ::bits;

#[derive(Default)]
pub struct Channel4 {
    /// Enable
    pub enable: bool,

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

    /// Shift Clock Frequency - 4-bits
    pub shift: u8,

    /// Counter Step/Width (0=15 bits, 1=7 bits)
    pub width: bool,

    /// Dividing Ratio of Frequencies
    pub divisor: u8,

    /// Linear Feedback Shift Register (LFSR)
    pub lfsr: u16,

    /// Timer (Frequency)
    pub timer: u16,
}

fn get_divisor(index: u8) -> u16 {
    if index == 1 {
        16
    } else if index == 2 {
        32
    } else if index == 3 {
        48
    } else if index == 4 {
        64
    } else if index == 5 {
        80
    } else if index == 6 {
        96
    } else if index == 7 {
        112
    } else {
        8
    }
}

impl Channel4 {
    pub fn is_enabled(&self) -> bool {
        self.enable && (self.volume_envl_initial > 0 || self.volume_envl_direction)
    }

    pub fn clear(&mut self) {
        self.enable = false;

        self.length_enable = false;

        self.volume = 0;
        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;
        self.volume_envl_timer = 0;

        self.shift = 0;
        self.width = false;
        self.divisor = 0;
        self.lfsr = 0;
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
        self.timer = get_divisor(self.divisor) << self.shift;

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

        // Noise channel's LFSR bits are all set to 1.
        self.lfsr = 0x7FFF;
    }

    pub fn sample(&mut self) -> i16 {
        if !self.is_enabled() {
            return 0;
        }

        // The waveform output is bit 0 of the LFSR, INVERTED
        if (self.lfsr & 0x1) == 0 {
            self.volume as i16
        } else {
            0
        }
    }

    pub fn step(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;

            if self.timer == 0 {
                // Using a noise channel clock shift of 14 or 15 results in the
                // LFSR receiving no clocks.
                if self.shift < 14 {
                    // When clocked by the frequency timer, the low two bits (0 and 1)
                    // are XORed
                    let b = (self.lfsr & 0x1) ^ ((self.lfsr & 0x2) >> 1);

                    // All bits are shifted right by one
                    self.lfsr >>= 1;

                    // And the result of the XOR is put into the now-empty high bit
                    if b != 0 {
                        self.lfsr |= 0x4000;
                    }

                    // If width mode is 1 (NR43), the XOR result is ALSO put into
                    // bit 6 AFTER the shift, resulting in a 7-bit LFSR.
                    if self.width {
                        self.lfsr &= !0x40;
                        if b != 0 {
                            self.lfsr |= 0x40;
                        }
                    }
                }

                // Reload timer
                self.timer = get_divisor(self.divisor) << self.shift;
            }
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

    pub fn write(&mut self, address: u16, value: u8, frame_seq_step: u8, master_enable: bool) {
        match address {
            // Channel 4 Sound Length
            // [--LL LLLL] Length load (64-L)
            0xFF20 => {
                self.length = 64 - (value & 0b11_1111);
            }

            // Channel 4 Volume Envelope
            // [VVVV APPP] Starting volume, Envelope add mode, period
            0xFF21 if master_enable => {
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

            // Channel 4 Polynomial Counter
            // [SSSS WDDD] Clock shift, Width mode of LFSR, Divisor code
            0xFF22 if master_enable => {
                self.shift = (value >> 4) & 0b1111;
                self.width = bits::test(value, 3);
                self.divisor = value & 0b111;
            }

            // Channel 4 Misc.
            // [TL-- ----] Trigger, Length enable
            0xFF23 if master_enable => {
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

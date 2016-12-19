use ::bits;

#[derive(Default)]
pub struct Channel4 {
    /// Enable
    pub enable: bool,

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

    /// Linear Feedback Shift Register (LFSR)
    pub lfsr: u16,
}

impl Channel4 {
    pub fn is_enabled(&self) -> bool {
        self.enable && (!self.length_enable || self.length > 0) &&
        (self.volume_envl_initial > 0 || self.volume_envl_direction)
    }

    pub fn reset(&mut self) {
        self.enable = false;

        self.length = 0;
        self.length_enable = false;

        self.volume_envl_initial = 0;
        self.volume_envl_direction = false;
        self.volume_envl_period = 0;

        self.shift = 0;
        self.width = false;
        self.divisor = 0;
        self.lfsr = 0;
    }

    pub fn clear(&mut self) {
        self.reset();
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

        // Noise channel's LFSR bits are all set to 1.
        self.lfsr = 0xFFFF;
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

    pub fn write(&mut self, address: u16, value: u8, frame_seq_step: u8) {
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

                // Setting the volume envelope to 0 with a decrease direction will disable
                // the channel
                if self.volume_envl_initial == 0 && !self.volume_envl_direction {
                    self.enable = false;
                }
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

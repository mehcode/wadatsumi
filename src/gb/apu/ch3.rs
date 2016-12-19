use std::vec::Vec;
use ::bits;

#[derive(Default)]
pub struct Channel3 {
    /// Enable
    pub enable: bool,

    /// Sound On/Off
    pub dac_enable: bool,

    /// Sound Length
    pub length: u16,

    /// Counter / Consecutive selection (Length Enable)
    pub length_enable: bool,

    /// Volume
    pub volume: u8,

    /// Frequency - 11-bits
    pub frequency: u16,

    /// Wave RAM
    pub wave_ram: Vec<u8>,
}

impl Channel3 {
    pub fn is_enabled(&self) -> bool {
        self.enable && self.dac_enable && (!self.length_enable || self.length > 0)
    }

    pub fn clear(&mut self) {
        self.enable = false;
        self.dac_enable = false;

        self.length_enable = false;

        self.volume = 0;
        self.frequency = 0;
    }

    pub fn reset(&mut self) {
        // When the Game Boy is switched on (before the internal boot ROM executes),
        // the values in the wave table depend on the model.
        // TODO: Make it depend on model (following is for gb:dmg)
        self.wave_ram = vec![0x84, 0x40, 0x43, 0xAA, 0x2D, 0x78, 0x92, 0x3C, 0x60, 0x59, 0x59,
                             0xB0, 0x34, 0xB8, 0x2E, 0xDA];

        self.length = 0;

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
                255
            } else {
                256
            };
        }

        // TODO: Frequency timer is reloaded with period
        // TODO: Wave channel's position is set to 0 but sample buffer is NOT refilled.
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
            // Channel 3 Sound On/Off
            // [E--- ----] DAC Power
            0xFF1A => bits::bit(self.dac_enable, 7) | 0x7F,

            // Channel 3 Volume
            // [-VV- ----] Volume
            0xFF1C => (self.volume << 5) | 0x9F,

            // Channel 3 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF1E => bits::bit(self.length_enable, 6) | 0xBF,

            // Wave RAM
            0xFF30...0xFF3F => self.wave_ram[(address - 0xFF30) as usize],

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8, frame_seq_step: u8, master_enable: bool) {
        match address {
            // Channel 3 Sound On/Off
            // [E--- ----] DAC Power
            0xFF1A if master_enable => {
                self.dac_enable = bits::test(value, 7);

                // Disabling power to the channel kills the enabled bit
                if !self.dac_enable {
                    self.enable = false;
                }
            }

            // Channel 3 Sound Length
            // [LLLL LLLL] Length load (256-L)
            0xFF1B => {
                self.length = 256u16 - value as u16;
            }

            // Channel 3 Volume
            // [-VV- ----] Volume
            0xFF1C if master_enable => {
                self.volume = (value >> 5) & 0b11;
            }

            // Channel 2 Frequency (lo)
            // [FFFF FFFF] Frequency LSB
            0xFF1D if master_enable => {
                self.frequency &= !0xFF;
                self.frequency |= value as u16;
            }

            // Channel 2 Misc.
            // [TL-- -FFF] Trigger, Length enable, Frequency MSB
            0xFF1E if master_enable => {
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

            // Wave RAM
            0xFF30...0xFF3F => {
                self.wave_ram[(address - 0xFF30) as usize] = value;
            }

            _ => {}
        }
    }
}

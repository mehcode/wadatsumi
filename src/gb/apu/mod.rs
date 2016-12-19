use std::vec::Vec;

use ::bits;
use ::sound;

mod ch1;
mod ch2;
mod ch3;
mod ch4;

// TODO: Volume envelope is shared 1,2,4
// TODO: Length counter is shared 1,2,3,4

#[derive(Default)]
pub struct APU {
    ch1: ch1::Channel1,
    ch2: ch2::Channel2,
    ch3: ch3::Channel3,
    ch4: ch4::Channel4,

    // Sound buffer
    buffer: Vec<i16>,
    buffer_index: usize,

    // Sample timing
    sample_timer: usize,

    /// Callback: Refresh (sound buffer full)
    on_refresh: Option<Box<FnMut(&[i16]) -> ()>>,

    /// Master enable
    enable: bool,

    /// Current step of the frame sequencer
    frame_seq_step: u8,

    /// Output Vin to SO2 terminal (left)
    left_vin_enable: bool,

    /// Output Vin to SO1 terminal (right)
    right_vin_enable: bool,

    /// S02 terminal volume (left)
    left_volume: u8,

    /// S01 terminal volume (right)
    right_volume: u8,

    /// Output Channel 1 to SO2 (left) terminal
    ch1_left_enable: bool,

    /// Output Channel 2 to SO2 (left) terminal
    ch2_left_enable: bool,

    /// Output Channel 3 to SO2 (left) terminal
    ch3_left_enable: bool,

    /// Output Channel 4 to SO2 (left) terminal
    ch4_left_enable: bool,

    /// Output Channel 1 to SO1 (right) terminal
    ch1_right_enable: bool,

    /// Output Channel 2 to SO1 (right) terminal
    ch2_right_enable: bool,

    /// Output Channel 3 to SO1 (right) terminal
    ch3_right_enable: bool,

    /// Output Channel 4 to SO1 (right) terminal
    ch4_right_enable: bool,
}

impl APU {
    pub fn set_on_refresh(&mut self, callback: Box<FnMut(&[i16]) -> ()>) {
        self.on_refresh = Some(callback);
    }

    pub fn reset(&mut self) {
        self.buffer_index = 0;
        self.buffer = vec![0; sound::BUFFER_SIZE * 2];

        self.sample_timer = 0;

        self.ch1.reset();
        self.ch2.reset();
        self.ch3.reset();
        self.ch4.reset();

        self.clear();

        self.enable = true;

        self.left_volume = 0x7;
        self.right_volume = 0x7;

        self.ch1_left_enable = true;
        self.ch2_left_enable = true;
        self.ch3_left_enable = true;
        self.ch4_left_enable = true;

        self.ch1_right_enable = true;
        self.ch2_right_enable = true;
        self.ch3_right_enable = false;
        self.ch4_right_enable = false;
    }

    pub fn clear(&mut self) {
        self.enable = false;

        self.frame_seq_step = 0;

        self.left_volume = 0;
        self.right_volume = 0;

        self.left_vin_enable = false;
        self.right_vin_enable = false;

        self.ch1_left_enable = false;
        self.ch2_left_enable = false;
        self.ch3_left_enable = false;
        self.ch4_left_enable = false;

        self.ch1_right_enable = false;
        self.ch2_right_enable = false;
        self.ch3_right_enable = false;
        self.ch4_right_enable = false;
    }

    pub fn step(&mut self) {
        // Step: Channels
        self.ch1.step();
        self.ch2.step();
        self.ch3.step();
        self.ch4.step();

        // Collect sample from channels (if ready)
        if self.sample_timer > 0 {
            self.sample_timer -= 1;
        }

        if self.sample_timer == 0 {
            let mut sample_l = 0;
            let mut sample_r = 0;

            if self.enable {
                let ch1 = self.ch1.sample();
                let ch2 = self.ch2.sample();
                let ch3 = self.ch3.sample();
                let ch4 = self.ch4.sample();

                if self.ch1_left_enable {
                    sample_l += ch1;
                }
                if self.ch2_left_enable {
                    sample_l += ch2;
                }
                if self.ch3_left_enable {
                    sample_l += ch3;
                }
                if self.ch4_left_enable {
                    sample_l += ch4;
                }

                if self.ch1_right_enable {
                    sample_r += ch1;
                }
                if self.ch2_right_enable {
                    sample_r += ch2;
                }
                if self.ch3_right_enable {
                    sample_r += ch3;
                }
                if self.ch4_right_enable {
                    sample_r += ch4;
                }
            }

            sample_l *= (self.left_volume as i16) * 8;
            sample_r *= (self.right_volume as i16) * 8;

            self.buffer[self.buffer_index] = sample_l;
            self.buffer[self.buffer_index + 1] = sample_r;
            self.buffer_index += 2;

            if self.buffer_index >= (sound::BUFFER_SIZE * 2) {
                self.buffer_index = 0;

                if let &mut Some(ref mut on_refresh) = &mut self.on_refresh {
                    (on_refresh)(&self.buffer);
                }
            }

            // Reload sample timer
            self.sample_timer = 4194304 / sound::SAMPLE_RATE;
        }
    }

    pub fn on_change_div(&mut self, div_last: u16, div: u16) {
        if !self.enable {
            return;
        }

        // The APU is driven off ticks of the DIV timer
        // TODO: Double speed mode (APU goes the same speed regardless of the CPU speeding up)
        if bits::test((div_last >> 8) as u8, 4) && !bits::test((div >> 8) as u8, 4) {
            if self.frame_seq_step % 2 == 0 {
                // Steps 0, 2, 4, and 6 clock the length counters (every 16,384 T-cycles)
                self.ch1.step_length();
                self.ch2.step_length();
                self.ch3.step_length();
                self.ch4.step_length();
            }

            if self.frame_seq_step == 7 {
                // Step 7 clocks the volume envelope
                self.ch1.step_volume();
                self.ch2.step_volume();
                self.ch4.step_volume();
            }

            if self.frame_seq_step == 6 || self.frame_seq_step == 2 {
                // Steps 6 and 2 clock the sweep
                self.ch1.step_sweep();
            }

            self.frame_seq_step += 1;
            self.frame_seq_step &= 7;
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF10...0xFF14 => self.ch1.read(address),
            0xFF16...0xFF19 => self.ch2.read(address),
            0xFF1A...0xFF1E | 0xFF30...0xFF3F => self.ch3.read(address),
            0xFF20...0xFF23 => self.ch4.read(address),

            // Channel control / ON-OFF / Volume
            0xFF24 => {
                bits::bit(self.left_vin_enable, 7) | bits::bit(self.right_vin_enable, 3) |
                (self.left_volume << 4) | (self.right_volume)
            }

            // Selection of Sound output terminal
            0xFF25 => {
                bits::bit(self.ch4_left_enable, 7) | bits::bit(self.ch3_left_enable, 6) |
                bits::bit(self.ch2_left_enable, 5) |
                bits::bit(self.ch1_left_enable, 4) |
                bits::bit(self.ch4_right_enable, 3) |
                bits::bit(self.ch3_right_enable, 2) |
                bits::bit(self.ch2_right_enable, 1) |
                bits::bit(self.ch1_right_enable, 0)
            }

            // Sound On/Off
            0xFF26 => {
                bits::bit(self.enable, 7) | bits::bit(self.ch4.is_enabled(), 3) |
                bits::bit(self.ch3.is_enabled(), 2) |
                bits::bit(self.ch2.is_enabled(), 1) |
                bits::bit(self.ch1.is_enabled(), 0) | 0x70
            }

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // TODO(Architecture): Each channel needs a read-only reference to the frame sequencer
            //                     step
            0xFF10...0xFF14 => self.ch1.write(address, value, self.frame_seq_step, self.enable),
            0xFF16...0xFF19 => self.ch2.write(address, value, self.frame_seq_step, self.enable),
            0xFF1A...0xFF1E | 0xFF30...0xFF3F => {
                self.ch3.write(address, value, self.frame_seq_step, self.enable)
            }
            0xFF20...0xFF23 => self.ch4.write(address, value, self.frame_seq_step, self.enable),

            // Channel control / ON-OFF / Volume
            0xFF24 if self.enable => {
                self.left_vin_enable = bits::test(value, 7);
                self.right_vin_enable = bits::test(value, 3);
                self.left_volume = (value >> 4) & 0b111;
                self.right_volume = value & 0b111;
            }

            // Selection of Sound output terminal
            0xFF25 if self.enable => {
                self.ch4_left_enable = bits::test(value, 7);
                self.ch3_left_enable = bits::test(value, 6);
                self.ch2_left_enable = bits::test(value, 5);
                self.ch1_left_enable = bits::test(value, 4);
                self.ch4_right_enable = bits::test(value, 3);
                self.ch3_right_enable = bits::test(value, 2);
                self.ch2_right_enable = bits::test(value, 1);
                self.ch1_right_enable = bits::test(value, 0);
            }

            // Sound On/Off
            0xFF26 => {
                self.enable = bits::test(value, 7);
                if !self.enable {
                    // When sound is disabled; clear all channels
                    self.clear();
                    self.ch1.clear();
                    self.ch2.clear();
                    self.ch3.clear();
                    self.ch4.clear();
                }
            }

            _ => {}
        }
    }
}

mod ch1;
mod ch2;
mod ch3;
mod ch4;

// TODO: Volume envelope is shared 1,2,4
// TODO: Length counter is shared 1,2,3,4

#[derive(Default)]
pub struct APU {
    ch1: ch1::Channel2,
    ch2: ch2::Channel2,
    ch3: ch3::Channel3,
    ch4: ch4::Channel4,

    // Output Vin to SO2 terminal (left)
    left_vin_enable: bool,

    // Output Vin to SO1 terminal (right)
    right_vin_enable: bool,

    // S02 terminal volume (left)
    left_volume: u8,

    // S01 terminal volume (right)
    right_volume: u8,
}

impl APU {
    pub fn reset(&mut self) {
        self.ch1.reset();
        self.ch2.reset();
        self.ch3.reset();
        self.ch4.reset();

        self.left_vin_enable = false;
        self.right_vin_enable = false;
        self.left_volume = 0;
        self.right_volume = 0;
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF10...0xFF14 => self.ch1.read(address),
            0xFF16...0xFF19 => self.ch2.read(address),
            0xFF1A...0xFF1E => self.ch3.read(address),
            0xFF20...0xFF23 => self.ch4.read(address),
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF10...0xFF14 => self.ch1.write(address, value),
            0xFF16...0xFF19 => self.ch2.write(address, value),
            0xFF1A...0xFF1E => self.ch3.write(address, value),
            0xFF20...0xFF23 => self.ch4.write(address, value),
            _ => {}
        }
    }
}

#![allow(unknown_lints)]

use std::fs::File;
use std::io;
use std::io::Read;
use std::vec;
use std::string;
use std::ascii::AsciiExt;

use ::bits;

#[allow(enum_variant_names)]
#[derive(PartialEq, Debug)]
pub enum MBC {
    // No MBC (less than 32 KiB)
    None,

    MBC1,
    MBC2,
    MBC3,
    MBC4,
    MBC5,

    PocketCamera,

    BandaiTama5,

    HUC3,
    HUC1,

    MMM01,
}

impl Default for MBC {
    fn default() -> MBC {
        MBC::None
    }
}

#[derive(Default)]
pub struct Cartridge {
    /// [0x0 -] Cartridge ROM (loaded from ROM file)
    pub rom: vec::Vec<u8>,

    /// [-] Cartridge RAM
    pub ram: vec::Vec<u8>,

    /// [0x134 - 0x0143] Title
    pub title: string::String,

    /// [0x0143] CGB Support Flag
    ///     0x80 — Game supports CGB functions, but works on old gameboys also.
    ///     0xC0 — Game works on CGB only (physically the same as 80h).
    pub cgb: u8,

    /// [0x0146] SGB Support Flag
    ///     0x00 — No SGB functions (Normal Gameboy or CGB only game)
    ///     0x03 — Game supports SGB functions
    pub sgb: u8,

    /// [0x0148] ROM Size (in bytes)
    pub rom_size: u32,

    /// [0x0149] RAM Size (in bytes)
    pub ram_size: u32,

    /// [0x0147] Type
    ///     Specifies which Memory Bank Controller (if any) is used in the cartridge,
    ///     and if further external hardware exists in the cartridge.
    type_: u8,

    /// [0x0147] Memory Bank Controller (derived from type)
    pub mbc: MBC,

    /// [0x0147] Has Battery (derived from type)
    pub has_battery: bool,

    /// [0x0147] Has RAM (derived from type)
    pub has_ram: bool,

    /// [0x0147] Has Rumble (derived from type)
    pub has_rumble: bool,

    /// [0x0147] Has Timer (derived from type)
    pub has_timer: bool,

    /// Selected ROM Bank
    rom_bank: usize,

    /// Selected RAM Bank
    ram_bank: usize,

    /// RAM Enabled (currently)
    ram_enable: bool,

    /// Timer Enabled (currently)
    timer_enable: bool,

    /// MBC1 Banking Mode ROM/RAM (true=RAM, false=ROM)
    mbc1_mode: bool,
}

impl Cartridge {
    pub fn reset(&mut self) {
        self.rom_bank = 1;
        self.ram_bank = 0;
        self.ram_enable = false;
        self.timer_enable = false;
        self.mbc1_mode = false;
    }

    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        // Read in cartridge memory
        let mut stream = try!(File::open(filename));
        try!(stream.read_to_end(&mut self.rom));

        // Parse ROM size
        // TODO(gameboy) match ROM size against read size
        self.rom_size = (match self.rom[0x0148] {
            // 32 KiB (no ROM banking)
            0x0 => 32,

            //  64 KiB (4 banks)
            0x1 => 64,

            // 128 KiB (8 banks)
            0x2 => 128,

            // 256 KiB (16 banks)
            0x3 => 256,

            // 512 KiB (32 banks)
            0x4 => 512,

            //   1 MiB (64 banks)
            0x5 => 1024,

            //   2 MiB (128 banks)
            0x6 => 2048,

            //   4 MiB (256 banks)
            0x7 => 4096,

            // 1.1 MiB (72 banks)
            0x52 => 1152,

            // 1.2 MiB (80 banks)
            0x53 => 1280,

            // 1.5 MiB (96 banks)
            0x54 => 1536,

            _ => {
                // Unknown ROM size code
                // TODO(rust): Is there a way to get access to the "otherwise" value
                panic!(format!("unknown rom size code: {}", self.rom[0x148]));
            }
        }) * 1024;

        debug!("rom size: {}", self.rom_size);

        // Set SGB / CGB support flags
        self.sgb = self.rom[0x0146];
        self.cgb = self.rom[0x0143];

        debug!("sgb: {:x}", self.sgb);
        debug!("cgb: {:x}", self.cgb);

        // Set cartridge type
        self.type_ = self.rom[0x0147];
        debug!("type: {:x}", self.type_);

        // Parse memory bank mbc
        self.mbc = match self.type_ {
            0x00 | 0x08 | 0x09 => MBC::None,
            0x01...0x03 => MBC::MBC1,
            0x05 | 0x06 => MBC::MBC2,
            0x0B...0x0D => MBC::MMM01,
            0x0F...0x13 => MBC::MBC3,
            0x15...0x17 => MBC::MBC4,
            0x19...0x1E => MBC::MBC5,
            0xFC => MBC::PocketCamera,
            0xFD => MBC::BandaiTama5,
            0xFE => MBC::HUC3,
            0xFF => MBC::HUC1,
            _ => {
                // Unknown cartridge type
                // TODO(rust): Is there a way to get access to the "otherwise" value
                panic!(format!("unknown cartridge type: {}", self.ram[0x148]));
            }
        };

        // Explode if we got a MBC type we don't support
        match self.mbc {
            MBC::None | MBC::MBC1 | MBC::MBC2 | MBC::MBC3 | MBC::MBC5 => {}
            _ => {
                panic!(format!("unsupported memory bank controller: {:?}", self.mbc));
            }
        }

        debug!("memory bank controller: {:?}", self.mbc);

        // Parse battery-backed RAM enable/disable
        self.has_battery = match self.type_ {
            0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x17 | 0x1B | 0x1E | 0xFF => true,
            _ => false,
        };

        debug!("has battery: {:?}", self.has_battery);

        // Parse RAM enable/disable
        self.has_ram = (self.has_battery && self.type_ != 0x0F) || self.mbc == MBC::MBC2 ||
                       match self.type_ {
            0x02 | 0x08 | 0x0C | 0x12 | 0x16 | 0x1A | 0x1D => true,
            _ => false,
        };

        debug!("has ram: {:?}", self.has_ram);

        // Parse timer enable/disable
        self.has_timer = match self.type_ {
            0x0F | 0x10 => true,
            _ => false,
        };

        debug!("has timer: {:?}", self.has_timer);

        // Parse rumble enable/disable
        self.has_rumble = match self.type_ {
            0x1C...0x1E => true,
            _ => false,
        };

        debug!("has rumble: {:?}", self.has_rumble);

        // Parse RAM size
        self.ram_size = if self.mbc == MBC::MBC2 {
            512
        } else {
            (match self.rom[0x0149] {
                // None
                0x0 => 0,

                // 2 KiB
                0x1 => 2,

                // 8 KiB
                0x2 => 8,

                // 32 KiB (4 banks of 8 KiB)
                0x3 => 32,

                _ => {
                    // Unknown RAM size code
                    // TODO(rust): Is there a way to get access to the "otherwise" value
                    panic!(format!("unknown ram size code: {}", self.ram[0x148]));
                }
            }) * 1024
        };

        // If we have a nonzero ram size; allocate some ram
        self.ram.resize(self.ram_size as usize, 0);
        debug!("ram size: {}", self.ram_size);

        // Parse title
        self.title.truncate(0);
        self.title.reserve(16);

        for i in 0..16 {
            let c = self.rom[0x134 + i];
            if !c.is_ascii() || c > 0x7F || c < 0x20 {
                break;
            }

            self.title.push(c as char);
        }

        debug!("title: {}", self.title);

        Ok(())
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0...0x3FFF => {
                // ROM Bank $0
                self.rom[address as usize]
            }

            0x4000...0x7FFF => {
                // ROM Bank $<N>
                self.rom[(self.rom_bank * 0x4000) + (address - 0x4000) as usize]
            }

            0xA000...0xBFFF => {
                // RAM Bank $<N>
                let i = (address - 0xA000) as usize + (self.ram_bank * 0x2000);
                if self.ram_enable && i < (self.ram_size as usize) {
                    self.ram[i]
                } else {
                    // RAM is disabled (or too small)
                    0xFF
                }
            }

            _ => {
                // Unhandled
                0xFF
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            /// RAM Enable: MBC1, MBC2, MBC3 (*), MBC5
            0x0000...0x1FFF => {
                self.ram_enable = (value & 0x0A) != 0;

                // MBC3 additionally affects the `timer_enable` flag here
                if self.mbc == MBC::MBC3 {
                    self.timer_enable = self.ram_enable;
                }
            }

            // MBC5: Lower 8 bits of ROM bank number
            0x2000...0x2FFF if (self.mbc == MBC::MBC5) => {
                self.rom_bank &= (!0xFF) as usize;
                self.rom_bank |= (value & 0xFF) as usize;
                self.limit_rom_bank();
            }

            // MBC5: Upper 1 bits of ROM bank number
            0x3000...0x3FFF if (self.mbc == MBC::MBC5) => {
                self.rom_bank &= (!0x100) as usize;
                self.rom_bank |= ((value & 0x01) as usize) << 8;
                self.limit_rom_bank();
            }

            // MBC1: Lower 5 bits of ROM bank number
            // MBC2: Lower (all) 4 bits; bit8 of address must be 1 of ROM bank number
            // MBC3: Lower (all) 7 bits of ROM bank number
            0x2000...0x3FFF if self.mbc != MBC::MBC5 => {
                let n = match self.mbc {
                    MBC::MBC1 => 5,
                    MBC::MBC2 => 4,
                    MBC::MBC3 => 7,
                    _ => {
                        return;
                    }
                };

                let mask = bits::mask(n);

                // In MBC2; the 8th bit of the address must be 1
                if self.mbc == MBC::MBC2 && (address & 0x100) == 0 {
                    return;
                }

                self.rom_bank &= (!mask) as usize;
                self.rom_bank |= (value & mask) as usize;
                self.limit_rom_bank();
            }

            // MBC1: RAM Bank Number - or - Upper 2 Bits of ROM Bank Number
            // MBC3: RAM Bank Number - or - RTC Register Select
            // MBC5: RAM Bank Number - and - Rumble ON/OFF
            0x4000...0x5FFF => {
                if (self.mbc == MBC::MBC1 && self.mbc1_mode) ||
                   (self.mbc == MBC::MBC3 && value < 0x08) ||
                   (self.mbc == MBC::MBC5) {
                    // RAM Bank Number
                    //  MBC1: Max of 0x3
                    //  MBC3: Max of 0x3
                    //  MBC5: Max of 0xF
                    self.ram_bank = value as usize;
                    self.ram_bank &= if self.mbc == MBC::MBC1 || self.mbc == MBC::MBC3 {
                        0x3
                    } else {
                        0xF
                    };
                }

                if self.mbc == MBC::MBC1 && !self.mbc1_mode {
                    // MBC1 ROM Banking Mode
                    self.rom_bank &= (!0x60) as usize;
                    self.rom_bank |= ((value & 0x3) as usize) << 5;
                    self.limit_rom_bank();
                }

                if self.mbc == MBC::MBC3 && value >= 0x08 {
                    warn!("unsupported: rtc register selected {:X}", value);
                }

                if self.mbc == MBC::MBC5 {
                    // If bit 4 is 1; start rumble
                    // If bit 4 is 0; stop rumble
                }
            }

            // MBC3: Latch Clock Data
            0x6000...0x7FFF if self.mbc == MBC::MBC3 => {
                warn!("unsupported: latch clock data <- {:X}", value);
            }

            // MBC1: ROM/RAM Mode Select
            0x6000...0x7FFF if self.mbc == MBC::MBC1 => {
                self.mbc1_mode = value & 1 != 0;
            }

            0xA000...0xBFFF => {
                // RAM Bank $<N>
                let i = (address - 0xA000) as usize + (self.ram_bank as usize) * 0x2000;
                if self.ram_enable && i < (self.ram_size as usize) {
                    self.ram[i] = value;
                }
            }

            _ => {
                // Unhandled
            }
        }
    }

    /// Limit ROM Bank Number
    ///     MBC1: 0, 20h, 40h, and 60h cannot be selected; max of 0x7F
    ///     MBC2: 0 cannot be selected; max of 0xF
    ///     MBC3: 0 cannot be selected; max of 0x7F
    ///     MBC5: 0 _can_ be selected; max of 0x1E0
    fn limit_rom_bank(&mut self) {
        // Ensure ROM bank doesn't select invalid banks and is under the limit for the MBC

        // Wrap around the max size
        self.rom_bank &= match self.mbc {
            MBC::MBC1 | MBC::MBC3 => 0x7F,
            MBC::MBC2 => 0xF,
            MBC::MBC5 => 0x1E0,
            _ => 0,
        };

        // Bump on invalid bank numbers
        match self.mbc {
            MBC::MBC1 => {
                if self.rom_bank == 0 || self.rom_bank == 0x20 || self.rom_bank == 0x40 ||
                   self.rom_bank == 0x60 {
                    self.rom_bank += 1;
                }
            }

            MBC::MBC2 | MBC::MBC3 => {
                if self.rom_bank == 0 {
                    self.rom_bank += 1;
                }
            }

            _ => {}
        }
    }
}

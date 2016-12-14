use std::fs::File;
use std::io;
use std::io::Read;
use std::vec;
use std::string;
use std::ascii::AsciiExt;

#[allow(enum_variant_names)]
#[derive(PartialEq)]
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
}

impl Cartridge {
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

        // Set SGB / CGB support flags
        self.sgb = self.rom[0x0146];
        self.cgb = self.rom[0x0143];

        // Set cartridge type
        self.type_ = self.rom[0x0147];

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

        // Parse battery-backed RAM enable/disable
        self.has_battery = match self.type_ {
            0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x17 | 0x1B | 0x1E | 0xFF => true,
            _ => false,
        };

        // Parse RAM enable/disable
        self.has_ram = (self.has_battery && self.type_ != 0x0F) || self.mbc == MBC::MBC2 ||
                       match self.type_ {
            0x02 | 0x08 | 0x0C | 0x12 | 0x16 | 0x1A | 0x1D => true,
            _ => false,
        };

        // Parse timer enable/disable
        self.has_timer = match self.type_ {
            0x0F | 0x10 => true,
            _ => false,
        };

        // Parse rumble enable/disable
        self.has_rumble = match self.type_ {
            0x1C...0x1E => true,
            _ => false,
        };

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

        Ok(())
    }

    pub fn read(&mut self, address: u16) -> u8 {
        // TODO: MBC / RAM
        if address <= 0x7FFF {
            self.rom[address as usize]
        } else {
            // Unhandled
            0xFF
        }
    }

    pub fn write(&mut self, _: u16, _: u8) {
        // TODO: MBC
    }
}

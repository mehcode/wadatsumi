use std::io::Read;
use std::ops::Range;
use errors::*;
use bus::Bus;

pub struct Cartridge {
    pub title: String,
    rom: Box<[u8]>,
    ram: Box<[u8]>,
}

// TODO(@rust): Is there a better way to declare this?
const NINTENDO_LOGO: [u8; 0x30] = [
    0xCE,
    0xED,
    0x66,
    0x66,
    0xCC,
    0x0D,
    0x00,
    0x0B,
    0x03,
    0x73,
    0x00,
    0x83,
    0x00,
    0x0C,
    0x00,
    0x0D,
    0x00,
    0x08,
    0x11,
    0x1F,
    0x88,
    0x89,
    0x00,
    0x0E,
    0xDC,
    0xCC,
    0x6E,
    0xE6,
    0xDD,
    0xDD,
    0xD9,
    0x99,
    0xBB,
    0xBB,
    0x67,
    0x63,
    0x6E,
    0x0E,
    0xEC,
    0xCC,
    0xDD,
    0xDC,
    0x99,
    0x9F,
    0xBB,
    0xB9,
    0x33,
    0x3E,
];

impl Cartridge {
    /// Read a cartridge from an IO stream.
    pub fn from_reader<R>(mut reader: R) -> Result<Self>
    where
        R: Read,
    {
        // Read up to the end of the cartridge header
        let mut header = vec![0; 0x014F];
        reader.read_exact(header.as_mut_slice())?;

        // If 0x14b "Old Licnesee Code" is set to 0x33, this cartridge
        // was almost certainly made after the SGB was released. We use this to determine
        // how to interpret the last 5 bytes of the title area.
        let is_newer_cartridge = header[0x14b] == 0x33;

        // TODO: 0x100 .. 0x0104 - Entry Point
        //  After displaying the Nintendo Logo, the built-in boot procedure jumps to this address
        //Instruction::from_slice(&header[0x100])

        // Validate contents of "Nintendo Logo"
        if header[0x104..0x134] != NINTENDO_LOGO[..] {
            bail!("invalid contents for Nintendo Logo at 0x104..0x134");
        }

        // Extract the "Title"
        let title = String::from_utf8_lossy(
            // If we were determined to be a "newer" cartridge then assume the last 5
            // bytes are unrelated to the title.
            &header[0x134..(if is_newer_cartridge { 0x13F } else { 0x144 })]
                // Chop off the the top half of all the bytes to protect against weirdness
                .into_iter().map(|b| b & 0x7F).collect::<Vec<_>>(),
        // Chop off trailing NUL
        ).trim_right_matches('\0')
            .to_string();

        // Determine the ROM size
        let rom_size = match header[0x148] {
            shift @ 0...7 => 32 << shift,

            0x52 => 1152,
            0x53 => 1280,
            0x54 => 1536,

            size => bail!("unsupported rom size {:02x}", size),
        };

        // Determine the RAM size
        let ram_size = match header[0x149] {
            0 => 0,
            1 => 2,
            2 => 8,
            3 => 32,

            size => bail!("unsupported ram size {:02x}", size),
        };

        // Check CGB Mode for _required_; we don't support CGB (yet)
        if (header[0x143] >> 4) == 0xc {
            bail!("requires cgb support");
        }

        // Check Cartridge Kind for 0 "ROM ONLY"; we don't support anything else (yet)
        // TODO(@rust): How to avoid repeating header[0x147] ?
        // if header[0x147] != 0 {
        //     bail!("unsupported cartridge type {}", header[0x147]);
        // }

        // Calculate the header checksum
        //  x=0:FOR i=0134h TO 014Ch:x=x-MEM[i]-1:NEXT
        let checksum: u8 = header[0x134..0x14d]
            .into_iter()
            .fold(0, |x, &val| x.wrapping_sub(val).wrapping_sub(1));

        // TODO(@rust): How to avoid repeating header[0x14d] ?
        if checksum != header[0x14d] {
            bail!(
                "invalid header checksum; expected ${:02x}, found ${:02x}",
                header[0x14d],
                checksum
            );
        }

        // Allocate cartridge ROM, copy the header to the start
        let mut rom = vec![0; (rom_size * 1024)].into_boxed_slice();
        rom[..header.len()].copy_from_slice(&mut header[..]);
        reader.read_exact(&mut rom[header.len()..])?;

        // Allocate cartridge RAM
        let ram = vec![0; (ram_size * 1024)].into_boxed_slice();

        Ok(Cartridge { title, rom, ram })
    }
}

impl Bus for Cartridge {
    fn read8(&self, address: u16) -> u8 {
        // TODO: Don't assume rom-only
        if address <= 0x7fff {
            self.rom[address as usize]
        } else {
            warn!("unhandled read from Cartridge: {:04x}", address);

            0
        }
    }

    fn write8(&mut self, address: u16, value: u8) {
        // TODO: Don't assume rom-only
        warn!("unhandled write to Cartridge: {:04x} <- {:02x}", address, value);
    }
}

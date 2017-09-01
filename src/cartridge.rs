use std::io::Read;

use errors::*;

pub struct Cartridge {
    pub title: String,

    /// ROM size of the cartridge in bytes
    pub rom_size: usize,

    /// External RAM size in the cartridge, if any, in bytes
    pub ram_size: usize,
}

const NINTENDO_LOGO: [u8; 0x30] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

impl Cartridge {
    /// Read a cartridge from an IO stream.
    pub fn from_reader<R>(mut reader: R) -> Result<()>
    where
        R: Read,
    {
        // Read up to the end of the cartridge header
        let mut data = vec![0; 0x014F];
        reader.read_exact(data.as_mut_slice())?;

        // If 0x14b "Old Licnesee Code" is set to 0x33, this cartridge
        // was almost certainly made after the SGB was released. We use this to determine
        // how to interpret the last 5 bytes of the title area.
        let is_newer_cartridge = data[0x14b] == 0x33;

        // TODO: 0x100 .. 0x0104 - Entry Point
        //  After displaying the Nintendo Logo, the built-in boot procedure jumps to this address
        //Instruction::from_slice(&data[0x100])

        // Validate contents of "Nintendo Logo"
        // TODO: Should this be a hard error
        if data[0x104..0x134] != NINTENDO_LOGO[..] {
            warn!("invalid contents for Nintendo Logo at 0x104..0x134");
        }

        // Extract the "Title"
        let title = {
            // If we were determined to be a "newer" cartridge then assume the last 5
            // bytes are unrelated to the title.
            let data = &data[0x134..(if is_newer_cartridge { 0x13F } else { 0x144 })];

            // Chop off the the top half of all the bytes to protect against weirdness
            let data = data.into_iter().map(|b| b & 0x7F).collect::<Vec<_>>();

            String::from_utf8_lossy(&data).into_owned()
        };

        info!("title: {}", title);

        // Determine the ROM size
        let rom_size = match data[0x148] {
            shift @ 0...7 => 32 << shift,

            0x52 => 1152,
            0x53 => 1280,
            0x54 => 1536,

            size => {
                bail!("unsupported rom size {:02x}", size)
            }
        };

        // Determine the RAM size
        let ram_size = match data[0x149] {
            0 => 0,
            1 => 2,
            2 => 8,
            3 => 32,

            size => {
                bail!("unsupported ram size {:02x}", size)
            }
        };

        Ok(())
    }
}

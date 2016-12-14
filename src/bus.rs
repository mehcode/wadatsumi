use std::vec;

use ::cart;

/// The Bus is the interconnect that facilitates communication from the CPU and the various other
/// components in the machine.
#[derive(Default)]
pub struct Bus {
    /// Component: Cartridge (Reader)
    pub cart: cart::Cartridge,

    /// [0xC000 - 0xDFFF] Work RAM (WRAM)
    ///   8 KiB in GB
    ///  32 KiB in CGB
    pub wram: vec::Vec<u8>,

    /// [0xFF70] WRAM Bank 0 - 7 (SVBK) — CGB Only
    pub wram_bank: u8,

    /// [0xFF80 - 0xFFFE] High RAM (HRAM) — 127 Bytes
    pub hram: vec::Vec<u8>,
}

impl Bus {
    /// Step
    pub fn step(&mut self) {
        // TODO: [...]
    }

    /// Read
    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // ROM: Cartridge
            0x0000...0x7FFF => self.cart.read(address),

            // Work RAM
            0xC000...0xFDFF => {
                self.wram[(address as usize & 0x1FFF) + (self.wram_bank as usize * 0x2000)]
            }

            // High RAM
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize],

            _ => {
                // Unhandled
                warn!("unhandled read at {:#04X}", address);
                0xFF
            }
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // ROM: Cartridge
            0x0000...0x7FFF => self.cart.write(address, value),

            // Work RAM
            0xC000...0xFDFF => {
                self.wram[(address as usize & 0x1FFF) + (self.wram_bank as usize * 0x2000)] = value;
            }

            // High RAM
            0xFF80...0xFFFE => {
                self.hram[(address - 0xFF80) as usize] = value;
            }

            _ => {
                // Unhandled
                warn!("unhandled write at {:#04X} with {:#02X}", address, value);
            }
        }
    }
}

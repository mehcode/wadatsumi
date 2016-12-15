use std::vec::Vec;

use ::cart;
use ::gpu;

/// The Bus is the interconnect that facilitates communication from the CPU and the various other
/// components in the machine.
#[derive(Default)]
pub struct Bus {
    /// Component: Cartridge (Reader)
    pub cart: cart::Cartridge,

    /// Component: GPU
    pub gpu: gpu::GPU,

    /// [0xC000 - 0xDFFF] Work RAM (WRAM)
    ///   8 KiB in GB
    ///  32 KiB in CGB
    pub wram: Vec<u8>,

    /// [0xFF70] WRAM Bank 0 - 7 (SVBK) — CGB Only
    pub wram_bank: u8,

    /// [0xFF80 - 0xFFFE] High RAM (HRAM) — 127 Bytes
    pub hram: Vec<u8>,

    /// [0xFFFF] Interrupt Enable (IE) R/W
    pub ie: u8,

    /// [0xFF0F] Interrupt Flag (IF) R/W
    pub if_: u8,
}

impl Bus {
    /// Step
    pub fn step(&mut self) {
        // TODO(architecture): The machine/bus is stepped each M-cycle. Some components operate
        //      by M-cycles and others by T-cycles

        self.gpu.step(&mut self.if_);
    }

    /// Reset
    pub fn reset(&mut self) {
        // Interrupt Enable/Flag
        self.ie = 0;
        self.if_ = 0;

        // Reset: WRAM
        self.wram.clear();
        // TODO: Depends on model (gb/cgb)
        self.wram.resize(32 * 1024, 0);

        // Reset: HRAM
        self.hram.clear();
        self.hram.resize(127, 0);

        // Reset: GPU
        self.gpu.reset();

        // Reset: (various)
        // TODO: Remove these as each component should be in charge of reset; this is just copied
        //       from pandocs for easy right now
        self.write(0xFF05, 0x00);
        self.write(0xFF06, 0x00);
        self.write(0xFF07, 0x00);
        self.write(0xFF10, 0x80);
        self.write(0xFF11, 0xBF);
        self.write(0xFF12, 0xF3);
        self.write(0xFF14, 0xBF);
        self.write(0xFF16, 0x3F);
        self.write(0xFF17, 0x00);
        self.write(0xFF19, 0xBF);
        self.write(0xFF1A, 0x7F);
        self.write(0xFF1B, 0xFF);
        self.write(0xFF1C, 0x9F);
        self.write(0xFF1E, 0xBF);
        self.write(0xFF20, 0xFF);
        self.write(0xFF21, 0x00);
        self.write(0xFF22, 0x00);
        self.write(0xFF23, 0xBF);
        self.write(0xFF24, 0x77);
        self.write(0xFF25, 0xF3);
        self.write(0xFF26, 0xF1);
    }

    /// Read
    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // Cartridge
            0x0000...0x7FFF => self.cart.read(address),

            // Video RAM, OAM, GPU registers
            0x8000...0x9FFF | 0xFE00...0xFE9F | 0xFF40...0xFF4F | 0xFF68...0xFF6B => {
                self.gpu.read(address)
            }

            // Work RAM
            0xC000...0xFDFF => {
                self.wram[(address as usize & 0x1FFF) + (self.wram_bank as usize * 0x2000)]
            }

            // High RAM
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize],

            // Interrupt Flag (IF)
            0xFF0F => (self.if_ | 0xE0),

            // Interrupt Enable (IE)
            0xFFFF => (self.ie | 0xE0),

            _ => {
                // Unhandled
                // warn!("unhandled read at {:#04X}", address);
                0xFF
            }
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // Cartridge
            0x0000...0x7FFF => self.cart.write(address, value),

            // Video RAM, OAM, GPU registers
            0x8000...0x9FFF | 0xFE00...0xFE9F | 0xFF40...0xFF4F | 0xFF68...0xFF6B => {
                self.gpu.write(address, value);
            }

            // Work RAM
            0xC000...0xFDFF => {
                self.wram[(address as usize & 0x1FFF) + (self.wram_bank as usize * 0x2000)] = value;
            }

            // High RAM
            0xFF80...0xFFFE => {
                self.hram[(address - 0xFF80) as usize] = value;
            }

            // Interrupt Flag (IF)
            0xFF0F => {
                self.if_ = value & !0xE0;
            }

            // Interrupt Enable (IE)
            0xFFFF => {
                self.ie = value & !0xE0;
            }

            _ => {
                // Unhandled
                // warn!("unhandled write at {:#04X} with {:#02X}", address, value);
            }
        }
    }
}

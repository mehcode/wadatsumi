use std::vec;
use std::cell::RefCell;
use std::rc::Rc;

/// Memory Rule
///   Each subsystem in the architecture could implement the memory rule trait.
///   These can then be pushed in sequence here to handle incoming addresses.
pub trait MemoryRule {
    /// Attempt to read an address
    /// Place result in `ptr` if successful
    /// Returns true if successful; otherwise, false
    fn try_read(&mut self, address: u16, ptr: &mut u8) -> bool;

    /// Attempt to write an address
    /// Returns true if successful; otherwise, false
    fn try_write(&mut self, address: u16, value: u8) -> bool;
}

/// Memory Management Unit (MMU)
#[derive(Default)]
pub struct MMU {
    /// [0xC000 - 0xDFFF] Work RAM (WRAM)
    ///   8 KiB in GB
    ///  32 KiB in CGB
    wram: vec::Vec<u8>,

    /// [0xFF70] WRAM Bank 0 - 7 (SVBK) — CGB Only
    wram_bank: u8,

    /// [0xFF80 - 0xFFFE] High RAM (HRAM) — 127 Bytes
    hram: vec::Vec<u8>,

    /// Memory rules
    pub rules: vec::Vec<Rc<RefCell<MemoryRule>>>,
}

impl MMU {
    pub fn reset(&mut self) {
        // TODO(gameboy): Depends on model (gb/cgb)
        self.wram.clear();
        // TODO(gameboy): Random fill values
        self.wram.resize(8 * 1024, 0);

        self.hram.clear();
        // TODO(gameboy): Random fill values
        self.hram.resize(127, 0);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        // Check memory rules
        // for rule in self.rules {
        //    let mut r: u8 = 0;
        //    if rule.get_mut().try_read(address, &mut r) {
        //        return r;
        //    }
        //

        // Unhandled
        // TODO: Warn
        0xFF
    }

    pub fn write(&mut self, address: u16, value: u8) {
        // Check memory rules
        // for rule in self.rules {
        //    if rule.get_mut().try_write(address, value) {
        //        return;
        //    }
        //

        // Unhandled
        // TODO: Warn
    }
}

impl MemoryRule for MMU {
    fn try_read(&mut self, address: u16, ptr: &mut u8) -> bool {
        *ptr = match address {
            // Work RAM
            0xC000...0xFDFF => {
                // 0xE000 - 0xFDFF mirror what is at 0xC000 - 0xDDFF
                self.wram[((address & 0x1FFF) + ((self.wram_bank as u16) * 0x2000)) as usize]
            }

            _ => {
                return false;
            }
        };

        true
    }

    fn try_write(&mut self, address: u16, value: u8) -> bool {
        false
    }
}

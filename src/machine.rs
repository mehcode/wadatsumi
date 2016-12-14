use std::io;

use ::cpu;
use ::bus;

#[derive(Default)]
pub struct Machine {
    /// Interconnect: Bus
    bus: bus::Bus,

    /// Component: CPU
    cpu: cpu::CPU,
}

impl Machine {
    pub fn new() -> Machine {
        Default::default()
    }

    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        self.bus.cart.open(filename)
    }

    pub fn reset(&mut self) {
        // TODO(gameboy): Depends on model (gb/cgb)
        self.bus.wram.clear();
        // TODO(gameboy): Random fill values
        self.bus.wram.resize(8 * 1024, 0);

        self.bus.hram.clear();
        // TODO(gameboy): Random fill values
        self.bus.hram.resize(127, 0);

        // Reset: CPU
        self.cpu.reset();
    }

    pub fn run(&mut self) {
        // Run: next instruction
        self.cpu.run_next(&mut self.bus);
    }
}

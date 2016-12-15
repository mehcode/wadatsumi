use std::io;

use ::cpu;
use ::bus;

#[derive(Default)]
pub struct Machine {
    // TODO: This should not be public but it is for my hacked SDL usage
    /// Interconnect: Bus
    pub bus: bus::Bus,

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
        // Reset: CPU
        self.cpu.reset();

        // Reset: Bus (and all attached components)
        self.bus.reset();
    }

    pub fn run(&mut self) {
        // Run: next instruction
        self.cpu.run_next(&mut self.bus);
    }
}

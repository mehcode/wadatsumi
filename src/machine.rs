use std::io;
use ::cpu;
use ::cart;

pub struct Machine {
    /// Component: Cartridge (Reader)
    pub cart: cart::Cartridge,

    /// Component: CPU
    pub cpu: cpu::CPU,
}

impl Machine {
    pub fn new() -> Self {
        Machine {
            cpu: cpu::CPU::new(),
            cart: cart::Cartridge::new(),
        }
    }

    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        self.cart.open(filename)
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn run(&mut self) {
        // Run: next instruction
        self.cpu.run_next();
    }

    pub fn step(&mut self) {
        unimplemented!();
    }
}

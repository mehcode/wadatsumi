use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use ::cpu;
use ::cart;
use ::mmu;

#[derive(Default)]
pub struct Machine {
    /// Component: Cartridge (Reader)
    pub cart: Rc<RefCell<cart::Cartridge>>,

    /// Component: CPU
    pub cpu: Rc<RefCell<cpu::CPU>>,

    /// Component: MMU
    pub mmu: Rc<RefCell<mmu::MMU>>,
}

impl Machine {
    pub fn new() -> Self {
        let mut m: Self = Default::default();
        // m.cpu.get_mut().mmu = Rc::downgrade(&m.mmu);

        m
    }

    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        // self.cart.get_mut().open(filename)
        Ok(())
    }

    pub fn reset(&mut self) {
        (*self.cpu.borrow_mut()).reset();
        //(self.mmu.get_mut().reset();

        // Add memory rules
        // self.mmu.get_mut().rules.push(self.cpu.clone());
    }

    pub fn run(&mut self) {
        // Run: next instruction
        // self.cpu.run_next();
    }

    pub fn step(&mut self) {
        unimplemented!();
    }
}

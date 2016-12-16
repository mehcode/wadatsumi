use std::io;
use sdl2;

use ::cpu;
use ::bus;
use ::mode;

#[derive(Default)]
pub struct Machine {
    /// Requested device/variation mode (at start)
    mode_req: Option<mode::Mode>,

    /// Current device/variation mode
    mode: Option<mode::Mode>,

    // TODO: This should not be public but it is for my hacked SDL usage
    /// Interconnect: Bus
    pub bus: bus::Bus,

    /// Component: CPU
    cpu: cpu::CPU,
}

impl Machine {
    pub fn new(mode: Option<mode::Mode>) -> Machine {
        Machine {
            mode_req: mode,
            mode: None,
            bus: Default::default(),
            cpu: Default::default(),
        }
    }

    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        // Open the ROM file (with cartridge)
        try!(self.bus.cart.open(filename));

        // If we have a nil mode_req; we need to determine the real mode to use
        if self.mode_req.is_none() {
            // TODO: mode::Mode::from_device(mode::GB) — get device mode from device
            // TODO: cgb/sgb support flag comparisons should be done in cart.rs
            if (self.bus.cart.cgb == 0x80) || (self.bus.cart.cgb == 0xC0) {
                self.mode = Some(mode::CGB_CGB);
            } else if self.bus.cart.sgb == 0x03 {
                self.mode = Some(mode::SGB_SGB2);
            } else {
                self.mode = Some(mode::GB_MGB);
            }
        } else {
            self.mode = self.mode_req;
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        // Reset: CPU
        self.cpu.reset(self.mode.unwrap());

        // Reset: Bus (and all assoc. components)
        self.bus.reset(self.mode.unwrap());
    }

    pub fn run(&mut self) {
        // Run: next instruction
        self.cpu.run_next(&mut self.bus);
    }

    pub fn on_key_down(&mut self, scancode: sdl2::keyboard::Scancode) {
        self.bus.joypad.on_key_down(scancode);
    }

    pub fn on_key_up(&mut self, scancode: sdl2::keyboard::Scancode) {
        self.bus.joypad.on_key_up(scancode);
    }
}

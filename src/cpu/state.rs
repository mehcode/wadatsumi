use std::mem;
use super::super::bus::Bus;

bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const ZERO         = 0b_1000_0000;     // Z
        const ADD_SUBTRACT = 0b_0100_0000;     // N
        const HALF_CARRY   = 0b_0010_0000;     // H
        const CARRY        = 0b_0001_0000;     // C
    }
}

#[derive(Default)]
pub struct State {
    pub(super) pc: u16,
    pub(super) a: u8,
    pub(super) b: u8,
    pub(super) c: u8,
    pub(super) d: u8,
    pub(super) e: u8,
    pub(super) h: u8,
    pub(super) l: u8,
    //pub(super) f: Flags,
}

impl State {
    pub fn next8<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let address = self.pc;
        self.pc = self.pc.wrapping_add(1);

        bus.read8(address)
    }

    pub fn next16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let l = self.next8(bus);
        let h = self.next8(bus);

        ((h as u16) << 8) | (l as u16)
    }
}

use std::fmt;
use std::mem;
use super::operands::{Address, Immediate16, Register16};
use super::io::{In16, Out16};
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

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME(@rust): Make this prettier

        write!(
            f,
            "{}{}{}{}",
            if self.contains(Flags::ZERO) { "z" } else { "-" },
            if self.contains(Flags::ADD_SUBTRACT) {
                "n"
            } else {
                "-"
            },
            if self.contains(Flags::HALF_CARRY) { "h" } else { "-" },
            if self.contains(Flags::CARRY) { "c" } else { "-" },
        )
    }
}

#[derive(Default)]
pub struct State {
    /// Stack Pointer (SP)
    pub sp: u16,

    /// Program Counter (PC)
    pub pc: u16,

    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: Flags,
}

impl State {
    pub fn reset(&mut self) {
        // TODO: Investigate how to do this properly: BIOS probably
        self.pc = 0x100;
        self.sp = 0xFFFE;
        self.a = 0x01;
        self.b = 0;
        self.c = 0x13;
        self.d = 0;
        self.e = 0xd8;
        self.h = 0x01;
        self.l = 0x4d;
        self.f.bits = 0xB0;
    }

    #[inline]
    pub fn next8<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let address = self.pc;
        self.pc = self.pc.wrapping_add(1);

        bus.read8(address)
    }

    #[inline]
    pub fn next16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let address = self.pc;
        self.pc = self.pc.wrapping_add(2);

        bus.read16(address)
    }

    #[inline]
    pub fn push16<B: Bus>(&mut self, bus: &mut B, value: u16) {
        // TODO: There is a 1-cycle delay

        self.sp = self.sp.wrapping_sub(2);
        bus.write16(self.sp, value);
    }

    #[inline]
    pub fn indirect<B: Bus>(&mut self, bus: &mut B, address: Address) -> u16 {
        match address {
            Address::Direct => Immediate16.read16(self, bus),
            Address::BC => Register16::BC.read16(self, bus),
            Address::DE => Register16::DE.read16(self, bus),
            Address::HL => Register16::HL.read16(self, bus),

            Address::HLI => {
                let address = Register16::HL.read16(self, bus);
                Register16::HL.write16(self, bus, address + 1);
                address
            }

            Address::HLD => {
                let address = Register16::HL.read16(self, bus);
                Register16::HL.write16(self, bus, address - 1);
                address
            }
        }
    }
}

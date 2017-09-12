use super::super::bus::Bus;
use super::io::{In16, In8, Out16, Out8};
use super::State;
use super::state::Flags;

#[derive(Debug, Clone, Copy)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl In8 for Register8 {
    #[inline]
    fn read8<B: Bus>(&self, state: &mut State, _: &mut B) -> u8 {
        use self::Register8::*;

        match *self {
            A => state.a,
            B => state.b,
            C => state.c,
            D => state.d,
            E => state.e,
            H => state.h,
            L => state.l,
        }
    }
}

impl Out8 for Register8 {
    #[inline]
    fn write8<B: Bus>(&self, state: &mut State, _: &mut B, value: u8) {
        use self::Register8::*;

        match *self {
            A => state.a = value,
            B => state.b = value,
            C => state.c = value,
            D => state.d = value,
            E => state.e = value,
            H => state.h = value,
            L => state.l = value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
}

impl In16 for Register16 {
    #[inline]
    fn read16<B: Bus>(&self, state: &mut State, _: &mut B) -> u16 {
        use self::Register16::*;

        match *self {
            AF => (state.a as u16) << 8 | state.f.bits() as u16,
            BC => (state.b as u16) << 8 | state.c as u16,
            DE => (state.d as u16) << 8 | state.e as u16,
            HL => (state.h as u16) << 8 | state.l as u16,
        }
    }
}

impl Out16 for Register16 {
    #[inline]
    fn write16<B: Bus>(&self, state: &mut State, _: &mut B, value: u16) {
        use self::Register16::*;

        match *self {
            AF => {
                state.a = (value >> 8) as u8;
                state.f = Flags::from_bits_truncate(value as u8);
            }

            BC => {
                state.b = (value >> 8) as u8;
                state.c = value as u8;
            }

            DE => {
                state.d = (value >> 8) as u8;
                state.e = value as u8;
            }

            HL => {
                state.h = (value >> 8) as u8;
                state.l = value as u8;
            }
        }
    }
}

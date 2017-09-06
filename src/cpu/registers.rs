use super::super::bus::Bus;
use super::io::{In8, Out8};
use super::State;

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

#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL
}

use self::Register8::*;

impl In8 for Register8 {
    #[inline]
    fn read8<B: Bus>(&self, state: &mut State, _: &mut B) -> u8 {
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

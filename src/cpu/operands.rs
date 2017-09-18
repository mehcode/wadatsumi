use std::ops::Not;
use std::fmt;
use super::super::bus::Bus;
use super::io::{In16, In8, Out16, Out8};
use super::State;
use super::state::Flags;
use super::disassembler::IntoCondition as IntoInstrCondition;
use super::instruction::Condition as InstrCondition;

/// 8-bit Register
#[derive(Debug, Clone, Copy)]
pub enum Register8 {
    /// Primary accumulator. Arithmetic instructions are encoded to work with, and only with,
    /// the accumulator.
    ///
    /// # Examples
    /// ```asm,ignore
    /// ; A <- A - C
    /// SUB C
    ///
    /// ; A <- A & C
    /// AND C
    /// ```
    A,

    B,
    C,
    D,
    E,
    H,
    L,
}

impl fmt::Display for Register8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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

/// 16-bit Registers
#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    /// Accumulator (A) and Flags (F); also known as, Processor Status Word (PSW).
    /// May only be pushed or poped.
    AF,

    /// Stack Pointer (SP)
    SP,

    BC,
    DE,
    HL,
}

impl fmt::Display for Register16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
            SP => state.sp,
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

            SP => {
                state.sp = value;
            }
        }
    }
}

/// 8-bit Immediate
#[derive(Debug, Clone, Copy)]
pub struct Immediate8;

impl In8 for Immediate8 {
    #[inline]
    fn read8<B: Bus>(&self, state: &mut State, bus: &mut B) -> u8 {
        state.next8(bus)
    }
}

/// 16-bit Immediate
#[derive(Debug, Clone, Copy)]
pub struct Immediate16;

impl In16 for Immediate16 {
    #[inline]
    fn read16<B: Bus>(&self, state: &mut State, bus: &mut B) -> u16 {
        state.next16(bus)
    }
}

/// Address
#[derive(Debug, Clone, Copy)]
pub enum Address {
    /// Immediate 16-bit operand used as an address.
    Direct,

    BC,
    DE,
    HL,

    /// Zero Page. Immediate 8-bit operand indexed into `0xFF00 ... 0xFFFF`.
    ZeroPage,

    /// Zero Page. Register C indexed into `0xFF00 ... 0xFFFF`.
    ZeroPageC,

    /// HL, Decrement or (HL-). Use the address HL then decrement HL.
    HLD,

    /// HL, Increment or (HL-). Use the address HL then increment HL.
    HLI,
}

impl In8 for Address {
    #[inline]
    fn read8<B: Bus>(&self, state: &mut State, bus: &mut B) -> u8 {
        let address = state.indirect(bus, *self);
        bus.read8(address)
    }
}

impl Out8 for Address {
    #[inline]
    fn write8<B: Bus>(&self, state: &mut State, bus: &mut B, value: u8) {
        let address = state.indirect(bus, *self);
        bus.write8(address, value)
    }
}

/// Condition
pub trait Condition: IntoInstrCondition + Copy + fmt::Debug {
    fn check(&self, state: &State) -> bool;
}

/// Unit or "constant true" condition. Used to allow a single operation
/// to optionally accept a condition.
impl Condition for () {
    #[inline]
    fn check(&self, _: &State) -> bool {
        true
    }
}

impl IntoInstrCondition for () {
    fn into_condition(self) -> Option<InstrCondition> {
        None
    }
}

pub mod condition {
    #![allow(non_camel_case_types)]

    use super::{Condition, Flags, InstrCondition, IntoInstrCondition, State};

    // FIXME: De-duplicate and reduce with macros

    #[derive(Debug, Clone, Copy)]
    pub struct ZERO;

    #[derive(Debug, Clone, Copy)]
    pub struct NOT_ZERO;

    #[derive(Debug, Clone, Copy)]
    pub struct CARRY;

    #[derive(Debug, Clone, Copy)]
    pub struct NOT_CARRY;

    impl IntoInstrCondition for ZERO {
        fn into_condition(self) -> Option<InstrCondition> {
            Some(InstrCondition::Zero)
        }
    }

    impl Condition for ZERO {
        #[inline]
        fn check(&self, state: &State) -> bool {
            state.f.contains(Flags::ZERO)
        }
    }

    impl IntoInstrCondition for NOT_ZERO {
        fn into_condition(self) -> Option<InstrCondition> {
            Some(InstrCondition::NotZero)
        }
    }

    impl Condition for NOT_ZERO {
        #[inline]
        fn check(&self, state: &State) -> bool {
            !state.f.contains(Flags::ZERO)
        }
    }

    impl IntoInstrCondition for CARRY {
        fn into_condition(self) -> Option<InstrCondition> {
            Some(InstrCondition::Carry)
        }
    }

    impl Condition for CARRY {
        #[inline]
        fn check(&self, state: &State) -> bool {
            state.f.contains(Flags::CARRY)
        }
    }

    impl IntoInstrCondition for NOT_CARRY {
        fn into_condition(self) -> Option<InstrCondition> {
            Some(InstrCondition::NotCarry)
        }
    }

    impl Condition for NOT_CARRY {
        #[inline]
        fn check(&self, state: &State) -> bool {
            !state.f.contains(Flags::CARRY)
        }
    }
}

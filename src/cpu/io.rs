use super::super::bus::Bus;
use super::State;
use super::disassembler::IntoOperand8;
use super::instruction::Operand8;

pub trait In8: IntoOperand8 + Copy {
    fn read8<B: Bus>(&self, state: &mut State, bus: &mut B) -> u8;
}

pub trait Out8: IntoOperand8 + Copy {
    fn write8<B: Bus>(&self, state: &mut State, bus: &mut B, value: u8);
}

pub trait In16 {
    fn read16<B: Bus>(&self, state: &mut State, bus: &mut B) -> u16;
}

pub trait Out16 {
    fn write16<B: Bus>(&self, state: &mut State, bus: &mut B, value: u16);
}

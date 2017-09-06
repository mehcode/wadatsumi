use super::super::bus::Bus;
use super::State;
use super::instruction::Operand8;

pub trait In8: Into<Operand8> + Copy {
    fn read8<B: Bus>(&self, state: &mut State, bus: &mut B) -> u8;
}

pub trait Out8: Into<Operand8> + Copy {
    fn write8<B: Bus>(&self, state: &mut State, bus: &mut B, value: u8);
}

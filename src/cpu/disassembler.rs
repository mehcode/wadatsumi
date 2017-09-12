use super::operations::Operations;
use super::io::{In8, Out8};
use super::instruction::{Instruction, Operand8};
use super::tracer::BusTracer;
use super::operands::{Address, Immediate8, Register16, Register8};
use super::super::bus::Bus;

pub trait ToOperand8 {
    fn to_operand8(&self, disassembler: &mut Disassembler) -> Operand8;
}

impl ToOperand8 for Address {
    fn to_operand8(&self, _: &mut Disassembler) -> Operand8 {
        Operand8::Memory(*self)
    }
}

impl ToOperand8 for Immediate8 {
    fn to_operand8(&self, disassembler: &mut Disassembler) -> Operand8 {
        Operand8::Immediate(disassembler.next8())
    }
}

impl ToOperand8 for Register8 {
    fn to_operand8(&self, _: &mut Disassembler) -> Operand8 {
        Operand8::Register(*self)
    }
}

pub struct Disassembler<'a>(pub Box<Fn() -> u8 + 'a>);

impl<'a> Disassembler<'a> {
    fn next8(&mut self) -> u8 {
        (self.0)()
    }

    fn next16(&mut self) -> u16 {
        let l = self.next8();
        let h = self.next8();

        ((h as u16) << 8) | (l as u16)
    }
}

impl<'a> Operations for Disassembler<'a> {
    type Output = Instruction;

    fn nop(&mut self) -> Instruction {
        Instruction::Nop
    }

    fn load8<I: In8, O: Out8>(&mut self, dst: O, src: I) -> Instruction {
        Instruction::Load8(dst.to_operand8(self), src.to_operand8(self))
    }

    fn load16_immediate(&mut self, r: Register16) -> Instruction {
        Instruction::Load16Immediate(r, self.next16())
    }

    fn jp(&mut self) -> Instruction {
        Instruction::Jp(self.next16())
    }

    fn and<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::And(io.to_operand8(self))
    }

    fn or<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::Or(io.to_operand8(self))
    }

    fn xor<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::Xor(io.to_operand8(self))
    }

    fn undefined(&mut self, opcode: u8) -> Instruction {
        Instruction::Undefined(opcode)
    }
}

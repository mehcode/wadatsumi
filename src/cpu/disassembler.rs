use super::operations::Operations;
use super::io::{In8, Out8};
use super::instruction::{Condition as InstrCondition, Data16, Data8, Instruction, Operand8,
                         SignedData8};
use super::tracer::BusTracer;
use super::operands::{Address, Condition, Immediate8, Register16, Register8};
use super::super::bus::Bus;

pub trait IntoCondition {
    fn into_condition(self) -> Option<InstrCondition>;
}

pub trait IntoOperand8 {
    fn into_operand8(self, disassembler: &mut Disassembler) -> Operand8;
}

impl IntoOperand8 for Address {
    fn into_operand8(self, _: &mut Disassembler) -> Operand8 {
        Operand8::Memory(self)
    }
}

impl IntoOperand8 for Immediate8 {
    fn into_operand8(self, disassembler: &mut Disassembler) -> Operand8 {
        Operand8::Immediate(disassembler.next8())
    }
}

impl IntoOperand8 for Register8 {
    fn into_operand8(self, _: &mut Disassembler) -> Operand8 {
        Operand8::Register(self)
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
        Instruction::Load8(dst.into_operand8(self), src.into_operand8(self))
    }

    fn load16_immediate(&mut self, r: Register16) -> Instruction {
        Instruction::Load16Immediate(r, Data16(self.next16()))
    }

    fn inc8<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::Increment8(io.into_operand8(self))
    }

    fn dec8<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::Decrement8(io.into_operand8(self))
    }

    fn add<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::Add(src.into_operand8(self))
    }

    fn compare<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::Compare(src.into_operand8(self))
    }

    fn and<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::And(src.into_operand8(self))
    }

    fn or<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::Or(src.into_operand8(self))
    }

    fn xor<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::Xor(src.into_operand8(self))
    }

    fn jr<C: Condition>(&mut self, cond: C) -> Instruction {
        Instruction::JumpRelative(cond.into_condition(), SignedData8(self.next8() as i8))
    }

    fn jp<C: Condition>(&mut self, cond: C) -> Instruction {
        Instruction::Jump(cond.into_condition(), Data16(self.next16()))
    }

    fn call<C: Condition>(&mut self, cond: C) -> Instruction {
        Instruction::Call(cond.into_condition(), Data16(self.next16()))
    }

    fn ret<C: Condition>(&mut self, cond: C) -> Instruction {
        Instruction::Return(cond.into_condition())
    }

    fn reti(&mut self) -> Instruction {
        Instruction::ReturnAndEnableInterrupts
    }

    fn ei(&mut self) -> Instruction {
        Instruction::EnableInterrupts
    }

    fn di(&mut self) -> Instruction {
        Instruction::DisableInterrupts
    }

    fn reset(&mut self, address: u8) -> Instruction {
        Instruction::Reset(Data8(address))
    }

    fn undefined(&mut self, opcode: u8) -> Instruction {
        Instruction::Undefined(Data8(opcode))
    }
}

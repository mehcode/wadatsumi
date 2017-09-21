use super::operations::Operations;
use super::io::{In16, In8, Out16, Out8};
use super::instruction::{Address, Condition as InstrCondition, Data16, Data8, Instruction,
                         Operand16, Operand8, SignedData8};
use super::tracer::BusTracer;
use super::operands::{Address as OperAddress, Condition, Immediate16, Immediate8, Register16,
                      Register8};
use super::super::bus::Bus;

pub trait IntoCondition {
    fn into_condition(self) -> Option<InstrCondition>;
}

pub trait IntoAddress {
    fn into_address(self, disassembler: &mut Disassembler) -> Address;
}

impl IntoAddress for OperAddress {
    fn into_address(self, disassembler: &mut Disassembler) -> Address {
        match self {
            OperAddress::Direct => Address::Direct(Data16(disassembler.next16())),
            OperAddress::BC => Address::BC,
            OperAddress::DE => Address::DE,
            OperAddress::HL => Address::HL,
            OperAddress::ZeroPage => Address::ZeroPage(Data8(disassembler.next8())),
            OperAddress::ZeroPageC => Address::ZeroPageC,
            OperAddress::HLD => Address::HLD,
            OperAddress::HLI => Address::HLI,
        }
    }
}

pub trait IntoOperand16 {
    fn into_operand16(self, disassembler: &mut Disassembler) -> Operand16;
}

impl IntoOperand16 for Immediate16 {
    fn into_operand16(self, disassembler: &mut Disassembler) -> Operand16 {
        Operand16::Immediate(Data16(disassembler.next16()))
    }
}

impl IntoOperand16 for OperAddress {
    fn into_operand16(self, disassembler: &mut Disassembler) -> Operand16 {
        Operand16::Memory(self.into_address(disassembler))
    }
}

impl IntoOperand16 for Register16 {
    fn into_operand16(self, _: &mut Disassembler) -> Operand16 {
        Operand16::Register(self)
    }
}

pub trait IntoOperand8 {
    fn into_operand8(self, disassembler: &mut Disassembler) -> Operand8;
}

impl IntoOperand8 for OperAddress {
    fn into_operand8(self, disassembler: &mut Disassembler) -> Operand8 {
        Operand8::Memory(self.into_address(disassembler))
    }
}

impl IntoOperand8 for Immediate8 {
    fn into_operand8(self, disassembler: &mut Disassembler) -> Operand8 {
        Operand8::Immediate(Data8(disassembler.next8()))
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

    fn load16<I: In16, O: Out16>(&mut self, dst: O, src: I) -> Instruction {
        Instruction::Load16(dst.into_operand16(self), src.into_operand16(self))
    }

    fn inc8<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::Increment8(io.into_operand8(self))
    }

    fn dec8<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::Decrement8(io.into_operand8(self))
    }

    fn inc16(&mut self, r: Register16) -> Instruction {
        Instruction::Increment16(r)
    }

    fn dec16(&mut self, r: Register16) -> Instruction {
        Instruction::Decrement16(r)
    }

    fn push16(&mut self, r: Register16) -> Instruction {
        Instruction::Push16(r)
    }

    fn pop16(&mut self, r: Register16) -> Instruction {
        Instruction::Pop16(r)
    }

    fn add16_hl(&mut self, r: Register16) -> Instruction {
        Instruction::Add16Hl(r)
    }

    fn add8<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::Add8(src.into_operand8(self))
    }

    fn adc8<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::AddWithCarry8(src.into_operand8(self))
    }

    fn sub<I: In8>(&mut self, src: I) -> Instruction {
        Instruction::Sub(src.into_operand8(self))
    }

    fn cp<I: In8>(&mut self, src: I) -> Instruction {
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

    fn jp<C: Condition>(&mut self, cond: C, address: OperAddress) -> Instruction {
        Instruction::Jump(cond.into_condition(), address.into_address(self))
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

    fn rla(&mut self) -> Instruction {
        Instruction::RotateAccumulatorLeft
    }

    fn rlca(&mut self) -> Instruction {
        Instruction::RotateAccumulatorLeftCircular
    }

    fn rra(&mut self) -> Instruction {
        Instruction::RotateAccumulatorRight
    }

    fn rrca(&mut self) -> Instruction {
        Instruction::RotateAccumulatorRightCircular
    }

    fn rl<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::RotateLeft(io.into_operand8(self))
    }

    fn rlc<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::RotateLeftCircular(io.into_operand8(self))
    }

    fn rr<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::RotateRight(io.into_operand8(self))
    }

    fn rrc<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::RotateRightCircular(io.into_operand8(self))
    }

    fn swap<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::ByteSwap(io.into_operand8(self))
    }

    fn sla<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::ShiftLeftA(io.into_operand8(self))
    }

    fn sra<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::ShiftRightA(io.into_operand8(self))
    }

    fn srl<IO: In8 + Out8>(&mut self, io: IO) -> Instruction {
        Instruction::ShiftRightL(io.into_operand8(self))
    }

    fn bit<I: In8>(&mut self, bit: u8, src: I) -> Instruction {
        Instruction::BitTest(bit, src.into_operand8(self))
    }

    fn set<IO: In8 + Out8>(&mut self, bit: u8, io: IO) -> Instruction {
        Instruction::BitSet(bit, io.into_operand8(self))
    }

    fn res<IO: In8 + Out8>(&mut self, bit: u8, io: IO) -> Instruction {
        Instruction::BitReset(bit, io.into_operand8(self))
    }

    fn rst(&mut self, address: u8) -> Instruction {
        Instruction::Reset(Data8(address))
    }

    fn cpl(&mut self) -> Instruction {
        Instruction::InvertA
    }

    fn ccf(&mut self) -> Instruction {
        Instruction::InvertCarry
    }

    fn scf(&mut self) -> Instruction {
        Instruction::SetCarry
    }

    fn undefined(&mut self, opcode: u8) -> Instruction {
        Instruction::Undefined(Data8(opcode))
    }
}

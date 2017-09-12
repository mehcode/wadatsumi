use super::operations::Operations;
use super::io::{In8, Out8};
use super::instruction::{Address, Instruction};
use super::tracer::BusTracer;
use super::super::bus::Bus;

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

    fn load8<I: In8, O: Out8>(&mut self, destination: O, source: I) -> Instruction {
        Instruction::Load8(destination.into(), source.into())
    }

    fn jp(&mut self) -> Instruction {
        Instruction::Jp(Address::Direct(self.next16()))
    }

    fn undefined(&mut self, opcode: u8) -> Instruction {
        Instruction::Undefined(opcode)
    }
}

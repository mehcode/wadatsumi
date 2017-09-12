use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::VecDeque;
use ansi_term::Colour;
use super::super::bus::Bus;
use super::executor::Executor;
use super::disassembler::Disassembler;
use super::operations::Operations;
use super::State;
use super::io::{In8, Out8};

pub struct BusTracer<'a, B: Bus + 'a> {
    inner: &'a mut B,
    read_buffer: Rc<RefCell<VecDeque<u8>>>,
}

impl<'a, B: Bus> BusTracer<'a, B> {
    pub fn new(inner: &'a mut B) -> Self {
        Self {
            inner,
            read_buffer: Default::default(),
        }
    }
}

impl<'a, B: Bus> Bus for BusTracer<'a, B> {
    fn read8(&self, address: u16) -> u8 {
        let value = self.inner.read8(address);
        self.read_buffer.borrow_mut().push_back(value);
        value
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.inner.write8(address, value)
    }
}

pub struct InstructionTracer<'a, B: Bus + 'a> {
    initial_pc: u16,
    executor: Executor<'a, BusTracer<'a, B>>,
    disassembler: Disassembler<'a>,
}

impl<'a, B: Bus> InstructionTracer<'a, B> {
    pub fn new(initial_pc: u16, executor: Executor<'a, BusTracer<'a, B>>) -> Self {
        let read_buffer = executor.1.read_buffer.clone();

        InstructionTracer {
            initial_pc,
            executor,
            disassembler: Disassembler(Box::new(move || {
                read_buffer.borrow_mut().pop_front().unwrap()
            })),
        }
    }
}

macro_rules! instr_trace {
    ($s:expr; $($e:tt)+) => {
        use ::cpu::registers::Register8::*;
        use ::cpu::registers::Register16::*;
        use ::cpu::io::{In8, In16};

        let output = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
            $s.executor.$($e)*
        }));

        let instr = $s.disassembler.$($e)*;

        let a = A.read8($s.executor.0, $s.executor.1);
        let bc = BC.read16($s.executor.0, $s.executor.1);
        let de = DE.read16($s.executor.0, $s.executor.1);
        let hl = HL.read16($s.executor.0, $s.executor.1);

        trace!("{}{:04x}{} {} {} {:04x} {} {:02x} {} {:04x} {} {:04x} {} {:04x} {} {}",
            Colour::Yellow.paint("["),
            $s.initial_pc,
            Colour::Yellow.paint("]"),
            Colour::Fixed(15).paint(format!("{:<25}", format!("{}", instr))),
            Colour::Yellow.paint("PC"),
            $s.executor.0.pc,
            Colour::Yellow.paint("A"),
            a,
            Colour::Yellow.paint("BC"),
            bc,
            Colour::Yellow.paint("DE"),
            de,
            Colour::Yellow.paint("HL"),
            hl,
            Colour::Yellow.paint("FLAGS"),
            "-nh-",
        );

        return output.unwrap_or_else(|_| ::std::process::exit(101));
    };
}

impl<'a, B: Bus> Operations for InstructionTracer<'a, B> {
    type Output = <Executor<'a, BusTracer<'a, B>> as Operations>::Output;

    fn nop(&mut self) -> Self::Output {
        instr_trace!(self; nop());
    }

    fn load8<I: In8, O: Out8>(&mut self, destination: O, source: I) -> Self::Output {
        instr_trace!(self; load8(destination, source));
    }

    fn jp(&mut self) -> Self::Output {
        instr_trace!(self; jp());
    }

    fn undefined(&mut self, opcode: u8) -> Self::Output {
        instr_trace!(self; undefined(opcode));
    }
}

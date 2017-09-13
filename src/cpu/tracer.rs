use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::VecDeque;
use ansi_term::Colour;
use super::super::bus::Bus;
use super::executor::Executor;
use super::disassembler::Disassembler;
use super::operations::Operations;
use super::State;
use super::operands::{Condition, Register16};
use super::io::{In16, In8, Out16, Out8};

pub struct BusTracer<'a, B: Bus + 'a> {
    inner: &'a mut B,
    read_buffer: Rc<RefCell<VecDeque<u8>>>,
}

impl<'a, B: Bus> BusTracer<'a, B> {
    pub fn new(inner: &'a mut B) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            inner,
            read_buffer: Default::default(),
        }))
    }
}

impl<'a, B: Bus> Bus for Rc<RefCell<BusTracer<'a, B>>> {
    fn read8(&self, address: u16) -> u8 {
        let value = self.borrow().inner.read8(address);

        {
            let self_mut = self.borrow_mut();
            self_mut.read_buffer.borrow_mut().push_back(value);
        }

        value
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.borrow_mut().inner.write8(address, value)
    }
}

pub struct InstructionTracer<'a, B: Bus + 'a> {
    initial_pc: u16,
    executor: Executor<'a, Rc<RefCell<BusTracer<'a, B>>>>,
    disassembler: Disassembler<'a>,
}

impl<'a, B: Bus> InstructionTracer<'a, B> {
    pub fn new(initial_pc: u16, pc: u16, executor: Executor<'a, Rc<RefCell<BusTracer<'a, B>>>>) -> Self {
        let bus_tracer = executor.1.clone();
        let unbuffered_read_counter = Rc::new(Cell::new(0));

        InstructionTracer {
            initial_pc,
            executor,
            disassembler: Disassembler(Box::new(
                move || {
                    let buffered_read = {
                        let self_mut = bus_tracer.borrow_mut();
                        let mut read_buffer = self_mut.read_buffer.borrow_mut();

                        read_buffer.pop_front()
                    };

                    buffered_read.unwrap_or_else(|| {
                        let offset = unbuffered_read_counter.get();
                        unbuffered_read_counter.set(offset + 1);

                        bus_tracer.borrow().inner.read8(pc + offset)
                    })
                },
            )),
        }
    }
}

macro_rules! instr_trace {
    ($s:expr; $($e:tt)+) => {
        use ::cpu::operands::Register8::*;
        use ::cpu::operands::Register16::*;
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
            ($s.executor.0).f,
        );

        return output.unwrap_or_else(|_| ::std::process::exit(101));
    };
}

impl<'a, B: Bus> Operations for InstructionTracer<'a, B> {
    type Output = <Executor<'a, Rc<RefCell<BusTracer<'a, B>>>> as Operations>::Output;

    fn nop(&mut self) -> Self::Output {
        instr_trace!(self; nop());
    }

    fn load8<I: In8, O: Out8>(&mut self, dst: O, src: I) -> Self::Output {
        instr_trace!(self; load8(dst, src));
    }

    fn load16_immediate(&mut self, r: Register16) -> Self::Output {
        instr_trace!(self; load16_immediate(r));
    }

    fn jp<C: Condition>(&mut self, cond: C) -> Self::Output {
        instr_trace!(self; jp(cond));
    }

    fn jr<C: Condition>(&mut self, cond: C) -> Self::Output {
        instr_trace!(self; jr(cond));
    }

    fn call<C: Condition>(&mut self, cond: C) -> Self::Output {
        instr_trace!(self; call(cond));
    }

    fn and<IO: In8 + Out8>(&mut self, io: IO) -> Self::Output {
        instr_trace!(self; and(io));
    }

    fn or<IO: In8 + Out8>(&mut self, io: IO) -> Self::Output {
        instr_trace!(self; or(io));
    }

    fn xor<IO: In8 + Out8>(&mut self, io: IO) -> Self::Output {
        instr_trace!(self; xor(io));
    }

    fn undefined(&mut self, opcode: u8) -> Self::Output {
        instr_trace!(self; undefined(opcode));
    }
}

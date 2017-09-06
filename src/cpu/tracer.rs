use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;

use super::super::bus::Bus;
use super::executor::Executor;
use super::disassembler::Disassembler;
use super::operations::Operations;
use super::State;
use super::io::{In8, Out8};

pub(super) struct BusTracer<'a, B: Bus + 'a> {
    inner: &'a mut B,
    pub(super) read_buffer: RefCell<VecDeque<u8>>,
}

impl<'a, B: Bus> BusTracer<'a, B> {
    #[cfg_attr(not(feature = "trace"), allow(unused))]
    pub(super) fn new(inner: &'a mut B) -> Self {
        Self { inner, read_buffer: Default::default() }
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

pub(super) struct InstructionTracer<'a, B: Bus + 'a>(
    Rc<RefCell<Executor<'a, BusTracer<'a, B>>>>,
    Disassembler<'a>,
);

impl<'a, B: Bus> InstructionTracer<'a, B> {
    pub(super) fn new(executor: Executor<'a, BusTracer<'a, B>>) -> Self {
        let executor = Rc::new(RefCell::new(executor));

        InstructionTracer(
            executor.clone(),
            Disassembler(Box::new(move || {
                let executor = executor.borrow_mut();
                let bus_tracer = &executor.1;
                let result = bus_tracer.read_buffer.borrow_mut().pop_front().unwrap();

                result
            })),
        )
    }
}

macro_rules! instr_trace {
    ($s:expr; $($e:tt)+) => {
        let output = ($s.0).borrow_mut().$($e)*;
        let instr = ($s.1).$($e)*;

        trace!("{:<25} PC #{:04x}", format!("{}", instr), ($s.0.borrow()).0.pc);

        return output;
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

    #[inline(always)]
    fn jp(&mut self) -> Self::Output {
        instr_trace!(self; jp());
    }

    #[inline(always)]
    fn undefined(&mut self, opcode: u8) -> Self::Output {
        instr_trace!(self; undefined(opcode));
    }
}

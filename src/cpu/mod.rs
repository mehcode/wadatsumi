mod operands;
mod state;
mod operations;
mod io;
mod executor;
mod instruction;
mod tracer;
mod disassembler;

pub use self::state::State;

use log::LogLevel::Trace;
use super::bus::Bus;
use self::operations::Operations;
use self::executor::Executor;

/// Interpreter for the Sharp LR35902, the Nintendo® Game Boy CPU.
#[derive(Default)]
pub struct Cpu {
    state: State,
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.state.pc = 0x100;
    }

    /// Run the _next_ instruction.
    pub fn run_next<B: Bus>(&mut self, bus: &mut B) {
        // Capture the initial PC (used for tracing)
        let pc = self.state.pc;

        // Fetch the opcode (and increment PC)
        let opcode = self.state.next8(bus);

        if log_enabled!(Trace) {
            // TODO(@rust): It'd be nice to move `BusTracer::new` into
            //              `InstructionTracer::new` but I can't get the lifetimes to be happy that way

            // Wrap the Bus in a `BusTracer`. This will buffer reads so we can retrieve them
            // at the end of the instruction. We do this to achieve an accurate
            // instruction decoding (properly reflecting timing).
            let mut bus = tracer::BusTracer::new(bus);
            let executor = Executor(&mut self.state, &mut bus);

            // Wrap our executor in an `InstructionTracer`. This will access the `BusTracer`
            // for values and produce `trace!` statements.
            let visitor = tracer::InstructionTracer::new(pc, executor);
            operations::visit(visitor, opcode)
        } else {
            // Directly execute the instruction
            operations::visit(Executor(&mut self.state, bus), opcode)
        }
    }
}

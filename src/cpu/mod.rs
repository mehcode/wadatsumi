mod registers;
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
        // Fetch the opcode (and increment PC)
        // TODO: Dedicated .next_ fn
        let opcode = bus.read8(self.state.pc);
        self.state.pc += 1;

        if log_enabled!(Trace) {
            // Wrap the Bus in a `BusTracer`. This will buffer reads so we can retrieve them
            // at the end of the instruction. We do this to achieve an accurate
            // instruction decoding (properly reflecting timing).
            let mut bus = tracer::BusTracer::new(bus);
            let executor = Executor(&mut self.state, &mut bus);

            // Wrap our executor in an `InstructionTracer`. This will access the `BusTracer`
            // for values and produce `trace!` statements.
            let visitor = tracer::InstructionTracer::new(executor);
            operations::visit(visitor, opcode)
        } else {
            // Directly execute the instruction
            operations::visit(Executor(&mut self.state, bus), opcode)
        }
    }
}

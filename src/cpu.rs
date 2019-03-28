use crate::state::State;
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::decode::decode;
use crate::execute::execute;

#[derive(Default)]
pub struct Cpu {
    pub state: State,

    pipeline_index: u8,
    opcode_pipeline: [Option<u32>; 3],
    instruction_pipeline: [Option<Instruction>; 3],
}

impl Cpu {
    pub fn run_next(&mut self, mem: &mut Memory) {
        self.fetch(mem);
        self.decode();
        self.execute(mem);

        // todo: handle interrupts

        if self.state.needs_pipeline_flush {
            self.flush_pipeline();
        } else {
            // Update program counter (instruction pointer) and increment pipeline stage
            // todo: self.state.r15 += (arm_mode == ARM) ? 4 : 2;
            self.state.r15 += 4;
            self.pipeline_index = (self.pipeline_index + 1) % 3;
        }
    }

    pub fn fetch(&mut self, mem: &mut Memory) {
        let index = self.pipeline_index as usize;

        // Read 32-bit ARM instruction
        self.opcode_pipeline[index] = Some(mem.read_u32(self.state.r15));

        // Clear the decoded instruction
        self.instruction_pipeline[index] = None;
    }

    pub fn decode(&mut self) {
        let index = ((self.pipeline_index + 2) % 3) as usize;

        if let Some(opcode) = self.opcode_pipeline[index] {
            self.instruction_pipeline[index] = Some(decode(opcode));
        }
    }

    pub fn execute(&mut self, mem: &mut Memory) {
        let index = ((self.pipeline_index + 1) % 3) as usize;

        if let Some(ix) = self.instruction_pipeline[index] {
            // todo: -8 isn't good enough for THUMB
            println!("execute [0x{:x}] {}", self.state.r15 - 8, ix);

            execute(ix, &mut self.state, mem);
        }
    }

    pub fn flush_pipeline(&mut self) {
        self.state.needs_pipeline_flush = false;
        self.pipeline_index = 0;

        // my kingdom for `slice.fill(None)`

        self.instruction_pipeline[0] = None;
        self.instruction_pipeline[1] = None;
        self.instruction_pipeline[2] = None;

        self.opcode_pipeline[0] = None;
        self.opcode_pipeline[1] = None;
        self.opcode_pipeline[2] = None;
    }
}

use crate::state::State;
use crate::memory::Memory;
use crate::instruction::DataProcessingOperation;
use crate::instruction::Instruction;

pub fn execute(ix: Instruction, state: &mut State, mem: &mut Memory) {
    match ix {
        Instruction::Branch {
            cond, link, offset,
        } => {
            // todo: condition
            // todo: link

            state.r15 = state.r15.wrapping_add(offset);
            state.needs_pipeline_flush = true;
        }

        Instruction::DataProcessing {
            cond,
            set_cond,
            op,
            n,
            d,
            o,
        } => {
            // todo: condition
            // todo: set condition
            match op {
                DataProcessingOperation::Move => {
                    *state.r_mut(d) = o.value(state);
                },

                _ => unimplemented!("execute data processing operation: {:?}", op)
            }
        }

        Instruction::StatusTransferFrom {
            cond,
            current,
            write_flags,
            write_control,
            o,
        } => {
            // todo: condition
            // todo: write_control
            let value = o.value(state);

            if write_flags {
                state.cpsr ^= !0xF8000000;
                state.cpsr |= (value & 0xF8000000);
            }
        }

        _ => {
            println!("execute unhandled instruction: {}", ix)
        }
    }
}

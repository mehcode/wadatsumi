use crate::state::State;
use crate::memory::Memory;
use crate::instruction::{DataProcessingOperation, Unit};
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

                DataProcessingOperation::MoveNegative => {
                    *state.r_mut(d) = 0xffffffff ^ o.value(state);
                }

                _ => unimplemented!("execute data processing operation: {:?}", op)
            }

            println!("{} = {:x}", d, *state.r(d));
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

        // fixme: consider separating loads and stores here
        Instruction::SingleDataTransfer {
            cond,
            unit,
            // fixme: this means "sign extend the loaded data"; should be not set for stores
            signed,
            write_back,
            // 1 = pre index; 0 = post index
            pre_post_indexing,
            // fixme: if set, this is a load; otherwise, store
            store_load,
            // 1 = add offset to base; 0 = subtract offset from base
            up_down,
            offset,
            d,
            n,
        } => {
            // todo: condition
            // todo: prevent R15 & write_back

            if unit != Unit::Word {
                unimplemented!("unhandled load/store non-word : {:?}", unit);
            }

            let base = *state.r(n);
            let offset = offset.value(state);
            let address = if up_down {
                base.wrapping_add(offset)
            } else {
                base.wrapping_sub(offset)
            };

            let transfer_address = if pre_post_indexing {
                address
            } else {
                base
            };

            if store_load {
                // 1 = Load from memory
                *state.r_mut(d) = mem.read_u32(transfer_address);
            } else {
                // 0 = Store to memory
                mem.write_u32(transfer_address, *state.r(d));
            }

            if write_back {
                *state.r_mut(n) = address;
            }
        }

        _ => {
            unimplemented!("execute unhandled instruction: {}", ix)
        }
    }
}

use crate::{instruction::arm::ArmInstruction, state::State, util::int::IntExt};

impl ArmInstruction {
    pub fn execute(self, state: &mut State, opcode: u32) {
        // TODO: Check condition up front

        (match self {
            ArmInstruction::BranchLink => &Self::exec_branch_link,

            _ => unimplemented!("execute: {:?}", self),
        })(self, state, opcode);
    }

    fn exec_branch_link(self, state: &mut State, opcode: u32) {
        // [Bits 0 - 23] Offset
        let mut offset = opcode & 0xffffff;

        // Branch instruction contains a signed 2's complement 24-bit offset. This is shifted
        // left two bits, sign extended to 32-bits, and added to PC.

        offset <<= 2;

        if offset.bit(25) {
            offset |= 0xfc_00_00_00;
        }

        let address = state.r(15).wrapping_add(offset);

        // [Bit 24] Link
        let is_link = opcode.bit(24);

        // todo: clock: 1N

        if is_link {
            // Branch with Link (BL) writes the old PC into the link register.
            // The PC value is adjusted to allow for prefetch and contains the address of
            // the instruction _following_ the branch instruction.

            state.set_r(14, address - 4);
        }

        state.set_r(15, address);
        // todo: state.pipeline_needs_flush = true;

        // todo: clock: 2S
    }
}

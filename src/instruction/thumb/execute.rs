use crate::{
    instruction::thumb::ThumbInstruction,
    state::{State, CPSR_C_FLAG, CPSR_N_FLAG, CPSR_Z_FLAG},
    util::int::IntExt,
};
use bitintr::Bextr;

impl ThumbInstruction {
    pub fn execute(self, state: &mut State, opcode: u16) {
        (match self {
            ThumbInstruction::MoveShiftedRegister => &Self::exec_move_shifted_register,

            _ => unimplemented!("execute: {:?}", self),
        })(self, state, opcode);
    }

    fn exec_move_shifted_register(self, state: &mut State, opcode: u16) {
        // [Bits 0-2] Destination register
        let dst = (opcode & 0b111) as usize;

        // [Bits 3-5] Source register
        let src = state.r(opcode.bextr(3, 3) as usize);

        // [Bits 6-10] Shift offset
        let offset = opcode.bextr(6, 5) as u32;

        // [Bits 11-12] Shift opcode
        let op = opcode.bextr(11, 2);
        let (value, carry) = match op {
            // Logical shift left (LSL)
            0b00 => (src << offset, src.bit(31)),

            // Logical shift right (LSR)
            0b01 => (src >> offset, src.bit(0)),

            // Arithmetic shift right (ASR)
            0b10 => (((src as i32) >> offset) as u32, src.bit(0)),

            _ => unreachable!(),
        };

        state.set_r(dst, value);

        state.cpsr.set_mask_from(CPSR_Z_FLAG, value == 0);
        state.cpsr.set_mask_from(CPSR_N_FLAG, value.bit(31));
        state.cpsr.set_mask_from(CPSR_C_FLAG, carry);

        // TODO: clock: 1S
    }
}

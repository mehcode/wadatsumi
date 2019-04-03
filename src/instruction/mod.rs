mod arm;
mod thumb;

use self::{arm::ArmInstruction, thumb::ThumbInstruction};

#[derive(Debug)]
pub enum Instruction {
    Arm(u32, ArmInstruction),
    Thumb(u16, ThumbInstruction),
}

use crate::opcode::Opcode;

#[derive(Debug)]
pub enum Instruction {
    ClearScreen,
    Load { x: u8, kk: u8 },
}

impl Instruction {
    pub fn decode(opcode: Opcode) -> anyhow::Result<Self> {
        Ok(match opcode.digits() {
            (0x0, 0x0, 0xE, 0) => Self::ClearScreen,
            (0x6, x, _, _) => Self::Load { x, kk: opcode.lo },

            _ => anyhow::bail!("unknown opcode {}", opcode),
        })
    }
}

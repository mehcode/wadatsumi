use crate::opcode::Opcode;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Instruction {
    /// Clear the display (`00E0`).
    ClearScreen,

    /// Return from a subroutine (`00EE`).
    ///
    /// Sets the program counter (PC) to the address at the top of the stack, then
    /// subtracts one from the stack pointer (SP).
    Return,

    /// Jump to location `nnn` (`1nnn`).
    ///
    /// Sets the program counter (PC) to `nnn`.
    Jump { nnn: u16 },

    /// Call the subroutine at `nnn` (`2nnn`).
    ///
    /// Increments the stack pointer (SP), the puts the current program counter (PC) on
    /// the top of the stack. The PC is then set to `nnn`.
    Call { nnn: u16 },

    /// Skip next instruction if `Vx` is equal to `kk` (`3xkk`).
    ///
    /// Compares register `Vx` to `kk`, and if they are equal, increments the program counter (PC)
    /// by two.
    SkipIfEqualToValue { x: u8, kk: u8 },

    /// Skip next instruction if `Vx` is not equal to `kk` (`4xkk`).
    ///
    /// Compares register `Vx` to `kk`, and if they are not equal, increments the
    /// program counter (PC) by two.
    SkipIfNotEqualToValue { x: u8, kk: u8 },

    /// Skip next instruction if `Vx` is equal to `Vy` (`5xy0`).
    ///
    /// Compares register `Vx` to `Vy`, and if they are equal, increments the
    /// program counter (PC) by two.
    SkipIfEqualToRegister { x: u8, y: u8 },

    /// Skip next instruction if `Vx` is not equal to `Vy` (`9xy0`).
    ///
    /// Compares register `Vx` to `Vy`, and if they are not equal, increments the
    /// program counter (PC) by two.
    SkipIfNotEqualToRegister { x: u8, y: u8 },

    /// Puts the value `kk` into the register `Vx` (`6xkk`).
    LoadRegister { x: u8, kk: u8 },

    /// Sets `Vx` to the result of `Vx` and `kk` (`7xkk`).
    AddValue { x: u8, kk: u8 },

    /// Puts the value of register `Vy` into the register `Vx` (`8xy0`).
    CopyRegister { x: u8, y: u8 },

    /// Sets `Vx` to the bitwise OR of `Vx` and `Vy` (`8xy1`).
    Or { x: u8, y: u8 },

    /// Sets `Vx` to the bitwise AND of `Vx` and `Vy` (`8xy2`).
    And { x: u8, y: u8 },

    /// Sets `Vx` to the bitwise exclusive OR of `Vx` and `Vy` (`8xy3`).
    Xor { x: u8, y: u8 },

    /// Sets `Vx` to the result of adding `Vx` and `Vy` together (`8xy4`).
    Add { x: u8, y: u8 },

    /// Sets `Vx` to the result of subtracting `Vy` from `Vx` (`8xy5`).
    Subtract { x: u8, y: u8 },

    /// Sets `Vx` to the result of subtracting `Vx` from `Vy` (`8xy7`).
    SubtractFrom { x: u8, y: u8 },

    /// Divides `Vx` by 2 (`8x_6`).
    ShiftRight { x: u8 },

    /// Multiples `Vx` by 2 (`8x_E`).
    ShiftLeft { x: u8 },

    /// Sets the Index (I) register to the value `nnn` (`Annn`).
    LoadIndex { nnn: u16 },

    /// Jump to location `nnn` offset by `V0` (`Bnnn`).
    /// 
    /// Sets the program counter (PC) to `nnn` plus the value of `V0`.
    JumpWithOffset { nnn: u16 },

    /// Sets `Vx` to the bitwise AND of a random number from 0 to 255 and `kk` (`Cxkk`).
    Random { x: u8, kk: u8 },

    /// Draw (`Dxyn`).
    Draw { x: u8, y: u8, n: u8 },
}

impl Instruction {
    pub fn decode(opcode: Opcode) -> anyhow::Result<Self> {
        Ok(match opcode.digits() {
            (0x0, 0x0, 0xE, 0x0) => Self::ClearScreen,
            (0x0, 0x0, 0xE, 0xE) => Self::Return,
            (0x1, _, _, _) => Self::Jump { nnn: opcode.nnn() },
            (0x2, _, _, _) => Self::Call { nnn: opcode.nnn() },
            (0x3, x, _, _) => Self::SkipIfEqualToValue { x, kk: opcode.kk() },
            (0x4, x, _, _) => Self::SkipIfNotEqualToValue { x, kk: opcode.kk() },
            (0x5, x, y, _) => Self::SkipIfEqualToRegister { x, y },
            (0x6, x, _, _) => Self::LoadRegister { x, kk: opcode.kk() },
            (0x7, x, _, _) => Self::AddValue { x, kk: opcode.kk() },
            (0x8, x, y, 5) => Self::Subtract { x, y },

            _ => anyhow::bail!("unknown opcode {}", opcode),
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClearScreen => write!(f, "CLS"),
            Self::Return => write!(f, "RET"),
            Self::Jump { nnn } => write!(f, "JP {nnn:X}"),
            Self::Call { nnn } => write!(f, "CALL {nnn:X}"),
            Self::SkipIfEqualToValue { x, kk } => write!(f, "SE V{x:X} {kk:X}"),
            Self::SkipIfNotEqualToValue { x, kk } => write!(f, "SNE V{x:X} {kk:X}"),
            Self::SkipIfEqualToRegister { x, y } => write!(f, "SE V{x:X} V{y:X}"),
            Self::LoadRegister { x, kk } => write!(f, "LD V{x:X}, {kk:X}"),
            Self::AddValue { x, kk } => write!(f, "ADD V{x:X}, {kk:X}"),

            _ => unimplemented!(),
        }
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use std::marker::ConstParamTy;
use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::state::CpuStatus;
use crate::cpu::{Cpu, CpuState};

mod arithmetic;
mod flow;
mod logical;
mod stack;
mod system;
mod transfer;

pub use arithmetic::{ADC, CMP, CPX, CPY, DCP, DEC, DEX, DEY, INC, INX, INY, ISC, SBC};
pub use flow::{BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS, JMP, JSR, RTI, RTS};
pub use logical::{ALR, ANC, AND, ARR, ASL, BIT, EOR, LSR, ORA, RLA, ROL, ROR, RRA, SLO, SRE};
pub use stack::{PHA, PHP, PLA, PLP};
pub use system::{CLC, CLD, CLI, CLV, NOP, SEC, SED, SEI};
pub use transfer::{
    LAX, LDA, LDX, LDY, LXA, SAX, SHA, SHX, SHY, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
};

/// Classifies how an operation accesses memory, driving the addressing-mode pipeline.
/// The addressing mode uses this to issue the correct read or write cycles before handing off to `apply`.
#[derive(Debug, Clone, Copy)]
pub enum MemoryAccess {
    Read,
    Write,
    ReadModifyWrite,
}

/// The effect of a single 2A03 instruction mnemonic.
///
/// An `Operation` is the second half of instruction dispatch, after an
/// [`AddressingMode`] resolves the effective address into `cpu.address`,
/// the operation applies the mnemonic's effect to CPU and bus state.
pub trait Operation {
    const ACCESS: Option<MemoryAccess> = None;

    /// Applies the operation's effect for the current cycle.
    ///
    /// Called only after the addressing mode has fully resolved. `cpu.address`
    /// holds the effective address. Returns `Poll::Ready(())` when the operation is
    /// complete, signalling the CPU to clear the in-flight instruction.
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()>
    where
        Self: Sized;
}

/// A CPU register (`A`, `X`, `Y`, or `SP`).
#[derive(Debug, Clone, Copy, ConstParamTy, PartialEq, Eq)]
pub enum Register {
    A,
    X,
    Y,
    SP,
}

impl Register {
    /// Reads the value of this register.
    #[inline(always)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    pub const fn get(self, cpu: &CpuState) -> u8 {
        match self {
            Self::A => cpu.a,
            Self::X => cpu.x,
            Self::Y => cpu.y,
            Self::SP => cpu.sp,
        }
    }

    /// Writes `value` to this register.
    #[inline(always)]
    pub const fn set(self, cpu: &mut CpuState, value: u8) {
        match self {
            Self::A => {
                cpu.a = value;
            }

            Self::X => {
                cpu.x = value;
            }

            Self::Y => {
                cpu.y = value;
            }

            Self::SP => {
                cpu.sp = value;
            }
        }
    }
}

/// The source or destination of an operation's data,
/// either a CPU register or a memory location.
///
/// Used as a const generic parameter so register and memory variants of the same mnemonic
/// (e.g. `INX`/`INY` vs `INC`) can share a single generic implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum Operand {
    /// A CPU register (`A`, `X`, `Y`, or `SP`).
    Register(Register),

    /// The effective address resolved by the addressing mode.
    Memory,
}

impl Operand {
    /// Returns `Some(access)` for memory operands, `None` for register operands.
    /// Suitable for use in `Operation::ACCESS` to drive the addressing-mode pipeline.
    #[inline(always)]
    #[must_use]
    pub const fn access(self, access: MemoryAccess) -> Option<MemoryAccess> {
        match self {
            Self::Register(_) => None,
            Self::Memory => Some(access),
        }
    }

    /// Reads the current value of this operand.
    #[inline(always)]
    #[must_use]
    pub const fn read<B: Bus>(self, cpu: &Cpu<B>) -> u8 {
        match self {
            Self::Register(r) => r.get(&cpu.state),
            Self::Memory => cpu.data,
        }
    }

    /// Writes `value` to this operand.
    #[inline(always)]
    pub fn write<B: Bus>(self, cpu: &mut Cpu<B>, bus: &mut B, value: u8) {
        match self {
            Self::Register(r) => {
                r.set(&mut cpu.state, value);
            }

            Self::Memory => {
                bus.write(cpu.address, value);
            }
        }
    }
}

/// A floating internal bus value OR'd into the accumulator before the AND in
/// unstable immediate-mode opcodes like LXA and XAA. Its true value depends on chip revision,
/// temperature, and board capacitance, making these instructions non-deterministic on real hardware.
/// 0xFF is the value that produces correct results against nestest and most practical test ROMs.
const MAGIC: u8 = 0xff;

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use std::marker::ConstParamTy;
use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::Cpu2A03;

mod arithmetic;
mod flow;
mod logical;
mod stack;
mod system;
mod transfer;

pub use arithmetic::{ADC, CMP, CPX, CPY, DCP, DEC, DEX, DEY, INC, INX, INY, ISC, SBC, SBX};
pub use flow::{BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS, JMP, JSR, RTI, RTS};
pub use logical::{ALR, ANC, AND, ARR, ASL, BIT, EOR, LSR, ORA, RLA, ROL, ROR, RRA, SLO, SRE};
pub use stack::{PHA, PHP, PLA, PLP};
pub use system::{BRK, CLC, CLD, CLI, CLV, NOP, SEC, SED, SEI};
pub use transfer::{
    LAS, LAX, LDA, LDX, LDY, LXA, SAX, SHA, SHX, SHY, STA, STX, STY, TAS, TAX, TAY, TSX, TXA, TXS,
    TYA, XAA,
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
    fn apply<B: Bus>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()>
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
    pub const fn get(self, cpu: &Cpu2A03) -> u8 {
        match self {
            Self::A => cpu.a,
            Self::X => cpu.x,
            Self::Y => cpu.y,
            Self::SP => cpu.sp,
        }
    }

    /// Writes `value` to this register.
    #[inline(always)]
    pub const fn set(self, cpu: &mut Cpu2A03, value: u8) {
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
    pub const fn read(self, cpu: &Cpu2A03) -> u8 {
        match self {
            Self::Register(r) => r.get(cpu),
            Self::Memory => cpu.data,
        }
    }

    /// Writes `value` to this operand.
    #[inline(always)]
    pub fn write<B: Bus>(self, cpu: &mut Cpu2A03, bus: &mut B, value: u8) {
        match self {
            Self::Register(r) => {
                r.set(cpu, value);
            }

            Self::Memory => {
                bus.write(cpu.address(), value);
            }
        }
    }

    /// Latches `value` into `cpu.data` when this operand is [`Operand::Memory`].
    ///
    /// Used by shift and rotate operations so that composed undocumented ops (SLO, SRE, RLA,
    /// RRA) can read the modified value from `cpu.data` without an extra bus access. For
    /// register variants this is a no-op; `O` is a const generic so LLVM eliminates the branch
    /// at compile time with zero overhead on the register-operand hot paths (ASL A, LSR A, etc.).
    #[inline(always)]
    pub const fn latch(self, cpu: &mut Cpu2A03, value: u8) {
        if matches!(self, Self::Memory) {
            cpu.data = value;
        }
    }
}

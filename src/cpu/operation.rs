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

pub use arithmetic::{ADC, CMP, CPX, CPY, DEC, DEX, DEY, INC, INX, INY, SBC};
pub use flow::{BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS, JMP, JSR, RTI, RTS};
pub use logical::{AND, BIT, EOR, ORA};
pub use stack::{PHA, PHP, PLA, PLP};
pub use system::{CLC, CLD, CLI, CLV, NOP, SEC, SED, SEI};
pub use transfer::{LDA, LDX, LDY, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA};

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

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CPU system operations: status flag manipulation (`CLC`, `SEC`, `CLI`, `SEI`,
//! `CLD`, `SED`, `CLV`) and the no-op (`NOP`). Nothing here touches general-purpose
//! registers or memory. `BRK` belongs here when added.

use std::task::Poll;

use crate::Bus;
use crate::cpu::operation::{MemoryAccess, Operation};
use crate::cpu::{Cpu, CpuStatus};

/// Clears status flag `FLAG` unconditionally (`CLC`, `CLI`, `CLD`, `CLV`).
/// Executes in a single implicit cycle after the opcode fetch.
/// No memory access occurs and no other flags are affected.
pub struct CLEAR<const FLAG: CpuStatus>;

impl<const FLAG: CpuStatus> Operation for CLEAR<FLAG> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        cpu.state.p.remove(FLAG);

        Poll::Ready(())
    }
}

pub type CLC = CLEAR<{ CpuStatus::C }>;
pub type CLI = CLEAR<{ CpuStatus::I }>;
pub type CLD = CLEAR<{ CpuStatus::D }>;
pub type CLV = CLEAR<{ CpuStatus::V }>;

/// No operation; idles for one additional cycle after the opcode fetch.
/// All registers and flags are left unchanged.
/// Commonly used for cycle-padding or dead-code patching.
pub struct NOP;

impl Operation for NOP {
    // Read prevents Absolute's JMP shortcut (which fires when ACCESS is None) from
    // short-circuiting at t=2; without it, NOP Absolute/Absolute,X would take one cycle too few.
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(_: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        // Do nothing
        Poll::Ready(())
    }
}

/// Sets status flag `FLAG` unconditionally (`SEC`, `SEI`, `SED`).
/// Executes in a single implicit cycle after the opcode fetch.
/// No memory access occurs and no other flags are affected.
pub struct SET<const FLAG: CpuStatus>;

impl<const FLAG: CpuStatus> Operation for SET<FLAG> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        cpu.state.p.insert(FLAG);

        Poll::Ready(())
    }
}

pub type SEC = SET<{ CpuStatus::C }>;
pub type SEI = SET<{ CpuStatus::I }>;
pub type SED = SET<{ CpuStatus::D }>;

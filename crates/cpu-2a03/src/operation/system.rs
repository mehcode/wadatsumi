// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CPU system operations: software interrupt (`BRK`), status flag manipulation
//! (`CLC`, `SEC`, `CLI`, `SEI`, `CLD`, `SED`, `CLV`), and the no-op (`NOP`).
//! Nothing here touches general-purpose registers or memory.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::interrupt::{InterruptKind, interrupt};
use crate::operation::{MemoryAccess, Operation};
use crate::status::CpuStatus;

/// Triggers a software interrupt; pushes the return address (`PC + 2`, the
/// byte after the padding) and the processor status (with both `B` and `U`
/// set so the handler can tell BRK from a hardware IRQ), sets the `I` flag
/// to mask further IRQs while in the handler, and loads PC from the IRQ/BRK
/// vector at `$FFFE/F`. 7 cycles, [`Implied`][crate::addressing] addressing.
///
/// The per-cycle body lives in [`interrupt`] and is shared with the hardware
/// IRQ and NMI paths; this struct is just the [`Operation`] wrapper that
/// hooks BRK into the opcode dispatch table.
pub struct BRK;

impl Operation for BRK {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        interrupt::<{ InterruptKind::BRK }, B>(cpu, bus)
    }
}

/// Clears status flag `FLAG` unconditionally (`CLC`, `CLI`, `CLD`, `CLV`).
/// Executes in a single implicit cycle after the opcode fetch.
/// No memory access occurs and no other flags are affected.
pub struct CLEAR<const FLAG: CpuStatus>;

impl<const FLAG: CpuStatus> Operation for CLEAR<FLAG> {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        cpu.p.remove(FLAG);

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
    fn apply<B: CpuReadWrite>(_: &mut Cpu2A03, _: &mut B) -> Poll<()> {
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
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        cpu.p.insert(FLAG);

        Poll::Ready(())
    }
}

pub type SEC = SET<{ CpuStatus::C }>;
pub type SEI = SET<{ CpuStatus::I }>;
pub type SED = SET<{ CpuStatus::D }>;

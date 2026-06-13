// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CPU system operations: the software interrupt (`BRK`), explicit status-flag
//! manipulation (`CLC`, `SEC`, `CLI`, `SEI`, `CLD`, `SED`, `CLV`), and the no-op (`NOP`).
//!
//! Nothing here touches general-purpose registers or memory, the flag setters run as
//! single-cycle implied ops and `BRK` only interacts with the stack and the IRQ/BRK
//! vector.
//!
//! See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for the per-instruction
//! reference.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::interrupt::{InterruptKind, interrupt};
use crate::operation::{MemoryAccess, Operation};
use crate::status::CpuStatus;

/// Triggers a software interrupt (`BRK`). 7 cycles, implied addressing.
///
/// `BRK` is a one-byte opcode but consumes two bytes of code, the byte after the opcode
/// is "padding" the hardware skips over. The pushed return address is `PC + 2` (one past
/// the padding), so an `RTI` lands cleanly on the next real instruction.
///
/// Push order matches a hardware IRQ:
///
/// 1. `PCH`
/// 2. `PCL`
/// 3. `P` with both `B` and `U` set, so the handler can distinguish `BRK` from a
///    hardware IRQ by inspecting the stacked `B` bit
///
/// `I` is set in the live `P` afterwards to mask further IRQs while in the handler, then
/// `PC` is loaded from the IRQ/BRK vector at `$FFFE/F`.
///
/// The per-cycle body lives in [`interrupt`] and is shared with the hardware IRQ and NMI
/// paths, this struct is just the [`Operation`] wrapper that hooks `BRK` into the opcode
/// dispatch table.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#BRK>.
pub struct BRK;

impl Operation for BRK {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        interrupt::<{ InterruptKind::BRK }, B>(cpu, bus)
    }
}

/// Clears status flag `FLAG` unconditionally (`CLC`, `CLI`, `CLD`, `CLV`).
///
/// Resolves in a single implicit cycle after the opcode fetch. No memory access, no
/// other flags touched. `CLD` is functionally a no-op on the 2A03 (decimal mode does not
/// exist) but the flag bit is still real and software does still set and clear it, so it
/// is dispatched the same as the others.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#CLC>.
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

/// No operation, idles for the cycles the addressing mode dictates and leaves every
/// register and flag unchanged (`NOP`).
///
/// Used by software for cycle-padding (raster-timed code, branch delay slots) and
/// dead-code patching. The 2A03 also has a swarm of undocumented `NOP` variants spread
/// across the opcode table at different addressing modes (zero-page, absolute,
/// absolute,X), all of which route here, the addressing mode determines how many
/// cycles they actually consume.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#NOP> and
/// <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
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
///
/// Resolves in a single implicit cycle after the opcode fetch. No memory access, no
/// other flags touched. `SED` sets a flag that has no functional effect on the 2A03
/// (decimal mode does not exist), but the bit still flips, so disassemblers and
/// debuggers can observe it.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#SEC>.
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

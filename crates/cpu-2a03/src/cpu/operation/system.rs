// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CPU system operations: status flag manipulation (`CLC`, `SEC`, `CLI`, `SEI`,
//! `CLD`, `SED`, `CLV`) and the no-op (`NOP`). Nothing here touches general-purpose
//! registers or memory. `BRK` belongs here when added.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::operation::{MemoryAccess, Operation};
use crate::cpu::{Cpu2A03, CpuStatus};

pub struct BRK;

impl Operation for BRK {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => {
                // Implied already read the padding byte spuriously; advance PC past it
                // so the return address pushed below is BRK+2, as the 6502 requires.
                cpu.pc = cpu.pc.wrapping_add(1);

                Poll::Pending
            }

            // Push PCH on stack, decrement S
            2 => {
                cpu.stack_push(bus, (cpu.pc >> 8) as u8);

                Poll::Pending
            }

            3 => {
                // Push PCL on stack, decrement S
                cpu.stack_push(bus, cpu.pc as u8);

                Poll::Pending
            }

            4 => {
                // Push P on stack with B and U always set, decrement S
                cpu.stack_push(bus, cpu.p.0 | CpuStatus::B | CpuStatus::U);

                Poll::Pending
            }

            5 => {
                // Fetch PCL; set I to suppress further IRQs while in the handler
                cpu.pc = u16::from(bus.read(0xfffe));
                cpu.p.insert(CpuStatus::I);

                Poll::Pending
            }

            _ => {
                // Fetch PCH
                cpu.pc |= u16::from(bus.read(0xffff)) << 8;

                Poll::Ready(())
            }
        }
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

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Explicit stack operations: push and pull of the accumulator (`PHA`, `PLA`) and
//! status register (`PHP`, `PLP`). Operations in [`flow`] that incidentally use the
//! stack (`JSR`, `RTS`, `RTI`) live there instead, grouped by their primary identity.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::operation::Operation;
use crate::cpu::{Cpu2A03, CpuStatus};

/// Pushes the accumulator onto the stack. 3 cycles.
/// Cycle 1 is a spurious read at PC; cycle 2 writes `A` to the stack pointer address and decrements `S`.
/// No flags are modified.
pub struct PHA;

impl Operation for PHA {
    #[allow(clippy::single_match_else)]
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => Poll::Pending,

            _ => {
                cpu.stack_push(bus, cpu.a);

                Poll::Ready(())
            }
        }
    }
}

/// Pushes the processor status register onto the stack with `U` and `B` always set. 3 cycles.
/// Cycle 1 is a spurious read at PC; cycle 2 writes `P | U | B` to the stack.
/// The live `P` register is not modified.
pub struct PHP;

impl Operation for PHP {
    #[allow(clippy::single_match_else)]
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => Poll::Pending,

            _ => {
                cpu.stack_push(bus, cpu.p.0 | CpuStatus::U | CpuStatus::B);

                Poll::Ready(())
            }
        }
    }
}

/// Pulls the accumulator from the stack. 4 cycles.
/// Cycles 1–2 are a spurious read and a stack-pointer increment; cycle 3 reads the new stack top into `A`.
/// Updates `Z` and `N`.
pub struct PLA;

impl Operation for PLA {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => Poll::Pending,

            2 => {
                let _ = bus.read(cpu.stack_address());
                cpu.sp = cpu.sp.wrapping_add(1);

                Poll::Pending
            }

            _ => {
                let value = bus.read(cpu.stack_address());

                cpu.a = value;
                cpu.p.update_zn(value);

                Poll::Ready(())
            }
        }
    }
}

/// Pulls the processor status register from the stack. 4 cycles.
/// Mirrors `PLA` timing but loads the pulled byte directly into `P` rather than `A`.
/// `B` (bit 4) has no physical register counterpart and is always cleared; `U` (bit 5) is
/// hardwired to 1 on the chip and is always set, regardless of what was on the stack.
pub struct PLP;

impl Operation for PLP {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 | 2 => PLA::apply(cpu, bus),

            _ => {
                let value = bus.read(cpu.stack_address());

                cpu.p = CpuStatus::new(value);

                Poll::Ready(())
            }
        }
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! 7-cycle hardware reset sequence run when `/RES` is released.
//!
//! During reset the chip forces R/W high, so the three stack cycles are *phantom
//! reads* at the stack address rather than pushes: SP still decrements but no
//! CPU state is written to memory. Reset also skips the P push entirely, sets
//! the `I` flag one cycle earlier than [`interrupt`][crate::interrupt::interrupt]
//! does, and reads the vector from `$FFFC/D`.
//!
//! The sequence is driven from `Phase::Reset` in [`Cpu2A03::tick`], one cycle
//! per call, the same way `Phase::IRQ`/`Phase::NMI` drive
//! [`interrupt`][crate::interrupt::interrupt].

use std::task::Poll;

use crate::{Cpu2A03, CpuReadWrite, CpuStatus};

/// Drives the reset sequence one cycle at a time. Seven cycles total: two
/// internal cycles (T0, T1) where the bus is read at PC/PC+1 and discarded,
/// three phantom stack reads (T2, T3, T4) that decrement SP without writing,
/// and two vector reads (T5, T6) that load PC from `$FFFC/D`. The `I` flag
/// is set at T4, one cycle before the first vector read.
///
/// See <https://www.nesdev.org/wiki/CPU_power_up_state> and the reset row in
/// <https://www.nesdev.org/wiki/CPU_interrupts> for the canonical per-cycle
/// reference.
#[expect(clippy::match_same_arms)]
pub(crate) fn reset<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
    match cpu.t {
        // T0: dummy read at PC; result discarded. The real chip is finishing
        // whatever it was doing when /RES asserted, so this read is along
        // for the ride.
        0 => {
            let _ = bus.read(cpu.pc);

            Poll::Pending
        }

        // T1: dummy read at PC+1; result discarded. Second of the two
        // internal pipeline cycles.
        1 => {
            let _ = bus.read(cpu.pc.wrapping_add(1));

            Poll::Pending
        }

        // T2: phantom stack access. On a live reset the R/W line is forced
        // high, so the chip issues a *read* at the stack address even though
        // it's going through the motions of a push. SP decrements.
        2 => {
            let _ = bus.read(cpu.stack_address());

            cpu.sp = cpu.sp.wrapping_sub(1);

            Poll::Pending
        }

        // T3: phantom stack access; SP decrements.
        3 => {
            let _ = bus.read(cpu.stack_address());

            cpu.sp = cpu.sp.wrapping_sub(1);

            Poll::Pending
        }

        // T4: phantom stack access; SP decrements. Sets `I` here, masking
        // IRQs before the vector read.
        4 => {
            let _ = bus.read(cpu.stack_address());

            cpu.sp = cpu.sp.wrapping_sub(1);
            cpu.p.insert(CpuStatus::I);

            Poll::Pending
        }

        // T5: fetch the reset vector low byte at $FFFC into ADL.
        5 => {
            cpu.adl = bus.read(0xFFFC);

            Poll::Pending
        }

        // T6: fetch the reset vector high byte at $FFFD into ADH and
        // assemble PC. Staging through ADL/ADH keeps the half-built PC off
        // the bus between the two reads.
        _ => {
            cpu.adh = bus.read(0xFFFD);
            cpu.pc = cpu.address();

            Poll::Ready(())
        }
    }
}

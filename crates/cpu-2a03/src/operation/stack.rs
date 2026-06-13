// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Explicit stack operations: push and pull of the accumulator (`PHA`, `PLA`) and the
//! status register (`PHP`, `PLP`).
//!
//! The stack lives in page 1 (`$0100`..`$01FF`) and grows downward, the stack pointer
//! `SP` always holds the *next* free byte. `TXS` (in [`transfer`][crate::operation])
//! seeds `SP` and `TSX` reads it back; the operations here only touch the stack itself.
//! Flow-control instructions that also touch the stack (`JSR`, `RTS`, `RTI`, `BRK`) live
//! in [`flow`] and [`system`], grouped by their primary purpose.
//!
//! See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for the per-instruction
//! reference.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::operation::Operation;
use crate::status::CpuStatus;

/// Pushes the accumulator onto the stack (`PHA`). 3 cycles.
///
/// - Cycle 1: opcode fetch (handled by the addressing-mode pipeline)
/// - Cycle 2: spurious read at `PC` (the canonical 6502 idle cycle for implied ops)
/// - Cycle 3: writes `A` to `$0100 | SP`, then decrements `SP`
///
/// No flags are modified.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#PHA>.
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

/// Pushes the processor status register onto the stack with `B` and `U` always set
/// (`PHP`). 3 cycles.
///
/// Same timing as `PHA`. The pushed byte is `P | B | U`, so software peeking at the
/// stacked status sees the canonical "this was a PHP/BRK" form regardless of how `P`
/// looks at the moment. The live `P` register is not modified, so neither bit lingers
/// after the push.
///
/// `B` and `U` have no physical storage on the chip, they're synthesized at push time
/// here and at pull time in [`PLP`]/[`RTI`](crate::operation::RTI).
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#PHP>.
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

/// Pulls the accumulator from the stack (`PLA`). 4 cycles.
///
/// - Cycle 1: opcode fetch
/// - Cycle 2: spurious read at `PC`
/// - Cycle 3: spurious read at the current stack top, then increment `SP`
/// - Cycle 4: read the new stack top into `A`
///
/// The two stack-side reads are how the hardware bridges the pre-increment `SP` and the
/// "where the data actually lives" address. Updates `Z` and `N`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#PLA>.
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

/// Pulls the processor status register from the stack (`PLP`). 4 cycles.
///
/// Same timing as `PLA`, but the pulled byte lands in `P` instead of `A`. The `B` and
/// `U` bits get rewritten on load: `B` (bit 4) is always cleared (it has no on-chip
/// counterpart, software just sees `0`), and `U` (bit 5) is always set (the bit is
/// hardwired to `1` on the real chip). Every other flag bit is taken verbatim from the
/// stacked byte.
///
/// Updates every flag except `B` and `U`, which are forced as described above.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#PLP>.
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

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

//! Shared 6502 interrupt sequence used by BRK, hardware IRQ, and hardware NMI.
//!
//! The three signals share most of the body: push return address, push status,
//! read PC from a vector, set the `I` flag. They only diverge at T1, on whether
//! `B` is set in the pushed status byte, and on which vector address is read.
//! [`InterruptKind`] carries those three variant points as a const-generic
//! parameter on [`interrupt`], so each kind monomorphizes to its own copy with
//! the constants folded into the body.

use std::marker::ConstParamTy;
use std::task::Poll;

use crate::{Cpu2A03, CpuReadWrite, CpuStatus};

/// Which 6502 interrupt signal the shared sequence is running for.
///
/// Used as a const-generic parameter on [`interrupt`]; each variant
/// monomorphizes to its own copy of the body with the T1 step, the status
/// mask, and the vector folded to immediates, so there is no runtime
/// branch on the kind inside the sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum InterruptKind {
    /// Hardware IRQ: `/IRQ` is asserted and the `I` flag is clear. Vector
    /// `$FFFE/F` (shared with BRK); pushed P has only `U` set; T1 is a
    /// dummy read at PC with no advance. Level-sensitive, so the asserter
    /// has to keep driving the line until the handler acknowledges.
    IRQ,

    /// Hardware NMI: a rising edge on `/NMI` was latched. Vector `$FFFA/B`;
    /// otherwise identical to [`IRQ`](Self::IRQ). Edge-triggered, so one
    /// pulse fires exactly one NMI regardless of how long the line stays
    /// high.
    NMI,

    /// Software interrupt via the `BRK` opcode (`$00`). Vector `$FFFE/F`
    /// shared with IRQ; pushed P sets both `B` and `U` so the handler can
    /// tell BRK from a hardware IRQ. T1 advances PC past the padding byte
    /// (the famous "BRK is a 2-byte instruction" quirk) so the pushed
    /// return address lands at BRK+2.
    BRK,
}

impl InterruptKind {
    /// Address of the vector's low byte. The high byte sits at `vector() + 1`.
    /// `$FFFE/F` for BRK and IRQ, `$FFFA/B` for NMI.
    ///
    /// `const` so it folds at monomorphization: the reads at T5/T6 hit the
    /// vector as fixed immediates.
    #[inline(always)]
    const fn vector(self) -> u16 {
        match self {
            Self::BRK | Self::IRQ => 0xfffe,
            Self::NMI => 0xfffa,
        }
    }
}

/// Drives the 6502 interrupt sequence one cycle at a time, shared by BRK,
/// hardware IRQ, and hardware NMI. The variant points (T1 step, pushed-P
/// mask, vector) come from `K` and fold to compile-time constants.
///
/// Entered at T1; T0 is absorbed by whichever path got here. For BRK that
/// means T0 was the `$00` opcode fetch in the CPU's fetch phase, and the
/// spurious operand read at T1 is done by [`Implied`][crate::addressing]
/// before this body runs. For hardware IRQ and NMI, T0 is the first dummy
/// read at PC, driven by the fetch phase before it hands off to the
/// dedicated interrupt phase that runs this routine.
///
/// Either way the bus pattern is: two dummy reads at PC (T0, T1), push PCH
/// (T2), push PCL (T3), push P (T4), read vector low and set `I` (T5),
/// read vector high (T6). Seven cycles total.
///
/// See <https://www.nesdev.org/wiki/CPU_interrupts> for the canonical
/// per-cycle reference.
#[expect(clippy::cast_possible_truncation)]
#[inline]
pub fn interrupt<const K: InterruptKind, B: CpuReadWrite>(
    cpu: &mut Cpu2A03,
    bus: &mut B,
) -> Poll<()> {
    match cpu.t {
        1 => {
            if matches!(K, InterruptKind::BRK) {
                // BRK: Implied already read the padding byte spuriously; advance PC past it
                // so the return address pushed below is BRK+2, as the 6502 requires.
                cpu.pc = cpu.pc.wrapping_add(1);
            } else {
                // IRQ/NMI: Second dummy read at PC without advancing
                let _ = bus.read(cpu.pc);
            }

            Poll::Pending
        }

        // Push PCH on stack, decrement S
        2 => {
            cpu.stack_push(bus, (cpu.pc >> 8) as u8);

            Poll::Pending
        }

        // Push PCL on stack, decrement S
        3 => {
            cpu.stack_push(bus, cpu.pc as u8);

            Poll::Pending
        }

        // Push P on stack with `U` always set, decrement S
        4 => {
            let mut status = cpu.p.0 | CpuStatus::U;

            if matches!(K, InterruptKind::BRK) {
                // BRK sets `B` to differentiate from IRQ
                status |= CpuStatus::B;
            }

            cpu.stack_push(bus, status);

            Poll::Pending
        }

        // Fetch PCL; set I to suppress further IRQs while in the handler
        5 => {
            cpu.pc = u16::from(bus.read(K.vector()));
            cpu.p.insert(CpuStatus::I);

            Poll::Pending
        }

        // Fetch PCH
        _ => {
            cpu.pc |= u16::from(bus.read(K.vector() + 1)) << 8;

            Poll::Ready(())
        }
    }
}

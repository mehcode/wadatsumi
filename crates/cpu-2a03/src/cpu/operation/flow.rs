// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Control flow, the only operations that write to the program counter.
//! Covers conditional branches (`BCC`–`BVS`), unconditional jumps (`JMP`),
//! and subroutine call and return (`JSR`, `RTS`, `RTI`).

use std::task::Poll;

use crate::Bus;
use crate::cpu::operation::Operation;
use crate::cpu::{Cpu2A03, CpuStatus};

/// Branches to a relative offset when status flag `FLAG` equals `EXPECTED` (`BCC`, `BCS`, `BEQ`, `BNE`, `BMI`, `BPL`, `BVC`, `BVS`).
/// Takes 2 cycles if not taken, 3 if taken same-page, or 4 if the branch crosses a page boundary.
pub struct BRANCH<const FLAG: CpuStatus, const EXPECTED: bool>;

impl<const FLAG: CpuStatus, const EXPECTED: bool> Operation for BRANCH<FLAG, EXPECTED> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            // Not-taken branches retire here (2 cycles total).
            1 => {
                let offset = i8::from_ne_bytes([cpu.data]);

                if cpu.p.contains(FLAG) == EXPECTED {
                    let target = cpu.pc.wrapping_add_signed(i16::from(offset));

                    cpu.data = u8::from(cpu.pc >> 8 != target >> 8);
                    [cpu.adl, cpu.adh] = target.to_le_bytes();

                    return Poll::Pending;
                }

                return Poll::Ready(());
            }

            // Spurious fetch at the next sequential opcode while the offset is applied to PCL.
            // Same-page branches retire here (3 cycles total).
            2 => {
                let _ = bus.read(cpu.pc); // dummy read

                if cpu.data != 0 {
                    return Poll::Pending;
                }
            }

            // Spurious read at the wrong-page PC while PCH is being corrected.
            // Page-crossing branches retire here (4 cycles total).
            3 => {
                let wrong_page_pc = (cpu.pc & 0xFF00) | u16::from(cpu.adl);
                let _ = bus.read(wrong_page_pc);
            }

            _ => {}
        }

        // Jump to the new effective address
        cpu.pc = cpu.address();

        Poll::Ready(())
    }
}

pub type BCC = BRANCH<{ CpuStatus::C }, false>;
pub type BCS = BRANCH<{ CpuStatus::C }, true>;
pub type BEQ = BRANCH<{ CpuStatus::Z }, true>;
pub type BNE = BRANCH<{ CpuStatus::Z }, false>;
pub type BMI = BRANCH<{ CpuStatus::N }, true>;
pub type BPL = BRANCH<{ CpuStatus::N }, false>;
pub type BVC = BRANCH<{ CpuStatus::V }, false>;
pub type BVS = BRANCH<{ CpuStatus::V }, true>;

/// Unconditional jump; sets the program counter to the resolved effective address.
/// Executes in a single step once the addressing mode has fully resolved `cpu.address`.
/// No flags are modified.
pub struct JMP;

impl Operation for JMP {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        cpu.pc = cpu.address();

        Poll::Ready(())
    }
}

/// Calls a subroutine at a 16-bit absolute address (`JSR nnnn`).
///
/// Pushes the address of the ADH operand byte (the last byte of this instruction) so that
/// `RTS` can pull and increment by one to resume at the following instruction. 6 cycles.
///
/// Uses `Implied` addressing: ADL is pre-read into `cpu.data` with PC left pointing at ADH.
pub struct JSR;

impl Operation for JSR {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => {
                // ADL was pre-read into cpu.data by Implied's spurious read without advancing PC.
                // Nudge PC to ADH so it is correctly positioned for the push and final fetch.
                cpu.pc = cpu.pc.wrapping_add(1);

                Poll::Pending
            }

            2 => {
                // Internal: spurious read from the current stack top (hardware pipeline artifact).
                let _ = bus.read(cpu.stack_address());

                Poll::Pending
            }

            3 => {
                // Push PCH. PC points at ADH, the last byte of this instruction, which is the
                // correct return address for RTS to pull and increment.
                cpu.stack_push(bus, (cpu.pc >> 8) as u8);

                Poll::Pending
            }

            4 => {
                cpu.stack_push(bus, cpu.pc as u8);

                Poll::Pending
            }

            _ => {
                // Fetch ADH and assemble the full target address; ADL is waiting in cpu.data.
                cpu.pc = u16::from(cpu.fetch(bus)) << 8 | u16::from(cpu.data);

                Poll::Ready(())
            }
        }
    }
}

/// Returns from a subroutine; pulls the return address from the stack and increments it by one.
///
/// The address on the stack is JSR's ADH byte (last byte of the JSR instruction), so
/// incrementing by 1 lands on the byte immediately following the full JSR instruction.
pub struct RTS;

impl Operation for RTS {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            // T1: internal, wait for stack pointer.
            1 => Poll::Pending,

            // Dummy read at current stack top, then increment S.
            2 => {
                let _ = bus.read(cpu.stack_address());

                cpu.sp = cpu.sp.wrapping_add(1);

                Poll::Pending
            }

            // Pull PCL from stack, increment S.
            3 => {
                cpu.pc = u16::from(bus.read(cpu.stack_address()));
                cpu.sp = cpu.sp.wrapping_add(1);

                Poll::Pending
            }

            // Pull PCH from stack.
            4 => {
                cpu.pc |= u16::from(bus.read(cpu.stack_address())) << 8;

                Poll::Pending
            }

            // Spurious read at the return address; hardware reads PC before incrementing.
            _ => {
                let _ = bus.read(cpu.pc);
                cpu.pc = cpu.pc.wrapping_add(1);

                Poll::Ready(())
            }
        }
    }
}

/// Returns from an interrupt; restores P and PC from the stack. 6 cycles.
///
/// Unlike `RTS`, the stacked PC is the exact return address (no +1 adjustment), and P is
/// pulled before PC. The `B` flag is cleared and `U` is set on the restored P, identical
/// to `PLP`.
pub struct RTI;

impl Operation for RTI {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            // T1: internal, wait for stack pointer.
            1 => Poll::Pending,

            // Dummy read at current stack top, then increment S.
            2 => {
                let _ = bus.read(cpu.stack_address());

                cpu.sp = cpu.sp.wrapping_add(1);

                Poll::Pending
            }

            // Pull P from stack, increment S.
            3 => {
                let value = bus.read(cpu.stack_address());

                cpu.p = CpuStatus::new(value);
                cpu.sp = cpu.sp.wrapping_add(1);

                Poll::Pending
            }

            // Pull PCL from stack, increment S.
            4 => {
                cpu.pc = u16::from(bus.read(cpu.stack_address()));
                cpu.sp = cpu.sp.wrapping_add(1);

                Poll::Pending
            }

            // Pull PCH from stack.
            _ => {
                cpu.pc |= u16::from(bus.read(cpu.stack_address())) << 8;

                Poll::Ready(())
            }
        }
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Control flow, the only operations that write directly to the program counter.
//!
//! Covers conditional branches (`BCC`..`BVS`), the unconditional jump (`JMP`), and
//! subroutine call and return (`JSR`, `RTS`, `RTI`). Everything else moves PC only by
//! the implicit "fetch advances PC" step the addressing-mode pipeline runs every
//! instruction.
//!
//! See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for the per-instruction
//! reference.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::operation::Operation;
use crate::status::CpuStatus;

/// Branches by a signed 8-bit offset when status flag `FLAG` equals `EXPECTED` (`BCC`,
/// `BCS`, `BEQ`, `BNE`, `BMI`, `BPL`, `BVC`, `BVS`).
///
/// Three possible costs depending on what the branch actually does:
///
/// - **2 cycles** when the branch is not taken
/// - **3 cycles** when taken and the target lies on the same 256-byte page as `PC`
/// - **4 cycles** when taken and the target crosses a page boundary
///
/// The extra cycle on a page cross comes from the 6502 carrying the offset into `PCL`
/// first (producing a wrong-page address) and then fixing up `PCH` on a separate cycle.
/// No flags are modified.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#BCC>.
pub struct BRANCH<const FLAG: CpuStatus, const EXPECTED: bool>;

impl<const FLAG: CpuStatus, const EXPECTED: bool> Operation for BRANCH<FLAG, EXPECTED> {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
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

/// Unconditional jump: sets `PC` to the resolved effective address (`JMP`).
///
/// Executes in a single step once the addressing mode has fully resolved `cpu.address`,
/// so the per-mode cycle count (3 for absolute, 5 for indirect) lives in the addressing
/// mode rather than here. No flags are modified.
///
/// Indirect `JMP` has the famous 6502 page-wrap bug, when the pointer's low byte is
/// `$FF`, the high byte is fetched from the *same* page rather than the next. That quirk
/// is handled by the indirect addressing mode rather than in `apply`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#JMP>.
pub struct JMP;

impl Operation for JMP {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        cpu.pc = cpu.address();

        Poll::Ready(())
    }
}

/// Calls a subroutine at a 16-bit absolute address (`JSR nnnn`). 6 cycles.
///
/// Pushes the address of the ADH operand byte (the last byte of this instruction, not the
/// first byte of the *next* one) onto the stack, then jumps to the absolute target. `RTS`
/// later pulls that address and increments by one to land on the byte immediately
/// following `JSR`. The "off by one" return address is a 6502 oddity worth knowing about:
/// if a stack trace is ever decoded by hand, every return address there points at the
/// byte *before* the resume point.
///
/// Dispatched through the `Implied` addressing mode so ADL is pre-read into `cpu.data`
/// with `PC` left pointing at ADH, ready for `apply` to push the in-flight return address
/// and then fetch ADH to assemble the target.
///
/// No flags are modified.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#JSR>.
pub struct JSR;

impl Operation for JSR {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
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

/// Returns from a subroutine: pulls the return address from the stack and increments it
/// by one (`RTS`). 6 cycles.
///
/// The on-stack address is the ADH byte of the matching `JSR` (the last byte of the
/// instruction, not the first byte after it), so `RTS` has to add 1 after the pull to
/// reach the actual resume point. The final cycle re-reads from the post-increment `PC`
/// to mirror the hardware's pipeline. No flags are modified.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#RTS>.
pub struct RTS;

impl Operation for RTS {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
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

/// Returns from an interrupt: restores `P` and `PC` from the stack (`RTI`). 6 cycles.
///
/// Two things distinguish this from `RTS`:
///
/// - The stacked `PC` is the *exact* resume address (no +1 fixup), because the interrupt
///   sequence pushes the in-flight `PC` directly rather than the off-by-one
///   `JSR`-style address.
/// - `P` is pulled before `PC`, and the `B` and `U` bits are forced (clear and set
///   respectively) on the restored `P`, the same masking [`PLP`](crate::operation::PLP)
///   applies.
///
/// `RTI` is the only instruction that re-enables interrupts atomically with the return
/// jump, so an IRQ pending at the instant `RTI` retires won't be serviced until the next
/// instruction has at least begun execution.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#RTI>.
pub struct RTI;

impl Operation for RTI {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
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

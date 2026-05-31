// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::state::CpuStatus;
use crate::cpu::state::Register::{self, A, X, Y};

#[derive(Debug, Clone, Copy)]
pub enum OperationMode {
    Implicit,
    Read,
    Write,
    ReadModifyWrite,
}

impl OperationMode {
    /// Returns `true` if the mode is *read*.
    pub const fn is_read(self) -> bool {
        matches!(self, OperationMode::Read)
    }
}

/// The effect of a single 2A03 instruction mnemonic.
///
/// An `Operation` is the second half of instruction dispatch, after an
/// [`AddressingMode`] resolves the effective address into `cpu.address`,
/// the operation applies the mnemonic's effect to CPU and bus state.
pub trait Operation {
    const MODE: OperationMode = OperationMode::Implicit;

    /// Applies the operation's effect for the current cycle.
    ///
    /// Called only after the addressing mode has fully resolved. `cpu.address`
    /// holds the effective address. Returns `Poll::Ready(())` when the operation is
    /// complete, signalling the CPU to clear the in-flight instruction.
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()>
    where
        Self: Sized;
}

/// Branches to a relative offset when status flag `FLAG` equals `EXPECTED` (`BCC`, `BCS`, `BEQ`, `BNE`, `BMI`, `BPL`, `BVC`, `BVS`).
/// Takes 2 cycles if not taken, 3 if taken same-page, or 4 if the branch crosses a page boundary.
pub struct BRANCH<const FLAG: u8, const EXPECTED: bool>;

impl<const FLAG: u8, const EXPECTED: bool> Operation for BRANCH<FLAG, EXPECTED> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let flag: CpuStatus = CpuStatus::from_bits_truncate(FLAG);

        match cpu.t {
            // Not-taken branches retire here (2 cycles total).
            1 => {
                let offset = i8::from_ne_bytes([cpu.data]);

                if cpu.state.p.contains(flag) == EXPECTED {
                    let target = cpu.state.pc.wrapping_add_signed(i16::from(offset));

                    cpu.data = u8::from(cpu.state.pc >> 8 != target >> 8);
                    cpu.address = target;

                    return Poll::Pending;
                }

                return Poll::Ready(());
            }

            // Spurious fetch at the next sequential opcode while the offset is applied to PCL.
            // Same-page branches retire here (3 cycles total).
            2 => {
                let _ = bus.read(cpu.state.pc); // dummy read

                if cpu.data != 0 {
                    return Poll::Pending;
                }
            }

            // Spurious read at the wrong-page PC while PCH is being corrected.
            // Page-crossing branches retire here (4 cycles total).
            3 => {
                let wrong_page_pc = (cpu.state.pc & 0xFF00) | (cpu.address & 0x00FF);
                let _ = bus.read(wrong_page_pc);
            }

            _ => {}
        }

        // Jump to the new effective address
        cpu.state.pc = cpu.address;

        Poll::Ready(())
    }
}

pub type BCC = BRANCH<{ CpuStatus::C.bits() }, false>;
pub type BCS = BRANCH<{ CpuStatus::C.bits() }, true>;
pub type BEQ = BRANCH<{ CpuStatus::Z.bits() }, true>;
pub type BNE = BRANCH<{ CpuStatus::Z.bits() }, false>;
pub type BMI = BRANCH<{ CpuStatus::N.bits() }, true>;
pub type BPL = BRANCH<{ CpuStatus::N.bits() }, false>;
pub type BVC = BRANCH<{ CpuStatus::V.bits() }, false>;
pub type BVS = BRANCH<{ CpuStatus::V.bits() }, true>;

/// Clears status flag `FLAG` unconditionally (`CLC`, `CLI`, `CLD`, `CLV`).
pub struct CLEAR<const FLAG: u8>;

impl<const FLAG: u8> Operation for CLEAR<FLAG> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        cpu.state.p.remove(CpuStatus::from_bits_truncate(FLAG));

        Poll::Ready(())
    }
}

pub type CLC = CLEAR<{ CpuStatus::C.bits() }>;
pub type CLI = CLEAR<{ CpuStatus::I.bits() }>;
pub type CLD = CLEAR<{ CpuStatus::D.bits() }>;
pub type CLV = CLEAR<{ CpuStatus::V.bits() }>;

/// Unconditional jump; sets the program counter to the resolved effective address.
pub struct JMP;

impl Operation for JMP {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        cpu.state.pc = cpu.address;

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
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => {
                // ADL was pre-read into cpu.data by Implied's spurious read without advancing PC.
                // Nudge PC to ADH so it is correctly positioned for the push and final fetch.
                cpu.state.pc = cpu.state.pc.wrapping_add(1);

                Poll::Pending
            }

            2 => {
                // Internal: spurious read from the current stack top (hardware pipeline artifact).
                let _ = bus.read(cpu.state.stack_address());

                Poll::Pending
            }

            3 => {
                // Push PCH. PC points at ADH, the last byte of this instruction, which is the
                // correct return address for RTS to pull and increment.
                cpu.stack_push(bus, (cpu.state.pc >> 8) as u8);

                Poll::Pending
            }

            4 => {
                cpu.stack_push(bus, cpu.state.pc as u8);

                Poll::Pending
            }

            _ => {
                // Fetch ADH and assemble the full target address; ADL is waiting in cpu.data.
                cpu.state.pc = u16::from(cpu.fetch(bus)) << 8 | u16::from(cpu.data);

                Poll::Ready(())
            }
        }
    }
}

/// No operation; idles for one additional cycle after the opcode fetch.
pub struct NOP;

impl Operation for NOP {
    #[inline]
    fn apply<B: Bus>(_: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        // Do nothing
        Poll::Ready(())
    }
}

/// Loads a byte from the effective address into the register (`LDA`, `LDX`, `LDY`).
/// Updates `Z` and `N`.
pub struct LOAD<const R: Register>;

impl<const R: Register> Operation for LOAD<R> {
    const MODE: OperationMode = OperationMode::Read;

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        cpu.state.set::<R>(cpu.data);
        cpu.state.p.update_zn(cpu.data);

        Poll::Ready(())
    }
}

pub type LDA = LOAD<{ A }>;
pub type LDX = LOAD<{ X }>;
pub type LDY = LOAD<{ Y }>;

/// Pushes the accumulator onto the stack. 3 cycles.
pub struct PHA;

impl Operation for PHA {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => Poll::Pending,

            _ => {
                cpu.stack_push(bus, cpu.state.a);

                Poll::Ready(())
            }
        }
    }
}

/// Pushes the processor status register onto the stack with `U` and `B` always set. 3 cycles.
pub struct PHP;

impl Operation for PHP {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => Poll::Pending,

            _ => {
                cpu.stack_push(bus, cpu.state.p.bits() | CpuStatus::U | CpuStatus::B);

                Poll::Ready(())
            }
        }
    }
}

/// Pulls the accumulator from the stack; updates `Z` and `N`. 4 cycles.
pub struct PLA;

impl Operation for PLA {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 => Poll::Pending,

            2 => {
                let _ = bus.read(cpu.state.stack_address());
                cpu.state.sp = cpu.state.sp.wrapping_add(1);

                Poll::Pending
            }

            _ => {
                let value = bus.read(cpu.state.stack_address());

                cpu.state.a = value;
                cpu.state.p.update_zn(value);

                Poll::Ready(())
            }
        }
    }
}

/// Pulls the processor status register from the stack. 4 cycles.
pub struct PLP;

impl Operation for PLP {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            1 | 2 => PLA::apply(cpu, bus),

            _ => {
                let value = bus.read(cpu.state.stack_address());

                cpu.state.p = CpuStatus::from_bits_truncate(value);

                Poll::Ready(())
            }
        }
    }
}

/// Returns from a subroutine; pulls the return address from the stack and increments it by one.
///
/// Uses `Implied` addressing: the spurious PC read (hardware cycle 2) is handled there.
/// The address on the stack is JSR's ADH byte (last byte of the JSR instruction), so
/// incrementing by 1 lands on the byte immediately following the full JSR instruction.
pub struct RTS;

impl Operation for RTS {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        match cpu.t {
            // Implied already performed the spurious fetch (hardware cycle 2); just advance.
            1 => Poll::Pending,

            // Hardware cycle 3: dummy read at current stack top, then increment S.
            2 => {
                let _ = bus.read(cpu.state.stack_address());
                cpu.state.sp = cpu.state.sp.wrapping_add(1);

                Poll::Pending
            }

            // Hardware cycle 4: pull PCL from stack, increment S.
            3 => {
                cpu.data = bus.read(cpu.state.stack_address());
                cpu.state.sp = cpu.state.sp.wrapping_add(1);

                Poll::Pending
            }

            // Hardware cycle 5: pull PCH from stack, assemble PC.
            4 => {
                let pch = u16::from(bus.read(cpu.state.stack_address()));
                cpu.state.pc = (pch << 8) | u16::from(cpu.data);

                Poll::Pending
            }

            // Hardware cycle 6: increment PC to point past JSR's last operand byte.
            _ => {
                cpu.state.pc = cpu.state.pc.wrapping_add(1);

                Poll::Ready(())
            }
        }
    }
}

/// Sets status flag `FLAG` unconditionally (`SEC`, `SEI`, `SED`).
pub struct SET<const FLAG: u8>;

impl<const FLAG: u8> Operation for SET<FLAG> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        cpu.state.p.insert(CpuStatus::from_bits_truncate(FLAG));

        Poll::Ready(())
    }
}

pub type SEC = SET<{ CpuStatus::C.bits() }>;
pub type SEI = SET<{ CpuStatus::I.bits() }>;
pub type SED = SET<{ CpuStatus::D.bits() }>;

/// Stores the contents of the register into memory,
/// at the effective address (`STA`, `STX`, `STY`).
pub struct STORE<const R: Register>;

impl<const R: Register> Operation for STORE<R> {
    const MODE: OperationMode = OperationMode::Write;

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.state.get::<R>();

        bus.write(cpu.address, value);

        Poll::Ready(())
    }
}

pub type STA = STORE<{ A }>;
pub type STX = STORE<{ X }>;
pub type STY = STORE<{ Y }>;

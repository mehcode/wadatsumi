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

pub struct BRANCH<const FLAG: u8, const EXPECTED: bool>;

impl<const FLAG: u8, const EXPECTED: bool> Operation for BRANCH<FLAG, EXPECTED> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let flag: CpuStatus = CpuStatus::from_bits_truncate(FLAG);

        match cpu.cycle {
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

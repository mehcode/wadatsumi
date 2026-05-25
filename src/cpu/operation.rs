// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::state::Register::{self, *};

/// The effect of a single 2A03 instruction mnemonic.
///
/// An `Operation` is the second half of instruction dispatch, after an
/// [`AddressingMode`] resolves the effective address into `cpu.address`,
/// the operation applies the mnemonic's effect to CPU and bus state.
pub trait Operation {
    /// Applies the operation's effect for the current cycle.
    ///
    /// Called only after the addressing mode has fully resolved. `cpu.address`
    /// holds the effective address. Returns `true` when the operation is
    /// complete, signalling the CPU to clear the in-flight instruction.
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool
    where
        Self: Sized;
}

/// Loads a byte from the effective address into the register (`LDA`, `LDX`, `LDY`).
/// Updates `Z` and `N`.
pub struct LOAD<const R: Register>;

impl<const R: Register> Operation for LOAD<R> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool {
        let value = bus.read(cpu.address);

        cpu.state.set::<R>(value);
        cpu.state.p.update_zn(value);

        true
    }
}

pub type LDA = LOAD<{ A }>;
pub type LDX = LOAD<{ X }>;
pub type LDY = LOAD<{ Y }>;

/// Stores the contents of the register into memory,
/// at the effective address (`STA`, `STX`, `STY`).
pub struct STORE<const R: Register>;

impl<const R: Register> Operation for STORE<R> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool {
        let value = cpu.state.get::<R>();

        bus.write(cpu.address, value);

        true
    }
}

pub type STA = STORE<{ A }>;
pub type STX = STORE<{ X }>;
pub type STY = STORE<{ Y }>;

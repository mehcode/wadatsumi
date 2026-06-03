// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pure data movement between registers and memory. Unlike [`arithmetic`] or
//! [`logical`], the value is copied verbatim, no computation is applied.
//! Covers all three directions: Memory → Register, Register → Memory, and
//! Register → Register.

use std::task::Poll;

use crate::Bus;
use crate::cpu::Cpu;
use crate::cpu::operation::Register::{self, A, SP, X, Y};
use crate::cpu::operation::{MemoryAccess, Operation};

/// Loads a byte from the effective address into the register (`LDA`, `LDX`, `LDY`).
/// Updates `Z` and `N`.
pub struct LOAD<const R: Register>;

impl<const R: Register> Operation for LOAD<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        R.set(&mut cpu.state, cpu.data);
        cpu.state.p.update_zn(cpu.data);

        Poll::Ready(())
    }
}

pub type LDA = LOAD<{ A }>;
pub type LDX = LOAD<{ X }>;
pub type LDY = LOAD<{ Y }>;

/// Stores register `R` into memory at the effective address (`STA`, `STX`, `STY`).
/// Writes in a single step once the addressing mode resolves.
/// No flags are modified.
pub struct STORE<const R: Register>;

impl<const R: Register> Operation for STORE<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = R.get(&cpu.state);

        bus.write(cpu.address, value);

        Poll::Ready(())
    }
}

pub type STA = STORE<{ A }>;
pub type STX = STORE<{ X }>;
pub type STY = STORE<{ Y }>;

/// Copies register `SRC` into register `DST` in a single implicit cycle (`TAX`, `TAY`, `TSX`, `TXA`, `TYA`, `TXS`).
/// Updates `Z` and `N` when the destination is `A`, `X`, or `Y`.
/// `TXS` is the sole exception and leaves all flags unchanged.
pub struct TRANSFER<const SRC: Register, const DST: Register>;

impl<const SRC: Register, const DST: Register> Operation for TRANSFER<SRC, DST> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        let value = SRC.get(&cpu.state);

        DST.set(&mut cpu.state, value);

        if matches!(DST, A | X | Y) {
            cpu.state.p.update_zn(value);
        }

        Poll::Ready(())
    }
}

pub type TAX = TRANSFER<{ A }, { X }>;
pub type TAY = TRANSFER<{ A }, { Y }>;
pub type TSX = TRANSFER<{ SP }, { X }>;
pub type TXA = TRANSFER<{ X }, { A }>;
pub type TYA = TRANSFER<{ Y }, { A }>;
pub type TXS = TRANSFER<{ X }, { SP }>;

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pure data movement between registers and memory. Unlike [`arithmetic`] or
//! [`logical`], the value is copied verbatim, no computation is applied.
//! Covers all three directions: Memory → Register, Register → Memory, and
//! Register → Register.

use std::task::Poll;

use crate::Bus;
use crate::cpu::Cpu2A03;
use crate::cpu::operation::Register::{self, A, SP, X, Y};
use crate::cpu::operation::{MAGIC, MemoryAccess, Operation};

/// Loads a byte from the effective address into both `A` and `X` (`LAX`).
/// Updates `Z` and `N`.
pub struct LAX;

impl Operation for LAX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        cpu.state.a = cpu.data;
        cpu.state.x = cpu.data;

        cpu.state.p.update_zn(cpu.data);

        Poll::Ready(())
    }
}

/// Loads a byte from the effective address into the register (`LDA`, `LDX`, `LDY`).
/// Updates `Z` and `N`.
pub struct LOAD<const R: Register>;

impl<const R: Register> Operation for LOAD<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        R.set(&mut cpu.state, cpu.data);
        cpu.state.p.update_zn(cpu.data);

        Poll::Ready(())
    }
}

pub type LDA = LOAD<{ A }>;
pub type LDX = LOAD<{ X }>;
pub type LDY = LOAD<{ Y }>;

/// AND `(A | MAGIC)` with immediate byte, then store in both `A` and `X` (`LXA`/`OAL`).
/// Updates `Z` and `N`.
pub struct LXA;

impl Operation for LXA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let result = (cpu.state.a | MAGIC) & cpu.data;

        cpu.state.a = result;
        cpu.state.x = result;
        cpu.state.p.update_zn(result);

        Poll::Ready(())
    }
}

pub struct SAX;

impl Operation for SAX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.state.a & cpu.state.x;

        bus.write(cpu.address, value);

        Poll::Ready(())
    }
}

/// Stores register `R` into memory at the effective address (`STA`, `STX`, `STY`).
/// Writes in a single step once the addressing mode resolves.
/// No flags are modified.
pub struct STORE<const R: Register>;

impl<const R: Register> Operation for STORE<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = R.get(&cpu.state);

        bus.write(cpu.address, value);

        Poll::Ready(())
    }
}

pub type STA = STORE<{ A }>;
pub type STX = STORE<{ X }>;
pub type STY = STORE<{ Y }>;

/// Stores `A & X & (baseAddrHigh + 1)` into memory (`SHA`/`AHX`).
/// `baseAddrHigh` is the high byte of the effective address before Y-indexing. No flags are modified.
pub struct SHA;

impl Operation for SHA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let base_high = (cpu.address.wrapping_sub(u16::from(cpu.state.y)) >> 8) as u8;
        let value = cpu.state.a & cpu.state.x & base_high.wrapping_add(1);

        bus.write(cpu.address, value);

        Poll::Ready(())
    }
}

/// Stores `R & (baseAddrHigh + 1)` into memory (`SHY`, `SHX`).
/// `baseAddrHigh` is the high byte of the effective address before `IDX`-indexing. No flags are modified.
/// On a page cross the hardware outputs the result on the address high bus instead of the carry-corrected
/// `base_hi+1`, so the write goes to `(result << 8) | lo` rather than the true effective address.
pub struct SH<const R: Register, const IDX: Register>;

impl<const R: Register, const IDX: Register> Operation for SH<R, IDX> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let index = IDX.get(&cpu.state);
        let base_high = (cpu.address.wrapping_sub(u16::from(index)) >> 8) as u8;

        let value = R.get(&cpu.state);
        let result = value & base_high.wrapping_add(1);

        // On a page cross the result ANDs into the address high byte (hardware bus conflict).
        let address: u16 = if (cpu.address >> 8) as u8 == base_high {
            cpu.address
        } else {
            (u16::from(result) << 8) | (cpu.address & 0xFF)
        };

        bus.write(address, result);

        Poll::Ready(())
    }
}

pub type SHY = SH<{ Y }, { X }>;
pub type SHX = SH<{ X }, { Y }>;

/// Copies register `SRC` into register `DST` in a single implicit cycle (`TAX`, `TAY`, `TSX`, `TXA`, `TYA`, `TXS`).
/// Updates `Z` and `N` when the destination is `A`, `X`, or `Y`.
/// `TXS` is the sole exception and leaves all flags unchanged.
pub struct TRANSFER<const SRC: Register, const DST: Register>;

impl<const SRC: Register, const DST: Register> Operation for TRANSFER<SRC, DST> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
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

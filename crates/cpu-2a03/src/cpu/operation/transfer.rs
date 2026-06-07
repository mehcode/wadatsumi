// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pure data movement between registers and memory. Unlike [`arithmetic`] or
//! [`logical`], the value is copied verbatim, no computation is applied.
//! Covers all three directions: Memory → Register, Register → Memory, and
//! Register → Register.

use std::task::{Poll, ready};

use crate::Bus;
use crate::cpu::Cpu2A03;
use crate::cpu::operation::Register::{self, A, SP, X, Y};
use crate::cpu::operation::{MemoryAccess, Operation};

/// Reads memory, ANDs with `SP`, and stores the result into `A`, `X`, and `SP` (`LAS`/`LAE`/`LAR`).
/// Updates `Z` and `N`.
pub struct LAS;

impl Operation for LAS {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let result = cpu.data & cpu.sp;

        cpu.a = result;
        cpu.x = result;
        cpu.sp = result;

        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Loads a byte from the effective address into both `A` and `X` (`LAX`).
/// Updates `Z` and `N`.
pub struct LAX;

impl Operation for LAX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        // LDA loads cpu.data into A and updates flags; LDX reads the same cpu.data into X.
        // Both halves consume the byte already latched by the Read pipeline, no extra bus read.
        // LDA loads cpu.data into A and updates flags; LDX reads the same cpu.data into X.
        // Both halves consume the byte already latched by the Read pipeline, no extra bus read.
        ready!(LDA::apply(cpu, bus));

        LDX::apply(cpu, bus)
    }
}

/// Loads a byte from the effective address into the register (`LDA`, `LDX`, `LDY`).
/// Updates `Z` and `N`.
pub struct LOAD<const R: Register>;

impl<const R: Register> Operation for LOAD<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        R.set(cpu, cpu.data);
        cpu.p.update_zn(cpu.data);

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
        let result = (cpu.a | cpu.magic) & cpu.data;

        cpu.a = result;
        cpu.x = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// AND `(A | MAGIC)` with `X` and the immediate byte; store result in `A` only (`XAA`/`ANE`).
/// Highly unstable on real hardware; behaviour depends on analog MAGIC which varies by chip revision.
/// Updates `Z` and `N`.
pub struct XAA;

impl Operation for XAA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let result = (cpu.a | cpu.magic) & cpu.x & cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Stores `A & X` into memory at the effective address (`SAX`).
/// Does not affect any flags or modify `A` or `X`.
pub struct SAX;

impl Operation for SAX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.a & cpu.x;

        bus.write(cpu.address(), value);

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
        let value = R.get(cpu);

        bus.write(cpu.address(), value);

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
        let base_high = (cpu.address().wrapping_sub(u16::from(cpu.y)) >> 8) as u8;
        let result = cpu.a & cpu.x & base_high.wrapping_add(1);

        // On a page cross the result ANDs into the address high byte (hardware bus conflict).
        let address = if cpu.adh == base_high {
            cpu.address()
        } else {
            u16::from_le_bytes([cpu.adl, result])
        };

        bus.write(address, result);

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
        let index = IDX.get(cpu);
        let base_high = (cpu.address().wrapping_sub(u16::from(index)) >> 8) as u8;

        let value = R.get(cpu);
        let result = value & base_high.wrapping_add(1);

        // On a page cross the result ANDs into the address high byte (hardware bus conflict).
        let address = if cpu.adh == base_high {
            cpu.address()
        } else {
            u16::from_le_bytes([cpu.adl, result])
        };

        bus.write(address, result);

        Poll::Ready(())
    }
}

pub type SHY = SH<{ Y }, { X }>;
pub type SHX = SH<{ X }, { Y }>;

/// Sets `SP = A & X`, then stores `SP & (base_high + 1)` into memory (`TAS`/`SHS`/`XAS`).
/// No flags modified. On a page cross the result corrupts the high address byte, identical to `SHA`.
pub struct TAS;

impl Operation for TAS {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        // SHA computes A & X & (base_high + 1) and handles the page-cross bus conflict.
        // Setting SP = A & X first means SP & (base_high + 1) == A & X & (base_high + 1),
        // so SHA's existing logic is correct here without any changes.
        cpu.sp = cpu.a & cpu.x;
        SHA::apply(cpu, bus)
    }
}

/// Copies register `SRC` into register `DST` in a single implicit cycle (`TAX`, `TAY`, `TSX`, `TXA`, `TYA`, `TXS`).
/// Updates `Z` and `N` when the destination is `A`, `X`, or `Y`.
/// `TXS` is the sole exception and leaves all flags unchanged.
pub struct TRANSFER<const SRC: Register, const DST: Register>;

impl<const SRC: Register, const DST: Register> Operation for TRANSFER<SRC, DST> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let value = SRC.get(cpu);

        DST.set(cpu, value);

        if matches!(DST, A | X | Y) {
            cpu.p.update_zn(value);
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

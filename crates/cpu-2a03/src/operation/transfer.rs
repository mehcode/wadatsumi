// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pure data movement between registers and memory.
//!
//! Unlike [`arithmetic`] or [`logical`], the value is copied verbatim, no computation
//! happens between source and destination. Covers all three directions: Memory to
//! Register (`LDA`/`LDX`/`LDY`), Register to Memory (`STA`/`STX`/`STY`), and Register to
//! Register (`TAX`/`TAY`/`TSX`/`TXA`/`TYA`/`TXS`). Several undocumented combinations
//! that fan a single source out to multiple destinations (`LAX`, `LAS`, `SAX`, `LXA`,
//! `XAA`, `SHA`/`SHX`/`SHY`, `TAS`) live here too.
//!
//! See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for the per-instruction
//! reference.

use std::task::{Poll, ready};

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::operation::Register::{self, A, SP, X, Y};
use crate::operation::{MemoryAccess, Operation};

/// Reads memory, ANDs the byte with `SP`, and stores the result into all three of `A`,
/// `X`, and `SP` (`LAS`/`LAE`/`LAR`).
///
/// An undocumented three-way fan-out. `SP` ends up holding the same value as `A` and `X`,
/// which means software that runs `LAS` accidentally trashes its own stack pointer.
/// Updates `Z` and `N`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct LAS;

impl Operation for LAS {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result = cpu.data & cpu.sp;

        cpu.a = result;
        cpu.x = result;
        cpu.sp = result;

        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Loads a byte from the effective address into both `A` and `X` (`LAX`).
///
/// An undocumented fused `LDA` + `LDX`, one bus read feeds both registers.
/// Updates `Z` and `N`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct LAX;

impl Operation for LAX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // LDA loads cpu.data into A and updates flags; LDX reads the same cpu.data into X.
        // Both halves consume the byte already latched by the Read pipeline, no extra bus read.
        ready!(LDA::apply(cpu, bus));

        LDX::apply(cpu, bus)
    }
}

/// Loads the byte at the effective address into register `R` (`LDA`, `LDX`, `LDY`).
///
/// Stores the byte verbatim, no transformation. Updates `Z` and `N`, leaves `C` and `V`
/// alone.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#LDA>.
pub struct LOAD<const R: Register>;

impl<const R: Register> Operation for LOAD<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        R.set(cpu, cpu.data);
        cpu.p.update_zn(cpu.data);

        Poll::Ready(())
    }
}

pub type LDA = LOAD<{ A }>;
pub type LDX = LOAD<{ X }>;
pub type LDY = LOAD<{ Y }>;

/// AND `(A | MAGIC)` with an immediate byte, then store the result in both `A` and `X`
/// (`LXA`/`OAL`/`ATX`).
///
/// An undocumented and unstable opcode. `MAGIC` is an analog quantity that depends on
/// chip revision, temperature, and the value previously sitting on the internal bus, on
/// hardware it commonly looks like `$EE` or `$FF` but is not guaranteed.
/// Updates `Z` and `N`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct LXA;

impl Operation for LXA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result = (cpu.a | cpu.magic) & cpu.data;

        cpu.a = result;
        cpu.x = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// AND `(A | MAGIC)` with `X` and the immediate byte, store the result in `A` only
/// (`XAA`/`ANE`).
///
/// Highly unstable on real hardware. `MAGIC` is an analog quantity that varies by chip
/// revision, temperature, and recent bus traffic, the same opcode at the same value can
/// produce different results across power cycles. No commercial NES title relies on it,
/// included here mostly so opcode-conformance tests can cover the entire 256-entry
/// table. Updates `Z` and `N`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct XAA;

impl Operation for XAA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result = (cpu.a | cpu.magic) & cpu.x & cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Stores `A & X` into memory at the effective address (`SAX`/`AXS`/`AAX`).
///
/// An undocumented combination, the AND is computed internally without consuming a bus
/// cycle, so this is the same length as a plain `STA`. Neither `A` nor `X` is modified,
/// and no flags are touched, which makes it a handy way to scratch `A & X` to memory
/// without disturbing the comparison state.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct SAX;

impl Operation for SAX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let value = cpu.a & cpu.x;

        bus.write(cpu.address(), value);

        Poll::Ready(())
    }
}

/// Stores register `R` into memory at the effective address (`STA`, `STX`, `STY`).
///
/// Writes in a single step once the addressing mode resolves. No flags are modified, so
/// stores are flag-transparent and can sit between a compare and its branch without
/// invalidating the condition.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#STA>.
pub struct STORE<const R: Register>;

impl<const R: Register> Operation for STORE<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let value = R.get(cpu);

        bus.write(cpu.address(), value);

        Poll::Ready(())
    }
}

pub type STA = STORE<{ A }>;
pub type STX = STORE<{ X }>;
pub type STY = STORE<{ Y }>;

/// Stores `A & X & (base_high + 1)` into memory (`SHA`/`AHX`/`AXA`).
///
/// `base_high` is the high byte of the effective address *before* Y-indexing, so the
/// stored value is partly derived from the destination address itself. On a page cross,
/// the hardware drops the carry into the address bus, the result ANDs into the high byte
/// of the address and the write lands at `(result << 8) | low` rather than the true
/// effective address. No flags are modified.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct SHA;

impl Operation for SHA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
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

/// Stores `R & (base_high + 1)` into memory (`SHY`, `SHX`).
///
/// `base_high` is the high byte of the effective address *before* indexing by `IDX`,
/// same construction as [`SHA`]. On a page cross the hardware drops the carry into the
/// address bus, the result ANDs into the high byte of the address and the write lands
/// at `(result << 8) | low` rather than the true effective address. No flags are
/// modified.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct SH<const R: Register, const IDX: Register>;

impl<const R: Register, const IDX: Register> Operation for SH<R, IDX> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
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

/// Sets `SP = A & X`, then stores `SP & (base_high + 1)` into memory
/// (`TAS`/`SHS`/`XAS`).
///
/// An undocumented combination. After the `SP` assignment, `SP & (base_high + 1)` is
/// algebraically equal to `A & X & (base_high + 1)`, which is exactly what [`SHA`]
/// computes, so the page-cross address-bus corruption behaves identically here.
/// `TAS` overwriting `SP` makes it nearly impossible to use safely, the running program
/// loses its stack the instant this opcode retires. No flags are modified.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct TAS;

impl Operation for TAS {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Write);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // SHA computes A & X & (base_high + 1) and handles the page-cross bus conflict.
        // Setting SP = A & X first means SP & (base_high + 1) == A & X & (base_high + 1),
        // so SHA's existing logic is correct here without any changes.
        cpu.sp = cpu.a & cpu.x;
        SHA::apply(cpu, bus)
    }
}

/// Copies register `SRC` into register `DST` in a single implicit cycle (`TAX`, `TAY`,
/// `TSX`, `TXA`, `TYA`, `TXS`).
///
/// Updates `Z` and `N` from the moved value whenever the destination is `A`, `X`, or
/// `Y`. `TXS` is the sole exception and leaves every flag untouched, which is what makes
/// it safe to drop into reset and interrupt prologues without disturbing the captured
/// state.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#TAX>.
pub struct TRANSFER<const SRC: Register, const DST: Register>;

impl<const SRC: Register, const DST: Register> Operation for TRANSFER<SRC, DST> {
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
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

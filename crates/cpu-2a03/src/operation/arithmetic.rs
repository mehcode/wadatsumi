// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Arithmetic operations: addition, subtraction, increment, decrement, and comparison.
//!
//! Unlike [`logical`], operands here are treated as numbers. `ADC` and `SBC` update `C`
//! and `V` (in addition to `Z` and `N`) to surface unsigned carry and signed overflow
//! across byte boundaries, so chained multi-byte arithmetic can stitch the result back
//! together one byte at a time.
//!
//! See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for the canonical
//! per-instruction reference.

use std::task::Poll;

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::operation::Register::{self, A, X, Y};
use crate::operation::{MemoryAccess, Operand, Operation};
use crate::status::CpuStatus;

/// Adds the byte at the effective address and the carry flag into the accumulator (`ADC`).
///
/// Stores the result in `A`. Sets `C` on unsigned carry-out and `V` on signed overflow,
/// so consecutive `ADC`s can chain across multi-byte numbers without losing bits.
/// Updates `Z`, `N`, `C`, and `V`.
///
/// The 2A03 omits the 65xx decimal mode entirely, so the `D` flag has no effect here
/// even when it is set, this implementation always computes in plain binary.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#ADC>.
pub struct ADC;

impl Operation for ADC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result =
            u16::from(cpu.a) + u16::from(cpu.data) + u16::from(cpu.p.contains(CpuStatus::C));

        cpu.p.update_zn(result as u8);
        cpu.p.set(CpuStatus::C, result > 0xFF);
        cpu.p.set(CpuStatus::V, (!(cpu.a ^ cpu.data) & (cpu.a ^ result as u8)) & 0x80 != 0);

        cpu.a = result as u8;

        Poll::Ready(())
    }
}

/// Subtracts the byte at the effective address from register `R` and discards the result
/// (`CMP`, `CPX`, `CPY`).
///
/// The register is left untouched, only the flags move. Sets `C` when the register is
/// greater than or equal to the operand (no borrow), clears it on borrow, and updates
/// `Z` and `N` from the difference. `V` is never touched, which is what distinguishes
/// `CMP` from a discarded `SBC` and lets branches read each flag independently.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#CMP>.
pub struct COMPARE<const R: Register>;

impl<const R: Register> Operation for COMPARE<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let value = R.get(cpu);
        let result = value.wrapping_sub(cpu.data);

        cpu.p.set(CpuStatus::C, value >= cpu.data);
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

pub type CMP = COMPARE<{ A }>;
pub type CPX = COMPARE<{ X }>;
pub type CPY = COMPARE<{ Y }>;

/// Decrement memory by one, then compare the result against the accumulator (`DCP`).
///
/// An undocumented combination of `DEC` and `CMP` sharing a single read-modify-write
/// pipeline. Writes the decremented byte back to memory, then sets `C` when
/// `A >= result` (no borrow) and updates `Z` and `N` from `A - result`. `A` is not
/// modified.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct DCP;

impl Operation for DCP {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // Inline the decrement-and-writeback rather than delegating to DEC: DEC does not write
        // cpu.data (to avoid a dead store on standalone DEC), so the result must be placed in
        // cpu.data explicitly here for CMP to consume it.
        cpu.data = cpu.data.wrapping_sub(1);
        bus.write(cpu.address(), cpu.data);
        CMP::apply(cpu, bus)
    }
}

/// Decrements operand `O` by one (`DEC`, `DEX`, `DEY`).
///
/// Memory operands go through the read-modify-write pipeline (the spurious write of the
/// original byte then the real write of the decremented byte). Register operands resolve
/// in a single implicit cycle. Wraps from `$00` to `$FF`. Updates `Z` and `N`, leaves
/// `C` and `V` alone, so this can sit inside a carry-propagating arithmetic sequence
/// without trashing it.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#DEC>.
pub struct DECREMENT<const O: Operand>;

impl<const O: Operand> Operation for DECREMENT<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let result = O.read(cpu).wrapping_sub(1);

        O.write(cpu, bus, result);
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

pub type DEC = DECREMENT<{ Operand::Memory }>;
pub type DEX = DECREMENT<{ Operand::Register(X) }>;
pub type DEY = DECREMENT<{ Operand::Register(Y) }>;

/// Increments operand `O` by one (`INC`, `INX`, `INY`).
///
/// Memory operands go through the read-modify-write pipeline (the spurious write of the
/// original byte then the real write of the incremented byte). Register operands resolve
/// in a single implicit cycle. Wraps from `$FF` to `$00`. Updates `Z` and `N`, leaves
/// `C` and `V` alone.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#INC>.
pub struct INCREMENT<const O: Operand>;

impl<const O: Operand> Operation for INCREMENT<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let result = O.read(cpu).wrapping_add(1);

        O.write(cpu, bus, result);
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

pub type INC = INCREMENT<{ Operand::Memory }>;
pub type INX = INCREMENT<{ Operand::Register(X) }>;
pub type INY = INCREMENT<{ Operand::Register(Y) }>;

/// Increment memory by one, then subtract the result from the accumulator (`ISC`/`ISB`).
///
/// An undocumented combination of `INC` and `SBC` sharing a single read-modify-write
/// pipeline. Writes the incremented byte back to memory, then performs the SBC against
/// `A` with the carry flag acting as the inverted borrow input.
/// Updates `Z`, `N`, `C`, and `V`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct ISC;

impl Operation for ISC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // Same pattern as DCP: inline the increment-and-writeback so that SBC reads the
        // incremented value from cpu.data without INC needing a cpu.data write of its own.
        cpu.data = cpu.data.wrapping_add(1);
        bus.write(cpu.address(), cpu.data);
        SBC::apply(cpu, bus)
    }
}

/// Subtracts the byte at the effective address and the borrow from the accumulator (`SBC`).
///
/// The 6502 defines borrow as the inverse of `C`, so `C` must be set before the first
/// `SBC` in a chain to avoid a phantom borrow. Stores the result in `A`. Sets `C` when
/// no borrow was needed (the unsigned result fits in a byte), and `V` on signed overflow.
/// Updates `Z`, `N`, `C`, and `V`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#SBC>.
pub struct SBC;

impl Operation for SBC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // The 6502 defines SBC as A - M - (1 - C), which is identical to A + ~M + C.
        // Inverting the operand before delegating to ADC exploits this equivalence so
        // all flag updates (C, V, Z, N) fall out of the shared addition logic for free.
        cpu.data = !cpu.data;

        ADC::apply(cpu, bus)
    }
}

/// ANDs `A` with `X`, subtracts the byte at the effective address, and stores the result
/// in `X` (`SBX`/`AXS`/`SAX`).
///
/// An undocumented hybrid of `CMP` (it sets `C` like a compare rather than chaining
/// borrow through it) and a destination-`X` subtraction. Sets `C` when `A & X` is greater
/// than or equal to the operand (no borrow). Updates `Z` and `N`. `V` is left alone.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct SBX;

impl Operation for SBX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let value = cpu.a & cpu.x;
        let result = value.wrapping_sub(cpu.data);

        cpu.p.set(CpuStatus::C, value >= cpu.data);
        cpu.p.update_zn(result);
        cpu.x = result;

        Poll::Ready(())
    }
}

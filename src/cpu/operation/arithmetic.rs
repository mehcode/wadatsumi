// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Arithmetic operations: addition, subtraction, increment, decrement, and comparison.
//! Unlike [`logical`], operands are treated as numbers — `ADC` and `SBC` update `C` and `V`
//! in addition to `Z` and `N`, reflecting carries and borrows across byte boundaries.

use std::task::Poll;

use crate::Bus;
use crate::cpu::operation::Register::{self, A, X, Y};
use crate::cpu::operation::{MemoryAccess, Operand, Operation};
use crate::cpu::{Cpu, CpuStatus};

/// Adds the accumulator, a byte from the effective address, and the carry flag (`ADC`).
/// Stores the result in `A`. Updates `Z`, `N`, `C`, and `V`.
pub struct ADC;

impl Operation for ADC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let result = u16::from(cpu.state.a)
            + u16::from(cpu.data)
            + u16::from(cpu.state.p.contains(CpuStatus::C));

        cpu.state.p.update_zn(result as u8);
        cpu.state.p.set(CpuStatus::C, result > 0xFF);
        cpu.state.p.set(
            CpuStatus::V,
            (!(cpu.state.a ^ cpu.data) & (cpu.state.a ^ result as u8)) & 0x80 != 0,
        );

        cpu.state.a = result as u8;

        Poll::Ready(())
    }
}

/// Subtracts the byte at the effective address from register `R` without storing the result (`CMP`, `CPX`, `CPY`).
/// Sets `C` if the register is greater than or equal to the operand (no borrow), clears it otherwise.
/// Updates `Z` and `N` from the difference.
pub struct COMPARE<const R: Register>;

impl<const R: Register> Operation for COMPARE<R> {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        let value = R.get(&cpu.state);
        let result = value.wrapping_sub(cpu.data);

        cpu.state.p.set(CpuStatus::C, value >= cpu.data);
        cpu.state.p.update_zn(result);

        Poll::Ready(())
    }
}

pub type CMP = COMPARE<{ A }>;
pub type CPX = COMPARE<{ X }>;
pub type CPY = COMPARE<{ Y }>;

/// Decrement memory by one, then compare the result with the accumulator (`DCP`).
/// Uses the read-modify-write pipeline. Sets `C` if `A >= result`. Updates `Z` and `N`.
pub struct DCP;

impl Operation for DCP {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data.wrapping_sub(1);

        bus.write(cpu.address, value);

        cpu.state.p.set(CpuStatus::C, cpu.state.a >= value);
        cpu.state.p.update_zn(cpu.state.a.wrapping_sub(value));

        Poll::Ready(())
    }
}

/// Decrements operand `O` by one (`DEC`, `DEX`, `DEY`).
/// For memory operands, uses the read-modify-write pipeline (spurious write then final write).
/// For register operands, executes in a single implicit cycle.
/// Updates `Z` and `N`.
pub struct DECREMENT<const O: Operand>;

impl<const O: Operand> Operation for DECREMENT<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu).wrapping_sub(1);

        O.write(cpu, bus, value);

        cpu.state.p.update_zn(value);

        Poll::Ready(())
    }
}

pub type DEC = DECREMENT<{ Operand::Memory }>;
pub type DEX = DECREMENT<{ Operand::Register(X) }>;
pub type DEY = DECREMENT<{ Operand::Register(Y) }>;

/// Increments operand `O` by one (`INC`, `INX`, `INY`).
/// For memory operands, uses the read-modify-write pipeline (spurious write then final write).
/// For register operands, executes in a single implicit cycle.
/// Updates `Z` and `N`.
pub struct INCREMENT<const O: Operand>;

impl<const O: Operand> Operation for INCREMENT<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu).wrapping_add(1);

        O.write(cpu, bus, value);

        cpu.state.p.update_zn(value);

        Poll::Ready(())
    }
}

pub type INC = INCREMENT<{ Operand::Memory }>;
pub type INX = INCREMENT<{ Operand::Register(X) }>;
pub type INY = INCREMENT<{ Operand::Register(Y) }>;

/// Increment memory by one, then subtract the result from the accumulator (`ISC`).
/// Uses the read-modify-write pipeline. Updates `Z`, `N`, `C`, and `V`.
pub struct ISC;

impl Operation for ISC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data.wrapping_add(1);

        bus.write(cpu.address, value);

        // Invert the operand and forward to ADC (same as SBC)
        // to handle the shared addition logic for C, V, Z, and N.
        cpu.data = !value;
        ADC::apply(cpu, bus)
    }
}

/// Subtracts a byte at the effective address and the borrow from the accumulator (`SBC`).
/// Stores the result in `A`. Updates `Z`, `N`, `C`, and `V`.
pub struct SBC;

impl Operation for SBC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        // The 6502 defines SBC as A - M - (1 - C), which is identical to A + ~M + C.
        // Inverting the operand before delegating to ADC exploits this equivalence so
        // all flag updates (C, V, Z, N) fall out of the shared addition logic for free.
        cpu.data = !cpu.data;

        ADC::apply(cpu, bus)
    }
}

/// ANDs the accumulator with `X`, subtracts the byte at the effective address, and stores
/// the result in `X` (`SBX`). Sets `C` if `A & X >= operand` (no borrow). Updates `Z` and `N`.
/// Does not affect `V`.
pub struct SBX;

impl Operation for SBX {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        let value = cpu.state.a & cpu.state.x;
        let result = value.wrapping_sub(cpu.data);

        cpu.state.p.set(CpuStatus::C, value >= cpu.data);
        cpu.state.p.update_zn(result);
        cpu.state.x = result;

        Poll::Ready(())
    }
}

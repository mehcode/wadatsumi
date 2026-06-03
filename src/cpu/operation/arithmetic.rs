// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Arithmetic operations: addition, subtraction, increment, decrement, and comparison.
//! Unlike [`logical`], operands are treated as numbers — `ADC` and `SBC` update `C` and `V`
//! in addition to `Z` and `N`, reflecting carries and borrows across byte boundaries.

use std::task::Poll;

use crate::Bus;
use crate::cpu::operation::Register::{self, A, X, Y};
use crate::cpu::operation::{MemoryAccess, Operation};
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

/// Decrements a byte in memory by one using the read-modify-write pipeline (`DEC`).
/// Reads the value, performs a spurious write with the original byte, then writes the decremented result.
/// Updates `Z` and `N`.
pub struct DEC;

impl Operation for DEC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data.wrapping_sub(1);

        bus.write(cpu.address, value);

        cpu.state.p.update_zn(value);

        Poll::Ready(())
    }
}

/// Decrements register `R` by one in a single implicit cycle (`DEX`, `DEY`).
/// Updates `Z` and `N` to reflect the new value.
pub struct DECREMENT<const R: Register>;

impl<const R: Register> Operation for DECREMENT<R> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        let value = R.get(&cpu.state).wrapping_sub(1);

        R.set(&mut cpu.state, value);

        cpu.state.p.update_zn(value);

        Poll::Ready(())
    }
}

pub type DEX = DECREMENT<{ X }>;
pub type DEY = DECREMENT<{ Y }>;

/// Increments a byte in memory by one using the read-modify-write pipeline (`INC`).
/// Reads the value, performs a spurious write with the original byte, then writes the incremented result.
/// Updates `Z` and `N`.
pub struct INC;

impl Operation for INC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data.wrapping_add(1);

        bus.write(cpu.address, value);

        cpu.state.p.update_zn(value);

        Poll::Ready(())
    }
}

/// Increments register `R` by one in a single implicit cycle (`INX`, `INY`).
/// Updates `Z` and `N` to reflect the new value.
pub struct INCREMENT<const R: Register>;

impl<const R: Register> Operation for INCREMENT<R> {
    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> Poll<()> {
        let value = R.get(&cpu.state).wrapping_add(1);

        R.set(&mut cpu.state, value);

        cpu.state.p.update_zn(value);

        Poll::Ready(())
    }
}

pub type INX = INCREMENT<{ X }>;
pub type INY = INCREMENT<{ Y }>;

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

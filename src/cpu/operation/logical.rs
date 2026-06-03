// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bitwise logical operations on the accumulator: `AND`, `EOR`, `ORA`, and `BIT`.
//! Unlike [`arithmetic`], operands are treated as bit patterns — no carry or overflow is produced.
//! Shifts and rotates (`ASL`, `LSR`, `ROL`, `ROR`) belong here when added.

use std::task::Poll;

use crate::Bus;
use crate::cpu::operation::{MemoryAccess, Operation};
use crate::cpu::{Cpu, CpuStatus};

/// Bitwise AND of the accumulator with a byte from the effective address (`AND`).
/// Stores the result in `A`. Updates `Z` and `N`.
pub struct AND;

impl Operation for AND {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let result = cpu.state.a & cpu.data;

        cpu.state.a = result;
        cpu.state.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Tests bits in memory against the accumulator (`BIT`).
/// Sets `Z` from `A & data`, copies bit 7 of `data` to `N`, and bit 6 of `data` to `V`. `A` is not modified.
pub struct BIT;

impl Operation for BIT {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        cpu.state.p.update_z(cpu.state.a & cpu.data);
        cpu.state.p.set(CpuStatus::N, cpu.data & 0x80 != 0);
        cpu.state.p.set(CpuStatus::V, cpu.data & 0x40 != 0);

        Poll::Ready(())
    }
}

/// Bitwise exclusive OR of the accumulator with a byte from the effective address (`EOR`).
/// Stores the result in `A`. Updates `Z` and `N`.
pub struct EOR;

impl Operation for EOR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let result = cpu.state.a ^ cpu.data;

        cpu.state.a = result;
        cpu.state.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Bitwise OR of the accumulator with a byte from the effective address (`ORA`).
/// Stores the result in `A`. Updates `Z` and `N`.
pub struct ORA;

impl Operation for ORA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
        let result = cpu.state.a | cpu.data;

        cpu.state.a = result;
        cpu.state.p.update_zn(result);

        Poll::Ready(())
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bitwise logical operations on the accumulator: `AND`, `EOR`, `ORA`, and `BIT`.
//! Unlike [`arithmetic`], operands are treated as bit patterns — no carry or overflow is produced.
//! Shifts and rotates (`ASL`, `LSR`, `ROL`, `ROR`) belong here when added.

use std::task::Poll;

use crate::Bus;
use crate::cpu::operation::{ADC, MemoryAccess, Operand, Operation};
use crate::cpu::{Cpu2A03, CpuStatus};

/// AND accumulator with immediate byte, then LSR the accumulator (`ALR`).
/// Sets `C` from bit 0 before the shift. Stores the result in `A`. Updates `Z`, `N`, and `C`.
pub struct ALR;

impl Operation for ALR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let value = cpu.a & cpu.data;
        let result = value >> 1;

        cpu.a = result;
        cpu.p.set(CpuStatus::C, value & 0x01 != 0);
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// AND accumulator with immediate byte, then copy bit 7 of result to carry (`ANC`).
/// Stores the result in `A`. Updates `Z`, `N`, and `C`.
pub struct ANC;

impl Operation for ANC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        // ANC is AND with an extra flag: delegate to AND for the shared AND + Z/N update,
        // then copy the sign bit of the result into C.
        let _ = AND::apply(cpu, bus);

        cpu.p.set(CpuStatus::C, cpu.a & 0x80 != 0);

        Poll::Ready(())
    }
}

/// Bitwise AND of the accumulator with a byte from the effective address (`AND`).
/// Stores the result in `A`. Updates `Z` and `N`.
pub struct AND;

impl Operation for AND {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let result = cpu.a & cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// AND accumulator with immediate byte, then rotate right through carry (`ARR`).
/// Unlike a plain `AND`+`ROR`, `C` is taken from bit 6 of the result and `V` from bit 6 XOR bit 5.
/// Stores the result in `A`. Updates `Z`, `N`, `C`, and `V`.
pub struct ARR;

impl Operation for ARR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let value = cpu.a & cpu.data;
        let result = (value >> 1) | (u8::from(cpu.p.contains(CpuStatus::C)) << 7);

        cpu.a = result;
        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, result & 0x40 != 0);

        // V detects a carry mismatch between BCD digits: not ADC-style overflow.
        cpu.p.set(CpuStatus::V, ((result >> 5) ^ (result >> 6)) & 1 != 0);

        Poll::Ready(())
    }
}

/// Shifts operand `O` one bit left, filling bit 0 with zero (`ASL`).
/// For memory operands, uses the read-modify-write pipeline (spurious write then final write).
/// For register operands, executes in a single implicit cycle.
/// Sets `C` to the original bit 7. Updates `Z` and `N`.
pub struct ASL<const O: Operand>;

impl<const O: Operand> Operation for ASL<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = value << 1;

        O.write(cpu, bus, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x80 != 0);

        Poll::Ready(())
    }
}

/// Tests bits in memory against the accumulator (`BIT`).
/// Sets `Z` from `A & data`, copies bit 7 of `data` to `N`, and bit 6 of `data` to `V`. `A` is not modified.
pub struct BIT;

impl Operation for BIT {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        cpu.p.update_z(cpu.a & cpu.data);
        cpu.p.set(CpuStatus::N, cpu.data & 0x80 != 0);
        cpu.p.set(CpuStatus::V, cpu.data & 0x40 != 0);

        Poll::Ready(())
    }
}

/// Bitwise exclusive OR of the accumulator with a byte from the effective address (`EOR`).
/// Stores the result in `A`. Updates `Z` and `N`.
pub struct EOR;

impl Operation for EOR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let result = cpu.a ^ cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Shifts operand `O` one bit right, filling bit 7 with zero (`LSR`).
/// For memory operands, uses the read-modify-write pipeline (spurious write then final write).
/// For register operands, executes in a single implicit cycle.
/// Sets `C` to the original bit 0. Updates `Z` and `N`.
pub struct LSR<const O: Operand>;

impl<const O: Operand> Operation for LSR<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = value >> 1;

        O.write(cpu, bus, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x01 != 0);

        Poll::Ready(())
    }
}

/// Bitwise OR of the accumulator with a byte from the effective address (`ORA`).
/// Stores the result in `A`. Updates `Z` and `N`.
pub struct ORA;

impl Operation for ORA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<()> {
        let result = cpu.a | cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Rotates operand `O` one bit left through the carry flag (`ROL`).
/// For memory operands, uses the read-modify-write pipeline (spurious write then final write).
/// For register operands, executes in a single implicit cycle.
/// Bit 0 is filled with `C`; sets `C` to the original bit 7. Updates `Z` and `N`.
pub struct ROL<const O: Operand>;

impl<const O: Operand> Operation for ROL<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = (value << 1) | u8::from(cpu.p.contains(CpuStatus::C));

        O.write(cpu, bus, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x80 != 0);

        Poll::Ready(())
    }
}

/// Rotates operand `O` one bit right through the carry flag (`ROR`).
/// For memory operands, uses the read-modify-write pipeline (spurious write then final write).
/// For register operands, executes in a single implicit cycle.
/// Bit 7 is filled with `C`; sets `C` to the original bit 0. Updates `Z` and `N`.
pub struct ROR<const O: Operand>;

impl<const O: Operand> Operation for ROR<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = (value >> 1) | (u8::from(cpu.p.contains(CpuStatus::C)) << 7);

        O.write(cpu, bus, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x01 != 0);

        Poll::Ready(())
    }
}

/// Rotate memory left through carry, then AND the result into the accumulator (`RLA`).
/// Uses the read-modify-write pipeline. Sets `C` from original bit 7. Updates `Z` and `N` from `A`.
pub struct RLA;

impl Operation for RLA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data;
        let rotated = (value << 1) | u8::from(cpu.p.contains(CpuStatus::C));

        bus.write(cpu.address(),rotated);

        cpu.p.set(CpuStatus::C, value & 0x80 != 0);
        cpu.a &= rotated;
        cpu.p.update_zn(cpu.a);

        Poll::Ready(())
    }
}

/// Rotate memory right through carry, then add the result to the accumulator (`RRA`).
/// Uses the read-modify-write pipeline. Bit 7 is filled with old `C`; sets `C` from bit 0,
/// then that new `C` feeds into the ADC. Updates `Z`, `N`, `C`, and `V`.
pub struct RRA;

impl Operation for RRA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data;

        cpu.data = (value >> 1) | (u8::from(cpu.p.contains(CpuStatus::C)) << 7);

        bus.write(cpu.address(),cpu.data);

        // The ROR carry-out (bit 0 of original) becomes the carry-in for ADC.
        cpu.p.set(CpuStatus::C, value & 0x01 != 0);

        ADC::apply(cpu, bus)
    }
}

/// Shift memory left one bit, then OR the result into the accumulator (`SLO`).
/// Uses the read-modify-write pipeline. Sets `C` to the original bit 7. Updates `Z` and `N` from `A`.
pub struct SLO;

impl Operation for SLO {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data;
        let shifted = value << 1;

        bus.write(cpu.address(),shifted);

        cpu.p.set(CpuStatus::C, value & 0x80 != 0);
        cpu.a |= shifted;
        cpu.p.update_zn(cpu.a);

        Poll::Ready(())
    }
}

/// Shift memory right one bit, then EOR the result into the accumulator (`SRE`).
/// Uses the read-modify-write pipeline. Sets `C` to the original bit 0. Updates `Z` and `N` from `A`.
pub struct SRE;

impl Operation for SRE {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()> {
        let value = cpu.data;
        let shifted = value >> 1;

        bus.write(cpu.address(),shifted);

        cpu.p.set(CpuStatus::C, value & 0x01 != 0);
        cpu.a ^= shifted;
        cpu.p.update_zn(cpu.a);

        Poll::Ready(())
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bitwise logical operations and the shift/rotate family: `AND`, `EOR`, `ORA`, `BIT`,
//! `ASL`, `LSR`, `ROL`, `ROR`, plus the undocumented combinations (`ALR`, `ANC`, `ARR`,
//! `RLA`, `RRA`, `SLO`, `SRE`).
//!
//! Unlike [`arithmetic`], operands here are treated as bit patterns, no signed-overflow
//! semantics are applied. The plain logical ops never touch `C` or `V`, while the shifts
//! and rotates surface the bit shifted out through `C` so software can stitch shifts
//! across multiple bytes.
//!
//! See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for the per-instruction
//! reference.

use std::task::{Poll, ready};

use crate::CpuReadWrite;
use crate::cpu::Cpu2A03;
use crate::operation::Operand::{self, Memory, Register};
use crate::operation::Register::A;
use crate::operation::{ADC, MemoryAccess, Operation};
use crate::status::CpuStatus;

/// AND the accumulator with an immediate byte, then `LSR` the accumulator in place
/// (`ALR`/`ASR`).
///
/// An undocumented fused `AND` plus `LSR`. `C` ends up holding bit 0 of the AND result
/// (the bit shifted out), with `A` holding the shifted value. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct ALR;

impl Operation for ALR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // AND reads cpu.data (the immediate byte) into A; LSR then shifts A in place.
        // The register variant of LSR needs no cpu.data latch, it reads and writes A directly.
        ready!(AND::apply(cpu, bus));

        LSR::<{ Register(A) }>::apply(cpu, bus)
    }
}

/// AND the accumulator with an immediate byte, then copy bit 7 of the result into `C`
/// (`ANC`/`AAC`).
///
/// An undocumented combination of `AND` and the bit-7-to-carry behaviour of `ROL`. `C`
/// effectively mirrors `N` after the AND, which makes signed-bit branches doable without
/// an explicit shift. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct ANC;

impl Operation for ANC {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // ANC is AND with an extra flag: delegate to AND for the shared AND + Z/N update,
        // then copy the sign bit of the result into C.
        ready!(AND::apply(cpu, bus));

        cpu.p.set(CpuStatus::C, cpu.a & 0x80 != 0);

        Poll::Ready(())
    }
}

/// Bitwise AND of the accumulator with the byte at the effective address (`AND`).
///
/// Stores the result in `A`. Updates `Z` and `N`, leaves `C` and `V` alone.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#AND>.
pub struct AND;

impl Operation for AND {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result = cpu.a & cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// AND the accumulator with an immediate byte, then rotate the result right through `C`
/// (`ARR`).
///
/// Looks like `AND` + `ROR` but the flag semantics are weirder: `C` comes from bit 6 of
/// the result (not bit 0 of the input) and `V` is bit 6 XOR bit 5, mimicking the BCD-mode
/// adder flags from the original NMOS 6502. Updates `Z`, `N`, `C`, and `V`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct ARR;

impl Operation for ARR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
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
///
/// Memory operands go through the read-modify-write pipeline (the spurious write of the
/// original byte then the real write of the shifted byte). Register operands resolve in
/// a single implicit cycle. `C` receives the original bit 7, making chained left shifts
/// across multi-byte values straightforward. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#ASL>.
pub struct ASL<const O: Operand>;

impl<const O: Operand> Operation for ASL<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = value << 1;

        O.write(cpu, bus, result);
        O.latch(cpu, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x80 != 0);

        Poll::Ready(())
    }
}

/// Tests bits in memory against the accumulator (`BIT`).
///
/// Sets `Z` from `A & data`, copies bit 7 of the memory byte into `N`, and bit 6 into
/// `V`. `A` is not modified. Frequently paired with `BMI`/`BPL`/`BVS`/`BVC` for cheap
/// one-bit flag polling, the PPU's `$2002` register layout is designed around this
/// trick: bits 6 and 7 (sprite-0 hit and vblank) land directly in `V` and `N`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#BIT>.
pub struct BIT;

impl Operation for BIT {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        cpu.p.update_z(cpu.a & cpu.data);
        cpu.p.set(CpuStatus::N, cpu.data & 0x80 != 0);
        cpu.p.set(CpuStatus::V, cpu.data & 0x40 != 0);

        Poll::Ready(())
    }
}

/// Bitwise exclusive-OR of the accumulator with the byte at the effective address (`EOR`).
///
/// Stores the result in `A`. Updates `Z` and `N`, leaves `C` and `V` alone.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#EOR>.
pub struct EOR;

impl Operation for EOR {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result = cpu.a ^ cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Shifts operand `O` one bit right, filling bit 7 with zero (`LSR`).
///
/// Memory operands go through the read-modify-write pipeline (the spurious write of the
/// original byte then the real write of the shifted byte). Register operands resolve in
/// a single implicit cycle. `C` receives the original bit 0. `N` is always cleared by
/// definition (bit 7 ends at zero), so this is a quick way to clear `N` if you don't
/// mind also clobbering the value. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#LSR>.
pub struct LSR<const O: Operand>;

impl<const O: Operand> Operation for LSR<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = value >> 1;

        O.write(cpu, bus, result);
        O.latch(cpu, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x01 != 0);

        Poll::Ready(())
    }
}

/// Bitwise OR of the accumulator with the byte at the effective address (`ORA`).
///
/// Stores the result in `A`. Updates `Z` and `N`, leaves `C` and `V` alone.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#ORA>.
pub struct ORA;

impl Operation for ORA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::Read);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, _: &mut B) -> Poll<()> {
        let result = cpu.a | cpu.data;

        cpu.a = result;
        cpu.p.update_zn(result);

        Poll::Ready(())
    }
}

/// Rotates operand `O` one bit left through the carry flag (`ROL`).
///
/// Bit 0 is filled with the *incoming* `C`, then the *outgoing* `C` receives the original
/// bit 7, so a chain of `ROL`s across consecutive bytes performs a multi-byte left shift.
/// Memory operands go through the read-modify-write pipeline (the spurious write of the
/// original byte then the real write of the rotated byte). Register operands resolve in
/// a single implicit cycle. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#ROL>.
pub struct ROL<const O: Operand>;

impl<const O: Operand> Operation for ROL<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = (value << 1) | u8::from(cpu.p.contains(CpuStatus::C));

        O.write(cpu, bus, result);
        O.latch(cpu, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x80 != 0);

        Poll::Ready(())
    }
}

/// Rotates operand `O` one bit right through the carry flag (`ROR`).
///
/// Bit 7 is filled with the *incoming* `C`, then the *outgoing* `C` receives the original
/// bit 0, mirroring `ROL` for multi-byte right shifts. Memory operands go through the
/// read-modify-write pipeline (the spurious write of the original byte then the real
/// write of the rotated byte). Register operands resolve in a single implicit cycle.
/// Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html#ROR>.
pub struct ROR<const O: Operand>;

impl<const O: Operand> Operation for ROR<O> {
    const ACCESS: Option<MemoryAccess> = O.access(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        let value = O.read(cpu);
        let result = (value >> 1) | (u8::from(cpu.p.contains(CpuStatus::C)) << 7);

        O.write(cpu, bus, result);
        O.latch(cpu, result);

        cpu.p.update_zn(result);
        cpu.p.set(CpuStatus::C, value & 0x01 != 0);

        Poll::Ready(())
    }
}

/// Rotate memory left through carry, then AND the result into the accumulator (`RLA`).
///
/// An undocumented fused `ROL` + `AND` sharing one read-modify-write pipeline. `C`
/// receives the original bit 7 of memory (from the rotate), and `Z`/`N` come from the
/// final value of `A`. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct RLA;

impl Operation for RLA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // ROL (Memory) writes the rotated value to both the bus and cpu.data; AND then reads
        // cpu.data so no redundant bus read is needed between the two halves.
        ready!(ROL::<{ Memory }>::apply(cpu, bus));

        AND::apply(cpu, bus)
    }
}

/// Rotate memory right through carry, then add the result to the accumulator (`RRA`).
///
/// An undocumented fused `ROR` + `ADC` sharing one read-modify-write pipeline. The
/// rotate fills bit 7 with the *old* `C` and pushes the original bit 0 out into `C`,
/// then the `ADC` consumes that new `C` as its carry-in.
/// Updates `Z`, `N`, `C`, and `V`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct RRA;

impl Operation for RRA {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // ROR sets C = original bit 0, which becomes the carry-in for ADC.
        ready!(ROR::<{ Memory }>::apply(cpu, bus));

        ADC::apply(cpu, bus)
    }
}

/// Shift memory left one bit, then OR the result into the accumulator (`SLO`/`ASO`).
///
/// An undocumented fused `ASL` + `ORA` sharing one read-modify-write pipeline. `C`
/// receives the original bit 7 of memory (from the shift), and `Z`/`N` come from the
/// final value of `A`. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct SLO;

impl Operation for SLO {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // ASL (Memory) writes the shifted value to both the bus and cpu.data; ORA then reads
        // cpu.data so no redundant bus read is needed between the two halves.
        ready!(ASL::<{ Memory }>::apply(cpu, bus));

        ORA::apply(cpu, bus)
    }
}

/// Shift memory right one bit, then EOR the result into the accumulator (`SRE`/`LSE`).
///
/// An undocumented fused `LSR` + `EOR` sharing one read-modify-write pipeline. `C`
/// receives the original bit 0 of memory (from the shift), and `Z`/`N` come from the
/// final value of `A`. Updates `Z`, `N`, and `C`.
///
/// See <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>.
pub struct SRE;

impl Operation for SRE {
    const ACCESS: Option<MemoryAccess> = Some(MemoryAccess::ReadModifyWrite);

    #[inline]
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()> {
        // LSR (Memory) writes the shifted value to both the bus and cpu.data; EOR then reads
        // cpu.data so no redundant bus read is needed between the two halves.
        ready!(LSR::<{ Memory }>::apply(cpu, bus));

        EOR::apply(cpu, bus)
    }
}

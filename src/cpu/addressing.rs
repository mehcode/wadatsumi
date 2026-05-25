// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bus::Bus;
use crate::cpu::Cpu;

/// Determines how an instruction locates its operand.
///
/// Each 6502 addressing mode performs the bus reads required to produce an effective address in
/// `cpu.address`, consuming exactly the cycles the real hardware would. Instructions then read
/// or write through that address without knowing how it was resolved.
pub trait AddressingMode {
    /// Advance resolution by one cycle and return whether the effective address is ready.
    ///
    /// Returns `true` when `cpu.address` holds the final effective address and the instruction
    /// may proceed to its execute phase. Returns `false` when additional cycles are still needed
    /// (e.g. while a multi-byte address is being fetched or a page-cross penalty is being paid).
    fn resolve<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool
    where
        Self: Sized;
}

/// The operand is encoded in the opcode itself; no memory address is needed (e.g. `CLC`, `TAX`).
/// The 6502 still performs a spurious read of the next byte as part of its fetch pipeline,
/// which is discarded before execution begins.
pub struct Implied;

impl AddressingMode for Implied {
    #[inline]
    fn resolve<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool {
        // Spurious pipeline fetch — result is discarded
        let _ = bus.read(cpu.state.pc);

        true // ready to apply
    }
}

/// The operand is the byte immediately following the opcode in the instruction stream (e.g. `LDA #$FF`).
/// PC is advanced past the operand byte so the next fetch returns the following opcode.
pub struct Immediate;

impl AddressingMode for Immediate {
    #[inline]
    fn resolve<B: Bus>(cpu: &mut Cpu<B>, _: &mut B) -> bool {
        cpu.address = cpu.state.pc;
        cpu.state.pc = cpu.state.pc.wrapping_add(1);

        true // ready to apply
    }
}

pub struct ZeroPage;

impl AddressingMode for ZeroPage {
    #[inline]
    fn resolve<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool {
        match cpu.cycle {
            1 => {
                // Fetch the first operand byte from the instruction stream.
                cpu.address = u16::from(cpu.fetch(bus));

                false // not ready to apply
            }

            _ => true, // ready to apply
        }
    }
}

pub struct ZeroPageX;

impl AddressingMode for ZeroPageX {
    #[inline]
    fn resolve<B: Bus>(cpu: &mut Cpu<B>, bus: &mut B) -> bool {
        match cpu.cycle {
            1 => ZeroPage::resolve(cpu, bus),

            2 => {
                let _ = bus.read(cpu.address); // dummy read

                // Offset the effective address by X
                cpu.address = u16::from((cpu.address as u8).wrapping_add(cpu.state.x));

                false // not ready to apply
            }

            _ => true, // ready to apply
        }
    }
}

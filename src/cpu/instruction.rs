// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::addressing::AddressingMode;
use crate::cpu::operation::Operation;

/// Single-cycle callback for one in-flight 6502 instruction.
///
/// Called once per CPU clock tick. Returns `true` when the instruction is
/// complete and the CPU should clear it; `false` when additional cycles are
/// still needed (addressing mode still resolving, or operation not yet done).
pub type Instruction<B> = fn(cpu: &mut Cpu<B>, bus: &mut B) -> bool;

/// Monomorphic handler for a specific `(Operation, AddressingMode)` pair.
pub fn execute<B: Bus, O: Operation, A: AddressingMode>(cpu: &mut Cpu<B>, bus: &mut B) -> bool {
    // Advance address resolution by one cycle; returns false immediately if the
    // effective address isn't ready yet (e.g. mid-fetch or page-cross penalty still pending).
    A::resolve(cpu, bus) &&
        // Apply the mnemonic's effect to CPU and bus state.
        O::apply(cpu, bus)
}

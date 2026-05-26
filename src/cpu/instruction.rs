// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::addressing::AddressingMode;
use crate::cpu::operation::Operation;

/// Single-cycle callback for one in-flight 6502 instruction.
///
/// Called once per CPU clock tick. Returns `Poll::Ready(())` when the instruction is
/// complete and the CPU should clear it; `Poll::Pending` when additional cycles are
/// still needed (addressing mode still resolving, or operation not yet done).
pub type Instruction<B> = fn(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()>;

/// Monomorphic handler for a specific `(Operation, AddressingMode)` pair.
pub fn execute<B: Bus, O: Operation, A: AddressingMode>(cpu: &mut Cpu<B>, bus: &mut B) -> Poll<()> {
    if !cpu.executing {
        // Advance address resolution by one cycle; returns Poll::Pending immediately if the
        // effective address isn't ready yet (e.g. mid-fetch or page-cross penalty still pending).
        let Poll::Ready(prefetched) = A::resolve::<O, _>(cpu, bus) else {
            return Poll::Pending;
        };

        cpu.executing = true;

        // Addressing modes that happen to read the operand as a side-effect of their final timing
        // cycle return it as Some(byte), avoiding a redundant bus access. Otherwise, read operations
        // get their single bus read here so O::apply always finds its operand in cpu.data.
        if let Some(data) = prefetched {
            cpu.data = data;
        } else if O::MODE.is_read() {
            cpu.data = bus.read(cpu.address);
        }
    }

    // Apply the mnemonic's effect to CPU and bus state.
    O::apply(cpu, bus)
}

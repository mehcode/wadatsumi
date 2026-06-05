// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::Cpu2A03;
use crate::cpu::addressing::AddressingMode;
use crate::cpu::operation::{MemoryAccess, Operation};

/// Single-cycle callback for one in-flight 6502 instruction.
///
/// Called once per CPU clock tick. Returns `Poll::Ready(())` when the instruction is
/// complete and the CPU should clear it; `Poll::Pending` when additional cycles are
/// still needed (addressing mode still resolving, or operation not yet done).
pub type Instruction<B> = fn(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<()>;

/// Monomorphic handler for a specific `(Operation, AddressingMode)` pair.
pub fn execute<B: Bus, O: Operation, A: AddressingMode>(
    cpu: &mut Cpu2A03<B>,
    bus: &mut B,
) -> Poll<()> {
    if cpu.executing == 0 {
        // Drive address resolution one cycle forward. Returns Poll::Pending while the effective
        // address is still being assembled (multi-byte fetch, index addition, page-cross penalty).
        // Only when Poll::Ready is returned does cpu.address hold the final effective address.
        let Poll::Ready(prefetched) = A::resolve::<O, _>(cpu, bus) else {
            return Poll::Pending;
        };

        // Snapshot the t-state where addressing completed. The RMW pipeline below uses
        // `cpu.t - cpu.executing` as an addressing-mode-agnostic cycle index so that the
        // three extra RMW bus cycles always occupy the same relative positions regardless
        // of how many cycles address resolution consumed.
        cpu.executing = cpu.t;

        // Some addressing modes (e.g. AbsoluteIndexed with no page cross) issue a bus read on
        // their final resolution cycle and hand back the byte as a prefetch shortcut. For plain
        // Read operations that shortcut is used directly; otherwise the operand is fetched here.
        // RMW operations intentionally skip both branches: their data read happens in the
        // dedicated cycle below so the spurious write always acts on a freshly-latched byte.
        if let Some(data) = prefetched {
            cpu.data = data;
        } else if matches!(O::ACCESS, Some(MemoryAccess::Read)) {
            cpu.data = bus.read(cpu.address);
        }
    }

    // RMW instructions interpose three hardware-mandated bus cycles between address resolution
    // and the final write performed by O::apply.
    if matches!(O::ACCESS, Some(MemoryAccess::ReadModifyWrite)) {
        match cpu.t - cpu.executing {
            0 => {
                // Latch the current value from memory; O::apply derives the new byte from cpu.data.
                cpu.data = bus.read(cpu.address);

                return Poll::Pending;
            }

            1 => {
                // Spurious write: the unmodified byte is driven onto the data bus while the ALU works.
                bus.write(cpu.address, cpu.data);

                return Poll::Pending;
            }

            _ => {}
        }
    }

    // Address resolution is complete and any RMW pre-cycles have been paid.
    // Delegate to the operation to apply its effect and signal completion.
    O::apply(cpu, bus)
}

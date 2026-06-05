// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::task::Poll;

use crate::bus::Bus;
use crate::cpu::Cpu2A03;
use crate::cpu::operation::Register::{self, X, Y};
use crate::cpu::operation::{MemoryAccess, Operation};

/// Determines how an instruction locates its operand.
///
/// Each 6502 addressing mode performs the bus reads required to produce an effective address in
/// `cpu.address`, consuming exactly the cycles the real hardware would. Instructions then read
/// or write through that address without knowing how it was resolved.
pub trait AddressingMode {
    /// Maximum number of cycles this addressing mode takes to resolve.
    ///
    /// For modes that can finish early (e.g. [`AbsoluteIndexed`] read with no page cross),
    /// this is the *longer* path. RMW instructions always take the longer path, so this
    /// constant is the correct base for computing the operation-relative cycle index:
    /// `cpu.t - A::CYCLES` yields 0 on the first RMW cycle regardless of which mode is used.
    const CYCLES: u8;

    /// Advance resolution by one cycle and return the current status.
    ///
    /// Returns `Poll::Pending` when additional cycles are still needed (e.g. while a multi-byte
    /// address is being fetched or a page-cross penalty is being paid).
    ///
    /// Returns `Poll::Ready(None)` when `cpu.address` holds the final effective address and the
    /// caller should issue its own bus read/write.
    ///
    /// Returns `Poll::Ready(Some(byte))` when the addressing mode's last bus cycle read the
    /// operand as a side-effect (e.g. `AbsoluteIndexed` with no page cross); the caller should
    /// use this byte directly and skip any redundant bus read.
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>>
    where
        Self: Sized;
}

/// The instruction operand is implied by the opcode itself; there is no explicit address
/// or data byte (e.g. `CLC`, `TAX`). The 6502 reads the next byte as a pipeline side-effect
/// without advancing PC; that byte is returned so operations may use it if needed.
pub struct Implied;

impl AddressingMode for Implied {
    const CYCLES: u8 = 1;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        // Spurious read surfaced in case the operation wants it (e.g. JSR uses it as ADL).
        Poll::Ready(Some(bus.read(cpu.pc)))
    }
}

/// The operand is the byte immediately following the opcode in the instruction stream (e.g. `LDA #$FF`).
/// PC is advanced past the operand byte so the next fetch returns the following opcode.
pub struct Immediate;

impl AddressingMode for Immediate {
    const CYCLES: u8 = 1;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, _: &mut B) -> Poll<Option<u8>> {
        // Point address at the literal operand byte sitting at PC, then skip past it.
        cpu.address = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(1);

        Poll::Ready(None)
    }
}

/// The single-byte operand is a zero-page address (`$00nn`), restricting effective addresses to
/// `$0000–$00FF`. One cycle faster than [`Absolute`] because only one address byte is fetched.
/// Syntax: `nn` (one byte).
pub struct ZeroPage;

impl AddressingMode for ZeroPage {
    const CYCLES: u8 = 2;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 => {
                // Fetch zero-page address byte.
                cpu.address = u16::from(cpu.fetch(bus));

                Poll::Pending
            }

            _ => Poll::Ready(None),
        }
    }
}

/// The single-byte operand is a zero-page base address; register `R` is added and the sum wraps
/// within page zero, so the effective address never escapes `$00`–`$FF`. Effective address:
/// `(nn + R) & $FF`. See [`ZeroPageX`] (`nn, X`) and [`ZeroPageY`] (`nn, Y`).
pub struct ZeroPageIndexed<const R: Register>;

impl<const R: Register> AddressingMode for ZeroPageIndexed<R> {
    const CYCLES: u8 = 3;

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 => ZeroPage::resolve::<O, _>(cpu, bus),

            2 => {
                // Hardware reads the unindexed address while the ALU adds the index; result discarded.
                let _ = bus.read(cpu.address);

                // Wrap the indexed offset within page zero; no carry into the high byte.
                let offset = R.get(cpu);
                cpu.address = u16::from((cpu.address as u8).wrapping_add(offset));

                Poll::Pending
            }

            _ => Poll::Ready(None),
        }
    }
}

pub type ZeroPageX = ZeroPageIndexed<{ X }>;
pub type ZeroPageY = ZeroPageIndexed<{ Y }>;

/// The two-byte little-endian operand provides the effective address directly, reaching the full
/// 64 KiB address space.
/// Syntax: `nnnn`.
pub struct Absolute;

impl AddressingMode for Absolute {
    const CYCLES: u8 = 3;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 => ZeroPage::resolve::<O, _>(cpu, bus),

            2 => {
                // Fetch high byte and merge; low byte was stored in cpu.address on cycle 1.
                cpu.address |= u16::from(cpu.fetch(bus)) << 8;

                // JMP (Implicit) jumps to the resolved address with no separate data bus cycle — done.
                // All other modes (Read, Write, RMW) need one more cycle for the actual memory access.
                if O::ACCESS.is_none() { Poll::Ready(None) } else { Poll::Pending }
            }

            _ => Poll::Ready(None),
        }
    }
}

/// The two-byte little-endian operand is an absolute base address; register `R` is added to produce
/// the effective address. A page crossing adds one extra cycle for read operations; write operations
/// always pay it to avoid acting on the pre-carry address. See [`AbsoluteX`] (`nnnn, X`) and
/// [`AbsoluteY`] (`nnnn, Y`).
pub struct AbsoluteIndexed<const R: Register>;

impl<const R: Register> AddressingMode for AbsoluteIndexed<R> {
    const CYCLES: u8 = 4;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 | 2 => Absolute::resolve::<O, _>(cpu, bus),

            3 => {
                let index = R.get(cpu);
                let address = cpu.address.wrapping_add(u16::from(index));
                let page_crossed = cpu.address >> 8 != address >> 8;

                // Hardware speculatively reads (base_hi, lo + index) before the carry is resolved.
                let speculative = bus.read((cpu.address & 0xFF00) | (address & 0x00FF));

                cpu.address = address;

                // Read ops with no page cross: the speculative read landed on the right address, so
                // its result is the real operand — return it to avoid a redundant bus access.
                // Write ops and page-crossing reads always take an extra cycle to commit the carry.
                if !page_crossed && matches!(O::ACCESS, Some(MemoryAccess::Read)) {
                    Poll::Ready(Some(speculative))
                } else {
                    Poll::Pending
                }
            }

            _ => Poll::Ready(None),
        }
    }
}

pub type AbsoluteX = AbsoluteIndexed<{ X }>;
pub type AbsoluteY = AbsoluteIndexed<{ Y }>;

/// The 16-bit operand is a pointer; the CPU fetches the effective address from that location.
/// Only used by `JMP`. **Hardware bug**: if the pointer is at `$xxFF`, the high byte is
/// read from `$xx00` (wraps within the page) instead of `$xx+1:00`.
/// Syntax: `(nnnn)`.
pub struct Indirect;

impl AddressingMode for Indirect {
    const CYCLES: u8 = 4;

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 => {
                // Fetch low byte of pointer address (same as ZeroPage).
                cpu.address = u16::from(cpu.fetch(bus));
                Poll::Pending
            }

            2 => {
                // Fetch high byte of pointer address. Do NOT delegate to Absolute here because
                // Absolute short-circuits for JMP (ACCESS=None) and returns Poll::Ready, which
                // would skip t=3/4 and jump to the pointer address instead of dereferencing it.
                cpu.address |= u16::from(cpu.fetch(bus)) << 8;
                Poll::Pending
            }

            3 => {
                // Read the low byte of the jump target from the pointer; hold it in data.
                cpu.data = bus.read(cpu.address);

                Poll::Pending
            }

            4 => {
                // Read the high byte from ptr+1, wrapping within the pointer's page (6502 hardware bug).
                // Form the final target address and fall through to the execute block.
                let ptr = (cpu.address & 0xFF00) | u16::from((cpu.address as u8).wrapping_add(1));
                let hi = u16::from(bus.read(ptr)) << 8;

                cpu.address = u16::from(cpu.data) | hi;

                Poll::Ready(None)
            }

            _ => Poll::Ready(None),
        }
    }
}

/// Pre-indexed indirect via zero page: add X to the zero-page operand (wrapping), then fetch
/// the 16-bit effective address from that zero-page location.
/// Effective address: `[$00 | ((nn + X) & $FF)]` (16-bit read).
/// Syntax: `(nn, X)`.
pub struct IndirectX;

impl AddressingMode for IndirectX {
    const CYCLES: u8 = 5;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 => {
                // Fetch the zero-page pointer byte that both indirect modes use as their base.
                cpu.ptr = cpu.fetch(bus);

                Poll::Pending
            }

            2 => {
                // The 6502 performs a dummy read at the un-indexed pointer before adding X.
                // The result wraps within page zero so the effective pointer stays in 0x00–0xFF.
                let _ = bus.read(u16::from(cpu.ptr)); // dummy read

                cpu.ptr = cpu.ptr.wrapping_add(cpu.x);

                Poll::Pending
            }

            3 => {
                // Read the low byte of the target address from the indexed pointer.
                cpu.address = u16::from(bus.read(u16::from(cpu.ptr)));

                Poll::Pending
            }

            4 => {
                // Read the high byte from ptr+1 (wrapping within page zero) to complete the 16-bit address.
                cpu.address |= u16::from(bus.read(u16::from(cpu.ptr.wrapping_add(1)))) << 8;

                Poll::Pending
            }

            _ => Poll::Ready(None),
        }
    }
}

/// Post-indexed indirect via zero page: fetch the 16-bit base address from the zero-page
/// operand, then add Y to produce the effective address.
/// A page crossing from the base address adds one extra cycle.
/// Effective address: `[$00nn] + Y` (16-bit read + Y).
/// Syntax: `(nn), Y`.
pub struct IndirectY;

impl AddressingMode for IndirectY {
    const CYCLES: u8 = 5;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        match cpu.t {
            1 => IndirectX::resolve::<O, _>(cpu, bus),

            2 => {
                // Read the low byte of the target address from the indexed pointer.
                cpu.address = u16::from(bus.read(u16::from(cpu.ptr)));

                Poll::Pending
            }

            3 => {
                // Read the high byte from ptr+1, form the full base address, then add Y.
                // Record whether the addition carried into the high byte so the next cycle can decide
                // whether an extra bus cycle is needed to correct the address.
                let hi = u16::from(bus.read(u16::from(cpu.ptr.wrapping_add(1)))) << 8;
                let address = cpu.address | hi;

                cpu.address = address.wrapping_add(u16::from(cpu.y));
                cpu.data = u8::from(address >> 8 != cpu.address >> 8);

                Poll::Pending
            }

            4 if !matches!(O::ACCESS, Some(MemoryAccess::Read)) || cpu.data != 0 => {
                // The 6502 always reads from the pre-carry address while correcting the high byte.
                // Stores always pay this cycle; loads skip it only when no page was crossed.
                // When no page cross occurred base_hi == effective_hi, so the speculative read lands
                // on the correct page and no subtraction is needed.
                let wrong_page_address = if cpu.data != 0 {
                    (cpu.address & 0x00FF) | ((cpu.address & 0xFF00).wrapping_sub(0x0100))
                } else {
                    cpu.address
                };

                let _ = bus.read(wrong_page_address);

                Poll::Pending
            }

            _ => Poll::Ready(None),
        }
    }
}

/// Signed 8-bit offset added to the program counter *after* the branch instruction is fetched
/// (i.e., relative to PC+2). Used exclusively by branch instructions (`BEQ`, `BNE`, etc.).
/// Range: −128..+127 bytes from the following instruction.
pub struct Relative;

impl AddressingMode for Relative {
    const CYCLES: u8 = 1;

    #[inline]
    fn resolve<O: Operation, B: Bus>(cpu: &mut Cpu2A03<B>, bus: &mut B) -> Poll<Option<u8>> {
        if cpu.t == 1 {
            // Fetch the signed offset; branch condition and PC update are handled by the operation.
            cpu.data = cpu.fetch(bus);
        }

        Poll::Ready(None)
    }
}

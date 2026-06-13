// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use std::marker::ConstParamTy;
use std::task::Poll;

use crate::bus::CpuReadWrite;
use crate::cpu::Cpu2A03;

mod arithmetic;
mod flow;
mod logical;
mod stack;
mod system;
mod transfer;

pub use arithmetic::{ADC, CMP, CPX, CPY, DCP, DEC, DEX, DEY, INC, INX, INY, ISC, SBC, SBX};
pub use flow::{BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS, JMP, JSR, RTI, RTS};
pub use logical::{ALR, ANC, AND, ARR, ASL, BIT, EOR, LSR, ORA, RLA, ROL, ROR, RRA, SLO, SRE};
pub use stack::{PHA, PHP, PLA, PLP};
pub use system::{BRK, CLC, CLD, CLI, CLV, NOP, SEC, SED, SEI};
pub use transfer::{
    LAS, LAX, LDA, LDX, LDY, LXA, SAX, SHA, SHX, SHY, STA, STX, STY, TAS, TAX, TAY, TSX, TXA, TXS,
    TYA, XAA,
};

/// Classifies how an operation accesses memory at the effective address. Drives the
/// addressing-mode pipeline so the right read or write cycles fire before [`Operation::apply`]
/// runs.
///
/// `Read` issues a single bus read into `cpu.data`. `Write` skips the read and lets
/// `apply` perform the bus write itself. `ReadModifyWrite` does the read, then a
/// spurious write of the *original* value back to the same address (a 6502 timing
/// quirk), then `apply` produces the modified byte and the addressing mode finishes
/// with the real write.
#[derive(Debug, Clone, Copy)]
pub enum MemoryAccess {
    /// One bus read into `cpu.data`. Used by load and arithmetic-on-memory ops.
    Read,

    /// No read, `apply` is responsible for emitting the write. Used by store ops.
    Write,

    /// Read, spurious write-back, then `apply` followed by the real write. Used by
    /// `INC`/`DEC`/`ASL`/`LSR`/`ROL`/`ROR` and their undocumented combinations.
    ReadModifyWrite,
}

/// The effect of a single 2A03 instruction mnemonic.
///
/// An `Operation` is the second half of instruction dispatch. The addressing mode runs
/// first, resolving the effective address into `cpu.address` (and, depending on
/// [`ACCESS`][Self::ACCESS], pre-reading the byte into `cpu.data` or doing the spurious
/// read-modify-write dance). Then `apply` runs and produces the actual mnemonic effect
/// against CPU and bus state.
///
/// See <https://www.nesdev.org/obelisk-6502-guide/reference.html> for what each
/// mnemonic is supposed to do, and <https://www.nesdev.org/wiki/CPU_unofficial_opcodes>
/// for the undocumented family.
pub trait Operation {
    /// How this operation accesses memory at the effective address, or `None` if it
    /// touches no memory beyond what the addressing mode already did (`JMP`, register
    /// ops, implied-only ops). Drives the addressing-mode pipeline before `apply`.
    const ACCESS: Option<MemoryAccess> = None;

    /// Runs one cycle of the operation against `cpu` and `bus`.
    ///
    /// Called only after the addressing mode has fully resolved. `cpu.address` holds the
    /// effective address. Returns [`Poll::Pending`] to consume another cycle (multi-step
    /// ops like `JSR`, `RTS`, `RTI`, branches), or [`Poll::Ready`] to retire the
    /// instruction and let the CPU fetch the next opcode.
    fn apply<B: CpuReadWrite>(cpu: &mut Cpu2A03, bus: &mut B) -> Poll<()>
    where
        Self: Sized;
}

/// One of the 2A03's four user-visible registers.
///
/// Used as a const generic parameter so register-flavored variants of the same mnemonic
/// share a single generic implementation: `LDA`/`LDX`/`LDY` collapse into one `LOAD<R>`,
/// `INX`/`INY`/`DEX`/`DEY` reuse the same increment/decrement body, transfer ops route
/// through `TRANSFER<SRC, DST>`, and so on. The compiler monomorphizes each `R` into a
/// dedicated specialization, so there is no runtime cost.
#[derive(Debug, Clone, Copy, ConstParamTy, PartialEq, Eq)]
pub enum Register {
    /// Accumulator. Source and destination for most arithmetic and logical operations.
    A,

    /// Index register `X`. Used for indexed addressing (`zp,X`, `abs,X`) and for the
    /// stack-pointer transfer pair `TSX`/`TXS`.
    X,

    /// Index register `Y`. Used for indexed addressing (`zp,Y`, `abs,Y`, `(zp),Y`).
    Y,

    /// Stack pointer. The low byte of an address in page 1 (`$0100`..`$01FF`).
    SP,
}

impl Register {
    /// Reads the value of this register.
    #[inline(always)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    pub const fn get(self, cpu: &Cpu2A03) -> u8 {
        match self {
            Self::A => cpu.a,
            Self::X => cpu.x,
            Self::Y => cpu.y,
            Self::SP => cpu.sp,
        }
    }

    /// Writes `value` to this register.
    #[inline(always)]
    pub const fn set(self, cpu: &mut Cpu2A03, value: u8) {
        match self {
            Self::A => {
                cpu.a = value;
            }

            Self::X => {
                cpu.x = value;
            }

            Self::Y => {
                cpu.y = value;
            }

            Self::SP => {
                cpu.sp = value;
            }
        }
    }
}

/// Where an operation's data lives, either a CPU register or the effective memory
/// address.
///
/// Used as a const generic parameter so register and memory variants of the same
/// mnemonic share one implementation. `INC`/`INX`/`INY` are all instances of
/// `INCREMENT<O>`, `ASL A` and `ASL $addr` are both `ASL<O>`, and so on. Const-eval
/// turns [`Operand::access`] into a fixed [`MemoryAccess`] for memory variants and
/// `None` for register variants, so the addressing-mode pipeline is configured at
/// compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum Operand {
    /// A CPU register (`A`, `X`, `Y`, or `SP`).
    Register(Register),

    /// The effective address resolved by the addressing mode (read or written through
    /// `cpu.data` / `cpu.address`).
    Memory,
}

impl Operand {
    /// Returns `Some(access)` for memory operands, `None` for register operands.
    /// Suitable for use in `Operation::ACCESS` to drive the addressing-mode pipeline.
    #[inline(always)]
    #[must_use]
    pub const fn access(self, access: MemoryAccess) -> Option<MemoryAccess> {
        match self {
            Self::Register(_) => None,
            Self::Memory => Some(access),
        }
    }

    /// Reads the current value of this operand.
    #[inline(always)]
    #[must_use]
    pub const fn read(self, cpu: &Cpu2A03) -> u8 {
        match self {
            Self::Register(r) => r.get(cpu),
            Self::Memory => cpu.data,
        }
    }

    /// Writes `value` to this operand.
    #[inline(always)]
    pub fn write<B: CpuReadWrite>(self, cpu: &mut Cpu2A03, bus: &mut B, value: u8) {
        match self {
            Self::Register(r) => {
                r.set(cpu, value);
            }

            Self::Memory => {
                bus.write(cpu.address(), value);
            }
        }
    }

    /// Latches `value` into `cpu.data` when this operand is [`Operand::Memory`].
    ///
    /// Used by shift and rotate operations so that composed undocumented ops (SLO, SRE, RLA,
    /// RRA) can read the modified value from `cpu.data` without an extra bus access. For
    /// register variants this is a no-op; `O` is a const generic so LLVM eliminates the branch
    /// at compile time with zero overhead on the register-operand hot paths (ASL A, LSR A, etc.).
    #[inline(always)]
    pub const fn latch(self, cpu: &mut Cpu2A03, value: u8) {
        if matches!(self, Self::Memory) {
            cpu.data = value;
        }
    }
}

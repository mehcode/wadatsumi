// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emulation of the Ricoh 2A03, the CPU found in the NES and Famicom.
//!
//! [`Cpu2A03`] is the chip itself: it owns the architectural registers (A, X, Y, PC, SP),
//! the [`CpuStatus`] flags, and the pipeline state that drives a single instruction across
//! its constituent T-states. The host drives it one clock cycle at a time via
//! [`Cpu2A03::tick`] and reaches WRAM, PPU/APU registers, and cartridge memory through the
//! [`CpuReadWrite`] bus trait (with [`CpuPeek`] for side-effect-free debugger reads and
//! [`CpuBus`] extending the bus with `/NMI`, `/IRQ`, and `RDY` line observation).
//!
//! See <https://www.nesdev.org/wiki/CPU> for the hardware reference.

#![feature(min_adt_const_params)]

mod addressing;
mod bus;
mod cpu;
mod instruction;
mod interrupt;
mod operation;
mod reset;
mod status;
mod table;

pub use bus::{CpuBus, CpuPeek, CpuReadWrite};
pub use cpu::Cpu2A03;
pub use status::CpuStatus;

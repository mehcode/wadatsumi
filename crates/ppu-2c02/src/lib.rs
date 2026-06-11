// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emulation of the Ricoh 2C02, the Picture Processing Unit found in the NES and Famicom.
//!
//! [`Ppu2C02`] is the chip itself: it owns the eight CPU-facing registers (`$2000`..`$2007`),
//! OAM, palette RAM, and the loopy `v`/`t`/`x`/`w` scrolling state. The host drives it one
//! dot at a time via [`Ppu2C02::tick`] and reaches CHR and nametable memory through the
//! [`PpuReadWrite`] bus trait (with [`PpuPeek`] for side-effect-free debugger reads).
//!
//! See <https://www.nesdev.org/wiki/PPU> for the hardware reference.

mod bus;
mod control;
mod mask;
mod ppu;
mod status;

pub use bus::{PpuPeek, PpuReadWrite};
pub use ppu::Ppu2C02;

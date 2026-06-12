// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emulation of the NES, the **Nintendo Entertainment System**.
//!
//! [`SystemNes`] is the console itself: it owns the 2A03 CPU, the 2C02 PPU, 2 KiB of WRAM,
//! 2 KiB of CIRAM (nametable) memory, and the inserted cartridge. The
//! frontend constructs it via [`SystemNes::open`] with an iNES `.nes` ROM and then drives
//! it through the [`System`](wadatsumi_system::System) trait, which interleaves one CPU
//! cycle with three PPU dots per tick to match the NTSC 21.477 MHz master clock.
//!
//! See <https://www.nesdev.org/wiki/NES_reference_guide> for the hardware reference.

mod cpu;
mod error;
mod pak;
mod ppu;
mod system;

pub use error::{Error, Result};
pub use system::SystemNes;

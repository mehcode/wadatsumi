// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod nrom;

pub use nrom::Nrom;

/// Abstracts over cartridge memory-mapping hardware (the physical chips on the PCB).
///
/// Real NES paks varied enormously, some held a flat ROM, others had bank-switching
/// controllers, extra RAM, or even their own audio chips. The mapper trait captures only the
/// part the emulator cares about: translating CPU and PPU addresses into bytes.
///
/// PRG and CHR data are owned by [`Pak`][crate::pak::Pak] and passed in on each call
/// so that the mapper itself is pure logic with no data duplication.
pub trait Mapper {
    /// Read one byte from PRG-ROM/RAM at the given CPU address.
    fn read_prg(&mut self, prg: &[u8], address: u16) -> u8;
}

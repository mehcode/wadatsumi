// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use enum_dispatch::enum_dispatch;

mod nrom;

pub use nrom::NROM;

/// Abstracts over cartridge memory-mapping hardware (the physical chips on the PCB).
///
/// Real NES paks varied enormously, some held a flat ROM, others had bank-switching
/// controllers, extra RAM, or even their own audio chips. The mapper trait captures only the
/// part the emulator cares about: translating CPU and PPU addresses into bytes.
///
/// PRG and CHR data are owned by [`Pak`][crate::pak::Pak] and passed in on each call
/// so that the mapper itself is pure logic with no data duplication.
#[enum_dispatch]
pub trait Mapper {
    /// Read one byte from PRG-ROM/RAM at the given CPU address.
    fn read_prg(&self, prg: &[u8], address: u16) -> u8;

    /// How many bytes of SRAM this cartridge board provides; `Pak` allocates this on open.
    fn sram_size(&self) -> usize;

    /// Read one byte from SRAM at the given CPU address (`$6000–$7FFF`).
    fn read_sram(&self, sram: &[u8], address: u16) -> u8;

    /// Write one byte to SRAM at the given CPU address (`$6000–$7FFF`).
    fn write_sram(&self, sram: &mut [u8], address: u16, value: u8);
}

#[enum_dispatch(Mapper)]
pub enum AnyMapper {
    NROM,
}

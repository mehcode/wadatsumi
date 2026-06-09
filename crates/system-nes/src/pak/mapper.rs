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
    /// Read one byte from PRG-ROM/RAM at the given CPU address without advancing mapper state.
    ///
    /// Side-effect-free; used by the debugger and disassembler.
    fn peek_prg(&self, prg: &[u8], address: u16) -> u8;

    /// Read one byte from PRG-ROM/RAM at the given CPU address.
    ///
    /// Takes `&mut self` because some mappers (e.g. MMC2) latch their CHR bank on certain PRG
    /// reads; defaults to `peek_prg` for mappers with no read side-effects.
    #[inline]
    fn read_prg(&mut self, prg: &[u8], address: u16) -> u8 {
        self.peek_prg(prg, address)
    }

    /// How many bytes of SRAM this cartridge board provides; `Pak` allocates this slice on open.
    ///
    /// Returns `0` for boards with no battery-backed RAM, in which case `read_sram` and
    /// `write_sram` are never called.
    fn sram_size(&self) -> usize;

    /// Read one byte from SRAM at the given CPU address (`$6000–$7FFF`).
    ///
    /// Only called when `sram_size() > 0`; the mapper may assume the slice length equals
    /// `sram_size()`.
    fn read_sram(&self, sram: &[u8], address: u16) -> u8;

    /// Write one byte to SRAM at the given CPU address (`$6000–$7FFF`).
    ///
    /// Only called when `sram_size() > 0`; the mapper may assume the slice length equals
    /// `sram_size()`.
    fn write_sram(&mut self, sram: &mut [u8], address: u16, value: u8);

    /// Read one byte from CHR-ROM/RAM at the given PPU address (`$0000–$1FFF`) without
    /// advancing mapper state.
    ///
    /// Side-effect-free; used by the debugger and disassembler.
    fn peek_chr(&self, chr: &[u8], address: u16) -> u8;

    /// Read one byte from CHR-ROM/RAM at the given PPU address (`$0000–$1FFF`).
    ///
    /// Takes `&mut self` because some mappers (e.g. MMC2/MMC4) update their CHR bank latch on
    /// pattern fetches; defaults to `peek_chr` for mappers without read side-effects.
    #[inline]
    fn read_chr(&mut self, chr: &[u8], address: u16) -> u8 {
        self.peek_chr(chr, address)
    }
}

#[enum_dispatch(Mapper)]
pub enum AnyMapper {
    NROM,
}

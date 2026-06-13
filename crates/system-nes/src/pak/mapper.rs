// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::upper_case_acronyms)]

use enum_dispatch::enum_dispatch;

use crate::pak::mirroring::{Mirroring, NametableAddress};

mod nrom;

pub use nrom::NROM;

/// Abstracts over pak memory-mapping hardware (the physical chips on the PCB).
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

    /// How many bytes of SRAM this pak board provides; `Pak` allocates this slice on open.
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

    /// Current level of the mapper's `/IRQ` output line: `true` while the mapper is
    /// asking the CPU for an IRQ, `false` otherwise. Modeled active-high.
    ///
    /// Boards without an IRQ source (`NROM`, `UxROM`, `CNROM`, `AxROM`, and so on) inherit the
    /// `false` default. Mappers with a scanline counter (MMC3, MMC5, FME-7) or a CPU-clock
    /// timer (VRC4/6/7, FME-7) override this and hold the line high until the game writes
    /// their IRQ-ack register. The system bus wired-ORs this with the APU's IRQ sources
    /// before handing the combined level to the CPU.
    ///
    #[inline]
    fn irq(&self) -> bool {
        false
    }

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

    /// The pak's current nametable mirroring arrangement.
    ///
    /// Many mappers (MMC1, MMC3, FME-7) update this from internal registers during
    /// gameplay; the value returned here is the *current* arrangement, not necessarily
    /// the value parsed from the iNES header.
    ///
    /// The default [`nt_ram_size`], [`nametable_peek`], and [`nametable_write`] impls
    /// use this to route accesses; exotic mappers whose nametable routing does not fit
    /// the four-way [`Mirroring`] enum (notably MMC5 fill mode and ExRAM-as-nametable)
    /// override the routing methods directly.
    fn mirroring(&self) -> Mirroring;

    /// How many bytes of pak-side nametable RAM this board provides; `Pak` allocates this
    /// slice on open.
    ///
    /// Defaults to [`Mirroring::default_nt_ram_size`] for the current [`mirroring`]: 2 KB
    /// for [`Mirroring::FourScreen`] and 0 otherwise. Override for boards that need
    /// pak-side nametable RAM independent of mirroring (e.g. MMC5 `ExRAM` provides 1 KB
    /// regardless).
    #[inline]
    fn nt_ram_size(&self) -> usize {
        self.mirroring().default_nt_ram_size()
    }

    /// Read one byte from the nametable space (`$2000–$3EFF`) without advancing mapper state.
    ///
    /// `pak_nt_ram` is the pak's own nametable RAM (length [`nt_ram_size`]); `ciram` is the
    /// console's internal 2 KB nametable RAM. The default body delegates routing to
    /// [`Mirroring::nametable_address`]; override for exotic mappers whose nametable routing
    /// does not fit the four-way [`Mirroring`] enum (notably MMC5 fill mode and `ExRAM`).
    ///
    /// Side-effect-free; used by the debugger and the PPU's peek path.
    #[inline]
    fn nametable_peek(&self, pak_nt_ram: &[u8], ciram: &[u8; 2048], address: u16) -> u8 {
        match self.mirroring().nametable_address(address) {
            NametableAddress::Ciram(address) => ciram[address],
            NametableAddress::PakNtRam(address) => pak_nt_ram[address],
        }
    }

    /// Read one byte from the nametable space (`$2000–$3EFF`).
    ///
    /// Takes `&mut self` because some mappers may latch on nametable fetches (notably MMC5
    /// in `ExRAM`/EXNT mode); defaults to [`nametable_peek`] for the common case.
    #[inline]
    fn nametable_read(&mut self, pak_nt_ram: &[u8], ciram: &[u8; 2048], address: u16) -> u8 {
        self.nametable_peek(pak_nt_ram, ciram, address)
    }

    /// Write one byte into the nametable space (`$2000–$3EFF`).
    ///
    /// The default body delegates routing to [`Mirroring::nametable_address`], symmetric
    /// with [`nametable_peek`]; override alongside `nametable_peek` for exotic mappers.
    #[inline]
    fn nametable_write(
        &mut self,
        pak_nt_ram: &mut [u8],
        ciram: &mut [u8; 2048],
        address: u16,
        value: u8,
    ) {
        match self.mirroring().nametable_address(address) {
            NametableAddress::Ciram(address) => {
                ciram[address] = value;
            }

            NametableAddress::PakNtRam(address) => {
                pak_nt_ram[address] = value;
            }
        }
    }
}

#[enum_dispatch(Mapper)]
pub enum AnyMapper {
    NROM,
}

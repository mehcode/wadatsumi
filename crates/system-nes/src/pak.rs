// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use bytes::Bytes;

use crate::error::Error;
use crate::pak::mapper::{AnyMapper, Mapper};
use crate::pak::mirroring::Mirroring;

mod mapper;
mod mirroring;

// iNES header is always exactly 16 bytes, followed by an optional trainer, then PRG, then CHR.
const HEADER_SIZE: usize = 16;

// A 512-byte block some ROM dumpers injected before the PRG data to patch games (cheats,
// infinite lives, etc.). It's a dumping artifact, real paks never had trainers.
// Modern emulators skip these bytes entirely.
const TRAINER_SIZE: usize = 512;

// PRG-ROM comes in 16 KB banks; the header stores the count, not the total size.
const PRG_BANK_SIZE: usize = 16 * 1024;

// CHR-ROM comes in 8 KB banks. A count of zero means the pak uses CHR-RAM instead —
// the mapper supplies 8 KB of writable RAM in place of read-only ROM.
const CHR_BANK_SIZE: usize = 8 * 1024;

/// A loaded NES game pak.
///
/// Holds the PRG-ROM (CPU-side program data), CHR-ROM (PPU-side pattern data),
/// the mapper that handles address translation, and any pak-side RAM the board
/// provides (SRAM at `$6000–$7FFF` on the CPU bus, and nametable RAM on the PPU bus).
pub struct Pak {
    prg: Bytes,
    chr: Bytes,
    sram: Option<Box<[u8]>>,

    /// Pak-side nametable RAM, on the PPU bus. Distinct from the console's
    /// 2 KB CIRAM (which lives in [`SystemNes`][crate::SystemNes]); allocated
    /// only when the mapper reports a non-zero [`Mapper::nt_ram_size`].
    ///
    /// Used by 4-screen boards (Gauntlet, Rad Racer II) to back the upper two
    /// logical nametables that CIRAM cannot hold; future boards may use this
    /// slot for MMC5 ExRAM-as-nametable and similar.
    nt_ram: Option<Box<[u8]>>,

    mapper: AnyMapper,
}

impl Pak {
    /// Open and parse an iNES `.nes` ROM file.
    ///
    /// Reads the 16-byte iNES header to identify the mapper and locate the PRG
    /// and CHR data within the file, then constructs the appropriate mapper.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPak`] if the file is too short or the magic bytes
    /// are wrong, and [`Error::UnsupportedMapper`] if the mapper number is not
    /// implemented.
    ///
    pub fn open(pak: Vec<u8>) -> crate::Result<Self> {
        let pak = Bytes::from(pak);

        // \x1A is the MS-DOS EOF marker, included so the file won't be misread as plain text.
        if pak.len() < HEADER_SIZE || &pak[0..4] != b"NES\x1A" {
            return Err(Error::InvalidPak);
        }

        let prg_banks = pak[4] as usize;
        let chr_banks = pak[5] as usize;

        if prg_banks == 0 {
            return Err(Error::InvalidPak);
        }

        // Flags 6 and 7 pack several fields into nibbles and individual bits.
        // The naming (flags6/flags7) matches the iNES specification directly.
        let flags6 = pak[6];
        let flags7 = pak[7];

        // The mapper number is split across both flag bytes: the upper nibble comes from
        // flags7 and the lower nibble from the upper bits of flags6.
        let mapper_num = (flags7 & 0xF0) | (flags6 >> 4);

        let has_trainer = flags6 & 0x04 != 0;

        // Bit 3 signals four-screen mode and overrides the standard mirroring bit (bit 0).
        let mirroring = if flags6 & 0x08 != 0 {
            Mirroring::FourScreen
        } else if flags6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let data_start = HEADER_SIZE + if has_trainer { TRAINER_SIZE } else { 0 };
        let prg_end = data_start + prg_banks * PRG_BANK_SIZE;
        let chr_end = prg_end + chr_banks * CHR_BANK_SIZE;

        if chr_end > pak.len() {
            return Err(Error::InvalidPak);
        }

        // slice() returns a reference into the same allocation.
        let prg = pak.slice(data_start..prg_end);

        // Fixed-window mappers (e.g. NROM) mirror PRG by masking the address with
        // `len - 1`; that only yields a correctly-mirrored index when the size is a
        // power of two. Every legal PRG-ROM is a power-of-two number of 16 KiB banks,
        // so reject anything else here, this is also the invariant the mappers' unchecked
        // indexing relies on for soundness.
        if !prg.len().is_power_of_two() {
            return Err(Error::InvalidPak);
        }

        // chr is an empty slice when chr_banks == 0; the mapper is responsible for
        // providing CHR-RAM in that case.
        let chr = pak.slice(prg_end..chr_end);

        let mapper = match mapper_num {
            0 => AnyMapper::from(mapper::NROM::new(mirroring)),

            _ => return Err(Error::UnsupportedMapper(mapper_num)),
        };

        let sram = (mapper.sram_size() > 0).then(|| vec![0; mapper.sram_size()].into_boxed_slice());

        let nt_ram =
            (mapper.nt_ram_size() > 0).then(|| vec![0; mapper.nt_ram_size()].into_boxed_slice());

        Ok(Self { prg, chr, sram, nt_ram, mapper })
    }

    /// Read one byte from PRG-ROM at the given CPU bus address, without advancing mapper state.
    ///
    /// Side-effect-free; used by the debugger and disassembler. The valid range is
    /// mapper-dependent but is typically $8000-$FFFF.
    #[inline]
    pub fn peek_prg(&self, address: u16) -> u8 {
        self.mapper.peek_prg(&self.prg, address)
    }

    /// Read one byte from PRG-ROM at the given CPU bus address.
    ///
    /// Delegates to the mapper, which translates the address according to the
    /// pak's bank-switching state and may update internal latch state.
    /// The valid range is mapper-dependent but is typically $8000-$FFFF.
    #[inline]
    pub fn read_prg(&mut self, address: u16) -> u8 {
        self.mapper.read_prg(&self.prg, address)
    }

    /// Read one byte from CHR-ROM at the given PPU address (`$0000–$1FFF`), without
    /// advancing mapper state. Used by the PPU's peek path and the debugger.
    #[inline]
    pub fn peek_chr(&self, address: u16) -> u8 {
        self.mapper.peek_chr(&self.chr, address)
    }

    /// Read one byte from CHR-ROM at the given PPU address (`$0000–$1FFF`).
    ///
    /// May update mapper latch state (e.g. MMC2/MMC4 bank switching on pattern fetches).
    #[inline]
    pub fn read_chr(&mut self, address: u16) -> u8 {
        self.mapper.read_chr(&self.chr, address)
    }

    /// Read one byte from SRAM at the given CPU bus address (`$6000–$7FFF`).
    ///
    /// Returns `0` if this pak has no SRAM.
    #[inline]
    pub fn read_sram(&self, address: u16) -> u8 {
        self.sram.as_deref().map_or(0, |sram| self.mapper.read_sram(sram, address))
    }

    /// Write one byte to SRAM at the given CPU bus address (`$6000–$7FFF`).
    ///
    /// Does nothing if this pak has no SRAM.
    #[inline]
    pub fn write_sram(&mut self, address: u16, value: u8) {
        if let Some(sram) = self.sram.as_deref_mut() {
            self.mapper.write_sram(sram, address, value);
        }
    }

    /// Read one byte from the nametable space (`$2000–$3EFF`), without advancing mapper
    /// state. `ciram` is the console's internal 2 KB nametable RAM.
    ///
    /// The mapper applies its current mirroring arrangement to route the access to either
    /// CIRAM or the pak's own nametable RAM (for four-screen boards).
    #[inline]
    pub fn nametable_peek(&self, ciram: &[u8; 2048], address: u16) -> u8 {
        self.mapper.nametable_peek(self.nt_ram.as_deref().unwrap_or(&[]), ciram, address)
    }

    /// Read one byte from the nametable space (`$2000–$3EFF`).
    ///
    /// May update mapper latch state on future mappers (e.g. MMC5 EXNT).
    #[inline]
    pub fn nametable_read(&mut self, ciram: &[u8; 2048], address: u16) -> u8 {
        self.mapper.nametable_read(self.nt_ram.as_deref().unwrap_or(&[]), ciram, address)
    }

    /// Current level of the pak's `/IRQ` output line: `true` while the mapper is asking
    /// the CPU for an IRQ. Boards without an IRQ source always return `false`.
    ///
    /// The system bus wired-ORs this with the APU's IRQ sources before handing the
    /// combined level to the CPU through [`CpuBus::irq`](wadatsumi_cpu_2a03::CpuBus::irq).
    ///
    #[inline]
    #[must_use]
    pub fn irq(&self) -> bool {
        self.mapper.irq()
    }

    /// Write one byte into the nametable space (`$2000–$3EFF`).
    #[inline]
    pub fn nametable_write(&mut self, ciram: &mut [u8; 2048], address: u16, value: u8) {
        self.mapper.nametable_write(
            self.nt_ram.as_deref_mut().unwrap_or(&mut []),
            ciram,
            address,
            value,
        );
    }
}

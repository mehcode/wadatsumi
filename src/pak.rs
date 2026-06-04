// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

use bytes::Bytes;

use crate::error::Error;
use crate::pak::mapper::{AnyMapper, Mapper};

mod mapper;

// iNES header is always exactly 16 bytes, followed by an optional trainer, then PRG, then CHR.
const HEADER_SIZE: usize = 16;

// A 512-byte block some ROM dumpers injected before the PRG data to patch games (cheats,
// infinite lives, etc.). It's a dumping artifact, real cartridges never had trainers.
// Modern emulators skip these bytes entirely.
const TRAINER_SIZE: usize = 512;

// PRG-ROM comes in 16 KB banks; the header stores the count, not the total size.
const PRG_BANK_SIZE: usize = 16 * 1024;

// CHR-ROM comes in 8 KB banks. A count of zero means the cartridge uses CHR-RAM instead —
// the mapper supplies 8 KB of writable RAM in place of read-only ROM.
const CHR_BANK_SIZE: usize = 8 * 1024;

/// How the cartridge wires its nametables to the PPU's VRAM.
///
/// The NES PPU has 2 KB of VRAM, enough for two nametables, but the screen is logically
/// divided into four. Mirroring determines which pairs of nametables share memory, which
/// in turn controls how the background scrolls at the edges.
pub enum Mirroring {
    /// Nametables A/C share memory; B/D share memory. Scrolling wraps left-right.
    Horizontal,

    /// Nametables A/B share memory; C/D share memory. Scrolling wraps top-bottom.
    Vertical,

    /// All four nametables map to distinct memory (requires extra VRAM on the cartridge).
    FourScreen,
}

/// A loaded NES game pak.
///
/// Holds the PRG-ROM (CPU-side program data), CHR-ROM (PPU-side pattern data),
/// the mapper that handles address translation, and cartridge-level metadata
/// the rest of the system needs to configure itself (e.g. nametable mirroring).
pub struct Pak {
    prg: Bytes,
    chr: Bytes,
    sram: Option<Box<[u8]>>,
    mapper: AnyMapper,

    /// Nametable mirroring arrangement, set by the cartridge hardware.
    /// The PPU reads this to determine how the four logical nametables
    /// map onto its 2 KB of VRAM.
    pub mirroring: Mirroring,
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
    pub fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let pak = fs::read(path)?;
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
        // so reject anything else here — this is also the invariant the mappers' unchecked
        // indexing relies on for soundness.
        if !prg.len().is_power_of_two() {
            return Err(Error::InvalidPak);
        }

        // chr is an empty slice when chr_banks == 0; the mapper is responsible for
        // providing CHR-RAM in that case.
        let chr = pak.slice(prg_end..chr_end);

        let mapper = match mapper_num {
            0 => AnyMapper::from(mapper::NROM),

            _ => return Err(Error::UnsupportedMapper(mapper_num)),
        };

        let sram = if mapper.sram_size() > 0 {
            Some(vec![0; mapper.sram_size()].into_boxed_slice())
        } else {
            None
        };

        Ok(Self { prg, chr, sram, mapper, mirroring })
    }

    /// Read one byte from PRG-ROM at the given CPU bus address.
    ///
    /// Delegates to the mapper, which translates the address according to the
    /// cartridge's bank-switching state. The valid range is mapper-dependent
    /// but is typically $8000-$FFFF.
    #[inline]
    pub fn read_prg(&self, address: u16) -> u8 {
        self.mapper.read_prg(&self.prg, address)
    }

    /// Read one byte from SRAM at the given CPU bus address (`$6000–$7FFF`).
    ///
    /// Returns `0` if this cartridge has no SRAM.
    #[inline]
    pub fn read_sram(&self, address: u16) -> u8 {
        self.sram.as_deref().map_or(0, |sram| self.mapper.read_sram(sram, address))
    }

    /// Write one byte to SRAM at the given CPU bus address (`$6000–$7FFF`).
    ///
    /// Does nothing if this cartridge has no SRAM.
    #[inline]
    pub fn write_sram(&mut self, address: u16, value: u8) {
        if let Some(sram) = self.sram.as_deref_mut() {
            self.mapper.write_sram(sram, address, value);
        }
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::pak::mapper::Mapper;

/// Mapper 0 (NROM). The simplest NES cartridge board, used by early titles like
/// Donkey Kong, Super Mario Bros., and Excitebike. The board has no bank-switching
/// hardware; the CPU sees the PRG-ROM directly at $8000-$FFFF. NROM-128 boards carry
/// 16 KB of PRG-ROM mirrored across the full window; NROM-256 boards carry 32 KB.
///
/// NROM has no read side-effects on either PRG or CHR, so only the side-effect-free
/// `peek_*` variants are implemented here; `read_prg` and `read_chr` use the trait
/// defaults, which delegate to `peek_prg` and `peek_chr` respectively.
pub struct NROM;

impl Mapper for NROM {
    #[inline]
    fn peek_prg(&self, prg: &[u8], address: u16) -> u8 {
        // PRG-ROM occupies $8000-$FFFF in the CPU address space.
        //
        // NROM-128 has 16 KB of PRG-ROM, mirrored across the full 32 KB window;
        // NROM-256 fills the window exactly. Masking with `len - 1` folds both cases:
        // it mirrors a 16 KB ROM into the window and is a no-op for a 32 KB ROM. The
        // mirroring is only correct when `len` is a power of two, which `Pak::open`
        // guarantees.
        //
        // SAFETY: `address & (len - 1)` can only clear bits, so the result is always
        // `<= len - 1`, hence a valid index whenever `len > 0`. `Pak::open` rejects a
        // zero-bank (empty) PRG, so `len >= 16 KiB > 0` holds here.
        unsafe { *prg.get_unchecked((address as usize) & (prg.len() - 1)) }
    }

    #[inline]
    fn sram_size(&self) -> usize {
        8 * 1024 // 8 KB
    }

    #[inline]
    fn read_sram(&self, sram: &[u8], address: u16) -> u8 {
        // SAFETY: the mask `& 0x1fff` yields an index in `0..=0x1fff` (0..8 KiB). `Pak`
        // allocates SRAM to `sram_size()` (8 KiB) and only calls this when SRAM exists,
        // so the slice is exactly 8 KiB and the index is always in bounds.
        unsafe { *sram.get_unchecked((address as usize) & 0x1fff) }
    }

    #[inline]
    fn write_sram(&mut self, sram: &mut [u8], address: u16, value: u8) {
        // SAFETY: identical invariant to `read_sram`, `& 0x1fff` indexes within the
        // 8 KiB SRAM slice that `Pak` allocates from `sram_size()`.
        unsafe {
            *sram.get_unchecked_mut((address as usize) & 0x1fff) = value;
        }
    }

    #[inline]
    fn peek_chr(&self, chr: &[u8], address: u16) -> u8 {
        // CHR window is $0000–$1FFF (8 KiB). Masking with `len - 1` handles the standard
        // 8 KiB CHR-ROM case (no-op) and any power-of-two bank size. `Pak::open` guarantees
        // CHR-ROM length is a non-zero power of two when this is called.
        //
        // SAFETY: `address & (len - 1)` is always `<= len - 1`, a valid index for `len > 0`.
        unsafe { *chr.get_unchecked((address as usize) & (chr.len() - 1)) }
    }
}

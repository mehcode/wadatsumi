// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::pak::mapper::Mapper;

/// Mapper 0 (NROM). The simplest NES cartridge board, used by early titles like
/// Donkey Kong, Super Mario Bros., and Excitebike. The board has no bank-switching
/// hardware; the CPU sees the PRG-ROM directly at $8000-$FFFF. NROM-128 boards carry
/// 16 KB of PRG-ROM mirrored across the full window; NROM-256 boards carry 32 KB.
pub struct Nrom;

impl Mapper for Nrom {
    fn read_prg(&self, prg: &[u8], address: u16) -> u8 {
        // PRG-ROM occupies $8000-$FFFF in the CPU address space.
        // Masking off bit 15 converts the CPU address to an offset within that 32 KB window.
        let offset = (address as usize) & 0x7fff;

        // NROM-128 has 16 KB of PRG-ROM, mirrored across the full 32 KB window.
        // NROM-256 fills the window exactly. The modulo handles both without branching,
        // it mirrors a 16 KB slice and is a no-op for 32 KB.
        prg[offset % prg.len()]
    }

    fn sram_size(&self) -> usize {
        8 * 1024 // 8 KB
    }

    fn read_sram(&self, sram: &[u8], address: u16) -> u8 {
        sram[(address as usize) & 0x1fff]
    }

    fn write_sram(&self, sram: &mut [u8], address: u16, value: u8) {
        sram[(address as usize) & 0x1fff] = value;
    }
}

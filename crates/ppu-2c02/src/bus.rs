// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Side-effect-free peek at the PPU address bus. Use this from debuggers or memory viewers
/// that want to look at VRAM without disturbing the read-ahead buffer or any other latched state.
pub trait PpuPeek {
    /// Returns the byte the PPU bus would read at `address`, without mutating any bus
    /// state. Mappers should mirror what [`PpuReadWrite::ppu_read`] would return but skip
    /// any side effects (such as bumping an MMC3 IRQ counter).
    fn ppu_peek(&self, address: u16) -> u8;
}

/// Read/write access to the PPU address bus. The PPU sees CHR ROM/RAM, the nametables,
/// and palette space through this interface; the pak (and mirroring config) decide
/// where each address actually lands.
pub trait PpuReadWrite {
    /// Reads one byte from the PPU bus at `address`. May have side effects on the
    /// pak (for example, mappers that watch A12 transitions for IRQ timing).
    fn ppu_read(&mut self, address: u16) -> u8;

    /// Writes one byte to the PPU bus at `address`. Writes into CHR ROM are silently
    /// ignored by most mappers; writes into nametable space land in the console's VRAM
    /// after the mapper applies mirroring.
    fn ppu_write(&mut self, address: u16, value: u8);
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU control register (`$2000`), written by the CPU to configure major rendering parameters.
/// Stored as a raw byte because every bit pattern is valid and the byte is needed as-is for open-bus behavior.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PpuControl(pub u8);

impl PpuControl {
    /// Selects the base nametable (0 – 3), mapping to VRAM addresses `$2000`, `$2400`, `$2800`, and `$2C00`.
    /// This determines where in the tilemap the PPU begins rendering the background.
    pub const fn nametable(self) -> u16 {
        (self.0 & 0b0000_0011) as u16
    }

    /// When true, the PPU fires an NMI at the start of vertical blank.
    /// This is the primary mechanism for the CPU to perform work during the non-rendering period.
    pub const fn nmi_enabled(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// The amount added to the VRAM address (`v`) after each CPU read or write of PPUDATA.
    /// Returns 1 (across a row) or 32 (down a column).
    pub const fn vram_increment(self) -> u16 {
        if self.0 & 0b0000_0100 != 0 { 32 } else { 1 }
    }

    /// Base CHR address for fetching 8×8 sprite tiles, either `$0000` or `$1000`.
    /// Ignored in 8×16 sprite mode, where the low bit of each tile index selects the pattern table instead.
    pub const fn sprite_pattern_table(self) -> u16 {
        if self.0 & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 }
    }

    /// Base CHR address for fetching background tiles, either `$0000` or `$1000`.
    /// Combined with the tile index from the nametable to form the full CHR fetch address.
    pub const fn background_pattern_table(self) -> u16 {
        if self.0 & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 }
    }

    /// Height of each sprite in pixels: 8 for 8×8 mode, 16 for 8×16 mode.
    /// In 8×16 mode the sprite tile index encodes both the pattern table and the top tile directly.
    pub const fn sprite_height(self) -> u16 {
        if self.0 & 0b0010_0000 != 0 { 16 } else { 8 }
    }

    /// When true, the PPU drives the EXT pins with the current background color rather than reading from them.
    /// This is a rarely-used feature for daisy-chaining an external video signal.
    pub const fn ext_output(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
}

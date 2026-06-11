// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU status register (`$2002`). The CPU reads this to detect vblank, sprite-0 hit,
/// and sprite overflow. Only bits 7..5 are driven by the PPU; bits 4..0 float and come
/// from `io_latch` on read (see [`Ppu2C02::cpu_read`](crate::Ppu2C02::cpu_read)).
///
/// See <https://www.nesdev.org/wiki/PPU_registers#PPUSTATUS> for the canonical reference.
///
/// ```text
/// 7  bit  0
/// ---- ----
/// VSOx xxxx
/// ||||
/// |||+----- PPU open bus; return io_latch
/// |||
/// ||+------ Sprite overflow
/// ||
/// |+------- Sprite 0 hit
/// |
/// +-------- Vertical blank flag, cleared on read
/// ```
///
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PpuStatus(pub u8);

impl PpuStatus {
    /// Sprite overflow flag (bit 5).
    ///
    /// In theory the hardware sets this when more than eight sprites land on the same
    /// scanline during OAM evaluation, so games can scale back sprite count to avoid
    /// flicker. In practice **the PPU has a well-known hardware bug**: after finding the
    /// eighth sprite it keeps evaluating with a corrupted row/column counter, which
    /// produces both false positives (flag set with <= 8 sprites) and false negatives
    /// (flag stays clear with > 8 sprites) depending on sprite positions and OAM layout.
    ///
    /// Cleared at dot 1 of the pre-render scanline (261) along with the other status flags.
    ///
    pub const fn sprite_overflow(self) -> bool {
        self.0 & 0b0010_0000 != 0
    }

    /// Sprite-0 hit flag (bit 6).
    ///
    /// Set on the dot where a non-transparent pixel of sprite 0 (the first OAM entry)
    /// overlaps a non-transparent background pixel, as long as both layers are enabled.
    /// Games poll this to split the screen at a precise scanline for status bars or HUDs,
    /// then rewrite the scroll registers the moment they see the hit.
    ///
    /// A couple of hardware quirks worth knowing:
    /// - **Never set on dot 255** (`x = 255`), even with overlap. Hard-baked into the chip.
    /// - Suppressed when either layer is off, and suppressed in the leftmost 8 pixels if
    ///   the corresponding clip bit in `PPUMASK` is clear.
    ///
    /// Cleared at dot 1 of the pre-render scanline (261) along with the other status flags.
    ///
    pub const fn sprite_0_hit(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    /// Vertical blank flag (bit 7).
    ///
    /// The PPU sets this at dot 1 of scanline 241 (the first scanline of vblank) and holds
    /// it until dot 1 of the pre-render scanline (261), where it gets cleared along with
    /// the other status flags. If `PPUCTRL` bit 7 is set, the PPU also asserts the CPU's
    /// NMI line at the moment it sets this bit.
    /// See <https://www.nesdev.org/wiki/PPU_frame_timing> for the full picture.
    ///
    /// Reading `$2002` captures the flag and clears it as a side effect, so software sees
    /// it at most once per vblank. Games usually read `$2002` at the top of their NMI
    /// handler both to detect the entry and to reset the `w` write-latch before any
    /// `PPUSCROLL`/`PPUADDR` writes.
    ///
    /// **Race condition**: reading `$2002` on the exact cycle the flag is being set
    /// (dot 1, scanline 241) returns `0` *and* suppresses the NMI for that frame.
    ///
    pub const fn vblank(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// Sets or clears the vblank flag (bit 7).
    ///
    /// Driven by the tick engine: set at scanline 241 dot 1, cleared at scanline 261 dot 1.
    ///
    pub(crate) const fn set_vblank(&mut self, vblank: bool) {
        self.0 = (self.0 & 0b0111_1111) | ((vblank as u8) << 7);
    }

    /// Sets or clears the sprite-0 hit flag (bit 6).
    ///
    /// Set by the pixel compositor when a non-transparent sprite-0 pixel overlaps a
    /// non-transparent background pixel (subject to clipping and the dot-255 exception
    /// noted on [`PpuStatus::sprite_0_hit`]). Cleared at scanline 261 dot 1.
    ///
    pub(crate) const fn set_sprite_0_hit(&mut self, hit: bool) {
        self.0 = (self.0 & 0b1011_1111) | ((hit as u8) << 6);
    }

    /// Sets or clears the sprite overflow flag (bit 5).
    ///
    /// Set when more than eight sprites are found on a scanline during OAM evaluation
    /// (subject to the hardware bug noted on [`PpuStatus::sprite_overflow`]). Cleared at
    /// scanline 261 dot 1.
    ///
    pub(crate) const fn set_sprite_overflow(&mut self, overflow: bool) {
        self.0 = (self.0 & 0b1101_1111) | ((overflow as u8) << 5);
    }
}

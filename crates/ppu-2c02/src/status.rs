// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU status register (`$2002`), readable by the CPU to detect rendering events.
/// Stored as a raw byte; only bits [7:5] are driven by the PPU, bits [4:0] float and
/// return `io_latch` on read (see `cpu_read`).
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
    /// Returns whether the sprite overflow flag is set (bit 5).
    ///
    /// The hardware sets this flag when more than eight sprites appear on any one scanline during
    /// OAM evaluation. In theory it lets games detect scanline overflow and reduce sprite count
    /// to avoid flicker, but **the NES PPU has a well-documented hardware bug**: it evaluates
    /// subsequent sprites with a corrupted row/column counter after finding the eighth sprite,
    /// causing both false positives (flag set with ≤ 8 sprites) and false negatives (flag stays
    /// clear with > 8 sprites) depending on sprite positions and OAM layout.
    ///
    /// The flag is cleared at dot 1 of pre-render scanline 261 along with the other status flags.
    ///
    pub const fn sprite_overflow(self) -> bool {
        self.0 & 0b0010_0000 != 0
    }

    /// Returns whether the sprite-0 hit flag is set (bit 6).
    ///
    /// The PPU sets this flag on the dot where a non-transparent pixel of sprite 0 (the first
    /// entry in OAM) overlaps a non-transparent background pixel, provided both background and
    /// sprite rendering are enabled. Games poll this flag to split the screen at a precise
    /// scanline for status bars and HUD elements, the split happens by writing to the scroll
    /// registers immediately after detecting the hit.
    ///
    /// **The flag is never set on dot 255** (x = 255) of any scanline, regardless of overlap;
    /// this is a hardware quirk. It is also suppressed when either layer is disabled or when the
    /// hit would fall in the leftmost 8 pixels if the corresponding clipping bit in `PPUMASK`
    /// is clear.
    ///
    /// The flag is cleared at dot 1 of pre-render scanline 261 along with the other status flags.
    ///
    pub const fn sprite_0_hit(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    /// Returns whether the vertical blank flag is set (bit 7).
    ///
    /// The PPU asserts this flag at dot 1 of scanline 241 (the first scanline of vblank) and
    /// holds it until dot 1 of the pre-render scanline (261), where it is cleared along with
    /// the other status flags. If NMI is enabled in `PPUCTRL` bit 7, the PPU also drives the
    /// CPU's NMI line low at the same moment it sets this flag.
    ///
    /// Reading `$2002` captures the flag and then clears it immediately (the clear takes effect
    /// on the *next* read), ensuring software sees it at most once per vblank interval. Games
    /// read this register at the top of their vblank handler both to detect vblank entry and to
    /// reset the `w` write-latch before beginning `PPUSCROLL`/`PPUADDR` sequences.
    ///
    /// **Race condition**: if the CPU reads `$2002` on the exact cycle the flag is being set
    /// (dot 1, scanline 241), the flag reads as `0` and the NMI for that frame is suppressed.
    ///
    pub const fn vblank(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// Sets or clears the vertical blank flag (bit 7).
    ///
    /// Set by the tick engine at scanline 241 dot 1; cleared at scanline 261 dot 1.
    pub(crate) const fn set_vblank(&mut self, vblank: bool) {
        self.0 = (self.0 & 0b0111_1111) | ((vblank as u8) << 7);
    }

    /// Sets or clears the sprite-0 hit flag (bit 6).
    ///
    /// Set by the pixel compositor when a non-transparent sprite-0 pixel overlaps a
    /// non-transparent background pixel (subject to clipping and the dot-255 exception).
    /// Cleared at scanline 261 dot 1.
    pub(crate) const fn set_sprite_0_hit(&mut self, hit: bool) {
        self.0 = (self.0 & 0b1011_1111) | ((hit as u8) << 6);
    }

    /// Sets or clears the sprite overflow flag (bit 5).
    ///
    /// Set when more than eight sprites are found on a scanline during OAM evaluation
    /// (subject to the hardware bug described on [`sprite_overflow`]). Cleared at scanline 261 dot 1.
    pub(crate) const fn set_sprite_overflow(&mut self, overflow: bool) {
        self.0 = (self.0 & 0b1101_1111) | ((overflow as u8) << 5);
    }
}

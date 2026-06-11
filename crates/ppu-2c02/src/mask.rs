// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU mask register (`$2001`), written by the CPU to control rendering visibility and color effects.
/// Stored as a raw byte because every bit pattern is valid and the byte is needed as-is for open-bus behavior.
///
/// ```text
/// 7  bit  0
/// ---- ----
/// BGRs bMmG
/// |||| ||||
/// |||| |||+- Greyscale (0: normal color; 1: greyscale display)
/// |||| |||
/// |||| ||+-- Background left-edge clip (0: clip; 1: show)
/// |||| ||
/// |||| |+--- Sprites left-edge clip    (0: clip; 1: show)
/// |||| |
/// |||| +---- Background enabled        (0: off; 1: on)
/// ||||
/// |||+------ Sprites enabled           (0: off; 1: on)
/// |||
/// ||+------- Emphasize red   (green on PAL/Dendy)
/// ||
/// |+-------- Emphasize green (red on PAL/Dendy)
/// |
/// +--------- Emphasize blue
/// ```
///
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PpuMask(pub u8);

impl PpuMask {
    /// Returns whether greyscale mode is enabled (bit 0).
    ///
    /// When true, the PPU ANDs every palette lookup with `$30`, which maps all colors to the
    /// grey column of the NES palette (`$00`, `$10`, `$20`, `$30`). The effect applies to both
    /// background and sprite output and is visible immediately on the next rendered dot.
    ///
    pub const fn greyscale(self) -> bool {
        self.0 & 0b0000_0001 != 0
    }

    /// Returns whether the leftmost 8 pixels of the background are clipped (bit 1 clear).
    ///
    /// When true (bit clear), the leftmost 8-pixel column is forced to the backdrop color regardless
    /// of what the background shift registers contain. Games clip this edge to hide partially-scrolled
    /// tiles at the left border of the playfield when scrolling horizontally.
    ///
    /// Has no effect unless [`background_enabled`] is also true.
    ///
    pub const fn background_left_clipped(self) -> bool {
        self.0 & 0b0000_0010 == 0
    }

    /// Returns whether the leftmost 8 pixels of sprites are clipped (bit 2 clear).
    ///
    /// When true (bit clear), all sprites within the leftmost 8-pixel column are suppressed. This
    /// is typically set alongside [`background_left_clipped`] to hide rendering artifacts at the
    /// left edge during horizontal scrolling, since the PPU cannot partially clip a tile at pixel
    /// granularity.
    ///
    /// Has no effect unless [`sprites_enabled`] is also true.
    ///
    pub const fn sprites_left_clipped(self) -> bool {
        self.0 & 0b0000_0100 == 0
    }

    /// Returns whether background rendering is enabled (bit 3).
    ///
    /// When false, the background layer outputs transparent pixels everywhere, leaving only the
    /// backdrop (universal background) color visible through any sprites. The PPU still runs its
    /// internal tile-fetch pipeline while this bit is clear, but the shift register output is
    /// suppressed.
    ///
    /// See also [`rendering_enabled`], which is true when either background or sprites are on.
    ///
    pub const fn background_enabled(self) -> bool {
        self.0 & 0b0000_1000 != 0
    }

    /// Returns whether sprite rendering is enabled (bit 4).
    ///
    /// When false, no sprites are drawn and sprite-0 hit and sprite-overflow detection are also
    /// disabled. The OAM evaluation pipeline still runs, but its output is suppressed before it
    /// reaches the pixel compositor.
    ///
    /// See also [`rendering_enabled`], which is true when either background or sprites are on.
    ///
    pub const fn sprites_enabled(self) -> bool {
        self.0 & 0b0001_0000 != 0
    }

    /// Returns whether the red channel is emphasized (bit 5).
    ///
    /// Emphasis dims the other two channels (green and blue) by roughly 12 %, making red
    /// appear more saturated relative to them. On **PAL and Dendy** hardware the R and G emphasis
    /// bits are swapped; bit 5 emphasizes green and bit 6 emphasizes red.
    ///
    /// The three emphasis bits can be combined freely; setting all three dims all channels equally,
    /// which darkens the screen without a hue shift.
    ///
    pub const fn emphasize_red(self) -> bool {
        self.0 & 0b0010_0000 != 0
    }

    /// Returns whether the green channel is emphasized (bit 6).
    ///
    /// Emphasis dims the other two channels (red and blue). On **PAL and Dendy** hardware the R and G
    /// emphasis bits are swapped; bit 6 emphasizes red and bit 5 emphasizes green.
    ///
    /// See [`emphasize_red`] for general notes on channel emphasis.
    ///
    pub const fn emphasize_green(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    /// Returns whether the blue channel is emphasized (bit 7).
    ///
    /// Emphasis dims the other two channels (red and green). The blue emphasis bit occupies the
    /// same position on both NTSC and PAL hardware.
    ///
    /// See [`emphasize_red`] for general notes on channel emphasis.
    ///
    pub const fn emphasize_blue(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// Returns whether any rendering is active (background or sprites, or both).
    ///
    /// Several PPU behaviors are gated on rendering being enabled: the `v`/`t` copy at frame and
    /// scanline boundaries, address bus activity during tile fetches, and sprite-0 hit detection.
    /// This is the canonical check used throughout the rendering pipeline.
    ///
    pub const fn rendering_enabled(self) -> bool {
        self.background_enabled() || self.sprites_enabled()
    }
}

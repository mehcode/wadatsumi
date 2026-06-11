// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU mask register (`$2001`). The CPU writes this to control what actually shows on
/// screen: background and sprite enables, left-edge clipping, greyscale, and the three
/// color-emphasis bits.
///
/// Stored as a raw byte so the written value round-trips intact, which matters for open-bus
/// reads where the PPU has to echo recently-written bytes back.
///
/// See <https://www.nesdev.org/wiki/PPU_registers#PPUMASK> for the canonical reference.
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
    /// Greyscale mode (bit 0).
    ///
    /// When set, the PPU ANDs every palette lookup with `$30`, snapping all colors to the
    /// grey column of the NES palette (`$00`, `$10`, `$20`, `$30`). Hits both backgrounds
    /// and sprites and takes effect on the very next dot.
    ///
    pub const fn greyscale(self) -> bool {
        self.0 & 0b0000_0001 != 0
    }

    /// Whether the leftmost 8 pixels of the background are clipped (bit 1 clear).
    ///
    /// When true (the bit is clear), the leftmost 8-pixel column is forced to the backdrop
    /// color no matter what the background shift registers hold. Games clip this edge to
    /// hide partially-scrolled tiles peeking in at the left while scrolling horizontally.
    ///
    /// Has no effect unless [`PpuMask::background_enabled`] is also true.
    ///
    pub const fn background_left_clipped(self) -> bool {
        self.0 & 0b0000_0010 == 0
    }

    /// Whether the leftmost 8 pixels of sprites are clipped (bit 2 clear).
    ///
    /// When true (the bit is clear), any sprite pixels in the leftmost 8-pixel column are
    /// suppressed. Usually set together with [`PpuMask::background_left_clipped`] so the
    /// left edge stays clean while scrolling. The PPU can't partially clip a tile at pixel
    /// granularity, so the choice is "all 8" or "none".
    ///
    /// Has no effect unless [`PpuMask::sprites_enabled`] is also true.
    ///
    pub const fn sprites_left_clipped(self) -> bool {
        self.0 & 0b0000_0100 == 0
    }

    /// Background rendering enable (bit 3).
    ///
    /// When clear, the background layer outputs transparent pixels everywhere, leaving only
    /// the universal backdrop color visible through any sprites. The tile-fetch pipeline
    /// still runs internally; only the shift register output is muted.
    ///
    /// See [`PpuMask::rendering_enabled`] for the "either layer is on" check the timing
    /// code uses.
    ///
    pub const fn background_enabled(self) -> bool {
        self.0 & 0b0000_1000 != 0
    }

    /// Sprite rendering enable (bit 4).
    ///
    /// When clear, no sprites are drawn, and sprite-0 hit and sprite overflow detection
    /// are also disabled. OAM evaluation still runs internally; the result just never
    /// reaches the pixel mixer.
    ///
    /// See [`PpuMask::rendering_enabled`] for the "either layer is on" check the timing
    /// code uses.
    ///
    pub const fn sprites_enabled(self) -> bool {
        self.0 & 0b0001_0000 != 0
    }

    /// Emphasize the red channel (bit 5).
    ///
    /// Emphasis attenuates the *other* two channels (green and blue) by roughly 12%, so red
    /// looks more saturated by comparison. On **PAL and Dendy** hardware bits 5 and 6 are
    /// swapped: bit 5 emphasizes green, bit 6 emphasizes red.
    ///
    /// The three emphasis bits combine freely. Setting all three dims every channel evenly,
    /// which fades the screen without shifting hue, often used for fade-to-black transitions.
    ///
    pub const fn emphasize_red(self) -> bool {
        self.0 & 0b0010_0000 != 0
    }

    /// Emphasize the green channel (bit 6).
    ///
    /// Emphasis attenuates the other two channels (red and blue). On **PAL and Dendy** hardware
    /// bits 5 and 6 are swapped: bit 6 emphasizes red, bit 5 emphasizes green.
    ///
    /// See [`PpuMask::emphasize_red`] for general notes on channel emphasis.
    ///
    pub const fn emphasize_green(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    /// Emphasize the blue channel (bit 7).
    ///
    /// Emphasis attenuates the other two channels (red and green). The blue emphasis bit
    /// stays in the same position on both NTSC and PAL hardware.
    ///
    /// See [`PpuMask::emphasize_red`] for general notes on channel emphasis.
    ///
    pub const fn emphasize_blue(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// True if either background or sprite rendering is on.
    ///
    /// A bunch of PPU behavior is gated on rendering being enabled: the `v`/`t` copies at
    /// frame and scanline boundaries, the address-bus activity during tile fetches, and
    /// sprite-0 hit detection. This is the canonical check the timing code uses.
    /// See <https://www.nesdev.org/wiki/PPU_rendering> for the full timing diagram.
    ///
    pub const fn rendering_enabled(self) -> bool {
        self.background_enabled() || self.sprites_enabled()
    }
}

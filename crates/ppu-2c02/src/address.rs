// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// A 15-bit PPU bus address with scroll position encoded into the address bits.
///
/// The type of the loopy `v` and `t` registers. Both live inside [`crate::Ppu2C02`] and
/// share the same bit layout:
///
/// ```text
/// yyy NN YYYYY XXXXX
/// ||| || ||||| +++++- coarse X (which tile column, 0..31)
/// ||| || +++++------- coarse Y (which tile row, 0..29 with 30 and 31 going off-screen)
/// ||| ++------------- nametable select (NN: nt_x then nt_y)
/// +++---------------- fine Y (which pixel row inside the tile, 0..7)
/// ```
///
/// Only bits [13:0] reach the PPU's external address bus; bit 14 is part of fine Y
/// storage and gets masked off on every fetch. The CPU never sees this register
/// directly — it pokes at it through `PPUCTRL` (`$2000`), `PPUSCROLL` (`$2005`),
/// `PPUADDR` (`$2006`), and `PPUDATA` (`$2007`).
///
/// See <https://www.nesdev.org/wiki/PPU_scrolling> for the canonical reference.
#[derive(Copy, Clone, Default)]
pub struct PpuAddress(pub u16);

impl PpuAddress {
    /// Constructs a zeroed address. `const` so callers can build one in a `const`
    /// context.
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns the 14-bit address driven onto the PPU's external bus.
    ///
    /// VRAM is 14-bit (`$0000`..`$3FFF`), so bit 14 (the top bit of fine Y) is masked
    /// off here even though it's part of the stored register.
    ///
    #[inline]
    #[must_use]
    pub const fn bus_address(self) -> u16 {
        self.0 & 0x3FFF
    }

    /// Advances the address by `step`, wrapping at the 15-bit boundary.
    ///
    /// Used by `PPUDATA` (`$2007`) auto-increment: `step` is 1 (one tile across) or 32
    /// (one tile down) depending on `PPUCTRL` bit 2.
    ///
    #[inline]
    pub fn advance(&mut self, step: u16) {
        self.0 = self.0.wrapping_add(step);
    }

    /// Sets the nametable-select bits [11:10] from the low 2 bits of `nametable`.
    ///
    /// Used by `PPUCTRL` (`$2000`) writes so a base-nametable change is already encoded
    /// in `t` before the next `t -> v` copy at the frame/scanline boundary.
    ///
    /// See <https://www.nesdev.org/wiki/PPU_scrolling#$2000_(PPUCTRL)_write>.
    ///
    #[inline]
    pub fn set_nametable(&mut self, nametable: u16) {
        self.0 = (self.0 & !0b1100_0000_0000) | ((nametable & 0b11) << 10);
    }

    /// Sets coarse X from the high 5 bits of a `PPUSCROLL` (`$2005`) first write.
    ///
    /// Fine X is not touched — it lives in its own register because the renderer reads
    /// it every dot to pick a bit out of the tile shift registers.
    ///
    /// See <https://www.nesdev.org/wiki/PPU_scrolling#$2005_first_write_(w_is_0)>.
    ///
    #[inline]
    pub fn set_scroll_x(&mut self, value: u8) {
        //   value[7:3] -> [4:0]  coarse X, which tile column (0..31)
        self.0 = (self.0 & !0b0001_1111) | (u16::from(value) >> 3);
    }

    /// Sets coarse Y and fine Y from a `PPUSCROLL` (`$2005`) second write.
    ///
    /// The high 5 bits land in coarse Y at [9:5]; the low 3 bits land in fine Y at
    /// [14:12].
    ///
    /// See <https://www.nesdev.org/wiki/PPU_scrolling#$2005_second_write_(w_is_1)>.
    ///
    #[inline]
    pub fn set_scroll_y(&mut self, value: u8) {
        self.0 = (self.0 & !0b0111_0011_1110_0000)
            //   value[7:3] -> [9:5]    coarse Y, which tile row (0..29)
            | ((u16::from(value) & 0b1111_1000) << 2)
            //   value[2:0] -> [14:12]  fine Y,   which pixel row inside the tile (0..7)
            | ((u16::from(value) & 0b0000_0111) << 12);
    }

    /// Sets bits [13:8] from the low 6 bits of a `PPUADDR` (`$2006`) first write.
    ///
    /// Bit 14 is forced clear: VRAM is 14-bit, so the top 2 bits of the input are
    /// discarded.
    ///
    /// See <https://www.nesdev.org/wiki/PPU_scrolling#$2006_first_write_(w_is_0)>.
    ///
    #[inline]
    pub fn set_address_high(&mut self, value: u8) {
        self.0 = (self.0 & 0x00FF) | ((u16::from(value) & 0b0011_1111) << 8);
    }

    /// Sets bits [7:0] from a `PPUADDR` (`$2006`) second write.
    ///
    /// Callers also need to copy `t` into `v` after this write so `PPUDATA` takes
    /// effect immediately.
    ///
    /// See <https://www.nesdev.org/wiki/PPU_scrolling#$2006_second_write_(w_is_1)>.
    ///
    #[inline]
    pub fn set_address_low(&mut self, value: u8) {
        self.0 = (self.0 & 0xFF00) | u16::from(value);
    }
}

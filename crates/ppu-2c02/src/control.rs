// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU control register (`$2000`). The CPU writes this to set high-level rendering
/// options: which pattern tables backgrounds and sprites are fetched from, sprite size,
/// NMI-on-vblank enable, and the base nametable.
///
/// Stored as a raw byte so the written value round-trips intact, which matters for
/// open-bus reads where the PPU has to echo recently-written bytes back.
///
/// See <https://www.nesdev.org/wiki/PPU_registers#PPUCTRL> for the canonical reference.
///
/// ```text
/// 7  bit  0
/// ---- ----
/// VPHB SINN
/// |||| ||||
/// |||| ||++- Base nametable address
/// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
/// |||| ||
/// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
/// |||| |     (0: add 1, going across; 1: add 32, going down)
/// |||| |
/// |||| +---- Sprite pattern table address for 8x8 sprites
/// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
/// ||||
/// |||+------ Background pattern table address (0: $0000; 1: $1000)
/// |||
/// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels, see PPU OAM byte 1)
/// ||
/// |+-------- PPU master/slave select
/// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
/// |
/// +--------- Vblank NMI enable (0: off, 1: on)
/// ```
///
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PpuControl(pub u8);

impl PpuControl {
    /// The 2-bit nametable select (bits 0..1) as a raw value of 0, 1, 2, or 3.
    ///
    /// This is the raw field, not a VRAM address. It gets packed into `t[11:10]` on
    /// every `$2000` write so the nametable bits are already in `t` by the time the
    /// PPU copies `t` into `v` at the start of each frame.
    ///
    /// Use [`PpuControl::nametable_address`] when you actually need a VRAM base address.
    pub const fn nametable_index(self) -> u16 {
        (self.0 & 0b0000_0011) as u16
    }

    /// VRAM base address of the active nametable: `$2000`, `$2400`, `$2800`, or `$2C00`.
    ///
    /// The four nametables form a 2x2 grid in VRAM:
    ///
    /// ```text
    /// 0 ($2000) | 1 ($2400)
    /// ----------+----------
    /// 2 ($2800) | 3 ($2C00)
    /// ```
    ///
    /// Bit 10 picks the horizontal nametable (left vs right) and bit 11 picks the vertical
    /// one (top vs bottom). That layout matches how the NES mirrors VRAM, so nametables
    /// adjacent in this grid stay adjacent in the scrolled image.
    ///
    /// Use [`PpuControl::nametable_index`] when you need the raw 2-bit value for packing
    /// into a register field (such as `t[11:10]`).
    pub const fn nametable_address(self) -> u16 {
        0x2000 | (self.nametable_index() << 10)
    }

    /// NMI-on-vblank enable (bit 7).
    ///
    /// When set, the PPU asserts the CPU's NMI line on the first dot of scanline 241,
    /// the same dot it sets the vblank flag in `PPUSTATUS`. The CPU services the
    /// interrupt as soon as the current instruction finishes.
    ///
    /// When clear, vblank still happens and the flag still gets set, but no NMI fires.
    /// Games that don't use NMI just poll `$2002` instead.
    ///
    /// Toggling this bit off and then back on while the vblank flag is still up will
    /// fire a fresh NMI. Games occasionally lean on that to schedule extra work mid-vblank.
    pub const fn nmi_enabled(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// VRAM auto-increment applied after each `$2007` (`PPUDATA`) access: `1` or `32`.
    ///
    /// - `1` walks across a nametable one tile at a time (left to right). Natural for
    ///   streaming a row of tile data.
    ///
    /// - `32` walks down a nametable one row at a time (top to bottom). Handy for writing
    ///   a column of tiles with consecutive `$2007` writes.
    ///
    /// Both reads and writes advance `v` by this amount (see
    /// [`Ppu2C02::cpu_read`](crate::Ppu2C02::cpu_read) and
    /// [`Ppu2C02::cpu_write`](crate::Ppu2C02::cpu_write)).
    pub const fn vram_increment(self) -> u16 {
        if self.0 & 0b0000_0100 != 0 { 32 } else { 1 }
    }

    /// Base CHR address for 8x8 sprite tiles (bit 3): `$0000` or `$1000`.
    ///
    /// The PPU adds each sprite's 8-bit tile index to this base to form the fetch address.
    /// Tile `$05` with a base of `$1000` reads from `$1050`.
    ///
    /// **Ignored in 8x16 mode.** When [`PpuControl::sprite_height`] is 16 the lowest bit
    /// of each OAM tile byte picks the pattern table directly (`0` to `$0000`, `1` to
    /// `$1000`), and this bit has no effect on sprite rendering.
    pub const fn sprite_pattern_table(self) -> u16 {
        if self.0 & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 }
    }

    /// Base CHR address for background tiles (bit 4): `$0000` or `$1000`.
    ///
    /// During background rendering the PPU fetches two bit-planes per tile. The full CHR
    /// address looks like this:
    ///
    /// ```text
    /// CHR addr = background_pattern_table | (tile_id << 4) | fine_y | bit_plane
    /// ```
    ///
    /// `tile_id` is the byte read from the nametable, `fine_y` (0..7) picks the row
    /// inside the tile, and `bit_plane` is `0` for the low plane or `8` for the high one.
    ///
    /// Many mappers bank-swap CHR ROM, so this bit lets a game choose which bank backs
    /// the background independently of sprites.
    pub const fn background_pattern_table(self) -> u16 {
        if self.0 & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 }
    }

    /// Sprite height in pixels (bit 5): `8` (8x8 mode) or `16` (8x16 mode).
    ///
    /// In **8x8 mode** each sprite is one tile and the OAM tile byte is just an index into
    /// the pattern table selected by [`PpuControl::sprite_pattern_table`].
    ///
    /// In **8x16 mode** each sprite is two tiles stacked vertically. The OAM tile byte is
    /// re-interpreted: bit 0 picks the pattern table (`0` to `$0000`, `1` to `$1000`),
    /// and bits 1..7 are the index of the top tile. The bottom tile is always `top + 1`,
    /// so the two halves have to sit next to each other in CHR.
    ///
    /// [`PpuControl::sprite_pattern_table`] is ignored entirely.
    pub const fn sprite_height(self) -> u16 {
        if self.0 & 0b0010_0000 != 0 { 16 } else { 8 }
    }

    /// PPU master/slave select (bit 6).
    ///
    /// Almost no software touches this. In master mode (clear, the normal case) the PPU
    /// samples its six EXT pins to receive a backdrop color from an external device. On
    /// a stock NES nothing drives those pins, so the input floats and the PPU just uses
    /// its own background color as the backdrop.
    ///
    /// In slave mode (set) the PPU drives the EXT pins with its current background palette
    /// index instead of reading them. The original intent was to daisy-chain a second PPU
    /// for compositing external video into the background layer. Setting this bit on a
    /// real Famicom reportedly draws extra current from the pak port; no commercial
    /// software uses it.
    pub const fn ext_output(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
}

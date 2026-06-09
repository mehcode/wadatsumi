// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// The PPU control register (`$2000`), written by the CPU to configure major rendering parameters.
/// Stored as a raw byte because every bit pattern is valid and the byte is needed as-is for open-bus behavior.
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
/// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
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
    /// Returns the nametable select field (bits [1:0]) as a raw 2-bit index (0–3).
    ///
    /// This is the raw bit field value, **not** a VRAM address. It is used when the index
    /// must be packed into a register field — specifically, `cpu_write` encodes it into
    /// `t[11:10]` on every `$2000` write so that the nametable is already embedded in the
    /// Loopy `t` register before `t` is copied to `v` at frame/scanline start.
    ///
    /// Use [`nametable_address`] when the PPU needs the actual VRAM fetch address.
    ///
    pub const fn nametable_index(self) -> u16 {
        (self.0 & 0b0000_0011) as u16
    }

    /// Returns the VRAM base address of the active nametable: `$2000`, `$2400`, `$2800`, or `$2C00`.
    ///
    /// The four nametables are laid out in VRAM as a 2×2 grid:
    ///
    /// ```text
    /// 0 ($2000) | 1 ($2400)
    /// ----------+----------
    /// 2 ($2800) | 3 ($2C00)
    /// ```
    ///
    /// The rendering pipeline uses this address as the starting point for background tile fetches.
    /// Bit 10 selects the horizontal nametable (left vs. right column) and bit 11 selects the
    /// vertical nametable (top vs. bottom row), matching the NES mirroring geometry.
    ///
    /// Use [`nametable_index`] when the raw 2-bit value is needed to pack into a register field
    /// (e.g., `t[11:10]`).
    ///
    pub const fn nametable_address(self) -> u16 {
        0x2000 | (self.nametable_index() << 10)
    }

    /// Returns whether the PPU is configured to assert NMI at the start of vertical blank (bit 7).
    ///
    /// When true, the PPU pulls the CPU's NMI line low on the first dot of scanline 241 (the
    /// start of vblank), immediately after setting the vblank flag in `PPUSTATUS` (`$2002`) bit 7.
    /// The CPU services the interrupt as soon as its current instruction completes.
    ///
    /// When false, vblank still occurs and the flag is still set, the CPU can detect it by
    /// polling `$2002`, but no interrupt is generated.
    ///
    /// Games clear the vblank flag by reading `$2002` and re-arm it each frame by writing this
    /// bit back to `1`, so the NMI handler runs exactly once per frame. Toggling this bit off
    /// and on during vblank is a known technique for manually triggering an extra NMI.
    ///
    pub const fn nmi_enabled(self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    /// Returns the number of bytes the VRAM address (`v`) advances after each `PPUDATA` access (bit 2).
    ///
    /// Returns `1` (bit clear) or `32` (bit set).
    ///
    /// - **Increment by 1** steps horizontally across the nametable one tile at a time (left → right),
    ///   which is the natural layout for streaming row-major background data.
    ///
    /// - **Increment by 32** steps vertically down the nametable one tile row at a time (top → bottom),
    ///   useful for writing entire columns of tiles with sequential `$2007` writes.
    ///
    /// Both reads and writes through `$2007` advance `v` by this amount (see [`Ppu2C02::cpu_read`]
    ///
    /// and [`Ppu2C02::cpu_write`]).
    pub const fn vram_increment(self) -> u16 {
        if self.0 & 0b0000_0100 != 0 { 32 } else { 1 }
    }

    /// Returns the base CHR address for fetching 8×8 sprite tiles: `$0000` (bit clear) or `$1000` (bit set).
    ///
    /// The PPU adds each sprite's 8-bit tile index to this base to form the CHR fetch address.
    /// For example, tile index `$05` with a base of `$1000` reads from `$1005` in CHR space.
    ///
    /// **This field is ignored in 8×16 sprite mode** ([`sprite_height`] returns 16). In that mode
    /// the lowest bit of each OAM tile byte selects the pattern table directly (`0` → `$0000`,
    /// `1` → `$1000`), and this control register bit has no effect on sprite rendering.
    ///
    pub const fn sprite_pattern_table(self) -> u16 {
        if self.0 & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 }
    }

    /// Returns the base CHR address for fetching background tiles: `$0000` (bit clear) or `$1000` (bit set).
    ///
    /// During background rendering the PPU fetches two bit-planes for each tile. The full CHR address
    /// is formed as:
    ///
    /// ```text
    /// CHR address = background_pattern_table() | (tile_id << 4) | fine_y | bit_plane
    /// ```
    ///
    /// where `tile_id` is the byte read from the nametable, `fine_y` (0–7) selects the pixel row
    /// within the tile, and `bit_plane` is `0` for the low bit-plane or `8` for the high bit-plane.
    ///
    /// Many cartridges bank-switch CHR ROM so that the pattern tables can be swapped at runtime;
    /// this bit lets the game choose which bank is active for backgrounds independently of sprites.
    ///
    pub const fn background_pattern_table(self) -> u16 {
        if self.0 & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 }
    }

    /// Returns the height of each sprite in pixels: `8` (bit clear, 8×8 mode) or `16` (bit set, 8×16 mode).
    ///
    /// In **8×8 mode** sprites are always one tile tall. The tile byte in OAM is an index into
    /// the pattern table selected by [`sprite_pattern_table`].
    ///
    /// In **8×16 mode** sprites are two tiles tall (top tile immediately above bottom tile). The OAM
    /// tile byte is re-interpreted: bit 0 selects the pattern table (`0` → `$0000`, `1` → `$1000`),
    /// and bits [7:1] are the index of the top tile. The bottom tile is always the top tile index
    /// plus one, so both tiles must be stored contiguously in CHR. [`sprite_pattern_table`] is
    /// ignored entirely in this mode.
    ///
    pub const fn sprite_height(self) -> u16 {
        if self.0 & 0b0010_0000 != 0 { 16 } else { 8 }
    }

    /// Returns whether the PPU is in EXT output mode (bit 6, the master/slave select).
    ///
    /// When false (master mode, the normal case), the PPU samples the six EXT pins to receive a
    /// backdrop color from an external device during transparent pixel output. Nearly all NES
    /// software leaves this bit clear and no consumer cartridge drives the EXT pins, so the input
    /// floats and the PPU renders its own background color as the backdrop.
    ///
    /// When true (slave mode), the PPU drives the EXT pins with the current background palette
    /// color index instead of reading them. This was intended for daisy-chaining a second PPU to
    /// composite an external video signal into the background layer. Setting this bit on a standard
    /// Famicom reportedly draws extra current from the cartridge port; it is effectively unused by
    /// any commercial software.
    ///
    pub const fn ext_output(self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
}

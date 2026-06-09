// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::PpuReadWrite;
use crate::control::PpuControl;
use crate::mask::PpuMask;
use crate::status::PpuStatus;

mod cpu;

/// The Ricoh 2C02 Picture Processing Unit (PPU).
///
/// Responsible for generating the NES video signal: fetches tiles and sprites from CHR memory,
/// composites them into a 256×240 pixel image, and outputs a pixel each clock cycle.
/// Runs at 3× the CPU clock rate (approximately 5.37 MHz NTSC).
///
/// The CPU communicates with the PPU through eight memory-mapped registers at `$2000`–`$2007`
/// and via DMA writes to OAM.
pub struct Ppu2C02 {
    /// Control register (`$2000`). See [`PpuControl`] for bit definitions.
    control: PpuControl,

    /// Mask register (`$2001`). See [`PpuMask`] for bit definitions.
    mask: PpuMask,

    /// Status register (`$2002`). See [`PpuStatus`] for bit definitions.
    status: PpuStatus,

    /// Object attribute memory (OAM). Stores up to 64 sprites as 4 bytes each:
    /// Y position, tile index, attribute flags, and X position.
    oam: [u8; 256],

    /// OAM byte address for the next `OAMDATA` (`$2004`) read or write.
    /// Auto-increments after each `OAMDATA` write; written directly by `OAMADDR` (`$2003`).
    oam_address: u8,

    /// Current VRAM address, the Loopy `v` register (15 bits).
    ///
    /// Bit layout: `fine_y[14:12] | nt_y[11] | nt_x[10] | coarse_y[9:5] | coarse_x[4:0]`
    ///
    /// Used as the PPU's live pointer during rendering: advanced each dot through nametable
    /// and attribute fetches, and exposed to the CPU via `PPUADDR`/`PPUDATA`.
    v: u16,

    /// Temporary VRAM address, the Loopy `t` register (15 bits).
    ///
    /// Shares the same bit layout as `v` (see above).
    ///
    /// Acts as a staging register: CPU writes to `PPUSCROLL` (`$2005`) and `PPUADDR` (`$2006`)
    /// update `t`. `t` is copied into `v` at the start of each frame (all bits) and at the
    /// start of each visible scanline (horizontal bits only).
    t: u16,

    /// Fine X scroll (3 bits). Selects the pixel column within the current 8-pixel tile column.
    /// Set by the first write to `PPUSCROLL` (`$2005`).
    x: u8,

    /// First/second write toggle for `PPUSCROLL` (`$2005`) and `PPUADDR` (`$2006`).
    /// `false` = first write pending, `true` = second write pending.
    /// Cleared to `false` by any read of `PPUSTATUS` (`$2002`).
    w: bool,

    /// Palette RAM (32 bytes). Internal to the PPU chip; not on the external bus.
    ///
    /// Layout: 16 background entries followed by 16 sprite entries. Entries at indices
    /// `$10`, `$14`, `$18`, `$1C` mirror `$00`, `$04`, `$08`, `$0C` (the universal
    /// background color appears in both the background and sprite backdrop slots).
    palette: [u8; 32],

    /// Read-ahead buffer for `PPUDATA` (`$2007`).
    ///
    /// Reads from non-palette VRAM (`$0000`–`$3EFF`) return the stale contents of this buffer,
    /// then refill it from VRAM. Reads from palette space (`$3F00`–`$3FFF`) bypass the buffer
    /// and return the value directly, though the buffer is still updated with the mirrored
    /// nametable byte at the same address.
    data_read_buffer: u8,

    /// Last byte driven onto the CPU data bus during any PPU register access (`$2000`–`$2007`).
    ///
    /// CMOS bus lines float rather than being actively pulled to zero when no device drives them.
    /// The capacitance of the bus holds the previous value for a short time, this is "open-bus"
    /// behavior. The PPU exploits it: write-only registers return whatever was last on the bus
    /// rather than a defined value, and readable registers that only drive some bits fill the rest
    /// from the same source.
    io_latch: u8,
}

impl Ppu2C02 {
    pub const fn new() -> Self {
        Self {
            control: PpuControl(0),
            mask: PpuMask(0),
            status: PpuStatus(0),
            oam: [0; 256],
            oam_address: 0,
            v: 0,
            t: 0,
            x: 0,
            w: false,
            palette: [0; 32],
            data_read_buffer: 0,
            io_latch: 0,
        }
    }

    /// Translates a PPU bus address in `$3F00`–`$3FFF` to a `palette` array index.
    ///
    /// Mirrors `$3F20`–`$3FFF` down into `$3F00`–`$3F1F`, then folds the backdrop
    /// slots (`$3F10`, `$3F14`, `$3F18`, `$3F1C`) onto their background counterparts
    /// (`$3F00`, `$3F04`, `$3F08`, `$3F0C`).
    #[inline]
    const fn palette_index(address: u16) -> usize {
        let idx = (address & 0x1F) as usize;
        if idx >= 0x10 && idx & 0x03 == 0 { idx & 0x0F } else { idx }
    }

    #[inline]
    pub(crate) fn palette_read(&self, address: u16) -> u8 {
        self.palette[Self::palette_index(address)]
    }

    #[inline]
    pub(crate) fn palette_write(&mut self, address: u16, value: u8) {
        self.palette[Self::palette_index(address)] = value;
    }

    pub fn tick<B: PpuReadWrite>(&mut self, bus: &mut B) {
        todo!()
    }
}

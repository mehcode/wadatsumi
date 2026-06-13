// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::PpuReadWrite;
use crate::address::PpuAddress;
use crate::clock::PpuFrameClock;
use crate::control::PpuControl;
use crate::mask::PpuMask;
use crate::status::PpuStatus;

mod cpu;

/// The Ricoh 2C02 Picture Processing Unit (PPU).
///
/// This is the chip that generates the NES video signal. It fetches tile and sprite data
/// from CHR memory through the pak, composites them into a 256x240 pixel image, and
/// shifts out one pixel per dot. It runs at three dots per CPU cycle, roughly 5.37 MHz on
/// NTSC.
///
/// The CPU talks to the PPU through eight memory-mapped registers at `$2000`..`$2007` (with
/// mirrors all the way up to `$3FFF`), plus OAM DMA via `$4014`. Everything timing-related
/// lives in [`Ppu2C02::tick`], driven by the host one dot at a time.
///
/// References used throughout this file:
/// - <https://www.nesdev.org/wiki/PPU>
/// - <https://www.nesdev.org/wiki/PPU_rendering>
/// - <https://www.nesdev.org/wiki/PPU_scrolling> (the "loopy" scrolling registers)
/// - <https://www.nesdev.org/wiki/PPU_frame_timing>
pub struct Ppu2C02 {
    /// Control register (`$2000`). See [`PpuControl`] for bit definitions.
    control: PpuControl,

    /// Mask register (`$2001`). See [`PpuMask`] for bit definitions.
    mask: PpuMask,

    /// Status register (`$2002`). See [`PpuStatus`] for bit definitions.
    status: PpuStatus,

    /// Object attribute memory (OAM): 64 sprites, 4 bytes each (Y position, tile index,
    /// attribute flags, X position). See <https://www.nesdev.org/wiki/PPU_OAM>.
    oam: [u8; 256],

    /// OAM byte address used by the next `OAMDATA` (`$2004`) read or write.
    /// Written directly by `OAMADDR` (`$2003`); auto-increments after each `OAMDATA` write.
    oam_address: u8,

    /// Current VRAM address, the loopy `v` register. See [`PpuAddress`] for the bit
    /// layout.
    ///
    /// This is the PPU's live pointer during rendering. It advances each dot through the
    /// nametable, attribute, and pattern fetches, and the CPU also pokes at it via
    /// `PPUADDR`/`PPUDATA`.
    ///
    v: PpuAddress,

    /// Temporary VRAM address, the loopy `t` register. Same bit layout as [`PpuAddress`].
    ///
    /// Think of this as the scroll origin: CPU writes to `PPUSCROLL` (`$2005`) and
    /// `PPUADDR` (`$2006`) update `t`, never `v` directly (except `PPUADDR`'s second
    /// write, which also latches `t` into `v`). At the start of each frame `t` gets copied
    /// into `v` in full, and at the start of each visible scanline only the horizontal
    /// bits get copied across.
    ///
    t: PpuAddress,

    /// Fine X scroll (3 bits). Picks which pixel column inside the current 8-pixel tile
    /// column ends up at screen X = 0. Set by the first write to `PPUSCROLL` (`$2005`).
    ///
    /// Lives in its own register (instead of inside `t`) because the renderer reads it
    /// every dot to select a bit out of the tile shift registers.
    ///
    x: u8,

    /// First/second write toggle shared by `PPUSCROLL` (`$2005`) and `PPUADDR` (`$2006`).
    /// `false` means "first write pending", `true` means "second write pending".
    /// Any read of `PPUSTATUS` (`$2002`) resets it back to `false`.
    ///
    w: bool,

    /// Palette RAM (32 bytes). Lives inside the PPU chip, not on the external bus.
    ///
    /// First 16 bytes are background entries, next 16 are sprite entries. Indices `$10`,
    /// `$14`, `$18`, and `$1C` mirror `$00`, `$04`, `$08`, and `$0C`, so the universal
    /// background color shows up in both the background and sprite backdrop slots.
    /// See <https://www.nesdev.org/wiki/PPU_palettes>.
    ///
    palette: [u8; 32],

    /// Read-ahead buffer for `PPUDATA` (`$2007`) reads.
    ///
    /// Reads from non-palette VRAM (`$0000`..`$3EFF`) return the *previous* buffer value
    /// and then refill the buffer from VRAM at the new address. So the very first read
    /// after setting `PPUADDR` returns stale garbage and has to be discarded.
    ///
    /// Reads from palette space (`$3F00`..`$3FFF`) skip the delay and return the palette
    /// byte directly, but the buffer still refills from the underlying nametable address
    /// (`v & $2FFF`) as a side effect, because that's still what the PPU bus sees.
    ///
    data_read_buffer: u8,

    /// Last byte driven onto the CPU data bus during any PPU register access
    /// (`$2000`..`$2007`).
    ///
    /// CMOS bus lines float instead of being actively pulled to zero when nothing drives
    /// them; the line's capacitance holds the previous value for a short time. That's
    /// "open-bus" behavior, and the PPU leans on it: write-only registers return whatever
    /// was last on the bus, and readable registers that only drive some bits fill the
    /// rest from this latch.
    ///
    io_latch: u8,

    /// Frame-timing position: which dot of which scanline the PPU is currently processing,
    /// plus the parity bit and absolute frame count. Advanced one dot per [`Self::tick`].
    /// See [`PpuFrameClock`] for the dot/scanline layout.
    ///
    pub frame: PpuFrameClock,
}

impl Default for Ppu2C02 {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu2C02 {
    /// Constructs a fresh PPU with every register, OAM byte, and palette entry zeroed.
    /// Equivalent to [`Ppu2C02::default`]; provided as a `const fn` so callers can build
    /// one in a `const` context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            control: PpuControl(0),
            mask: PpuMask(0),
            status: PpuStatus(0),
            oam: [0; 256],
            oam_address: 0,
            v: PpuAddress::new(),
            t: PpuAddress::new(),
            x: 0,
            w: false,
            palette: [0; 32],
            data_read_buffer: 0,
            io_latch: 0,
            frame: PpuFrameClock::new(),
        }
    }

    /// Translates a PPU bus address in `$3F00`..`$3FFF` to a `palette` array index.
    ///
    /// Two folds happen. First, anything in `$3F20`..`$3FFF` mirrors back down into
    /// `$3F00`..`$3F1F` (palette space is only 32 bytes). Second, the four sprite-backdrop
    /// slots (`$3F10`, `$3F14`, `$3F18`, `$3F1C`) mirror onto their background-backdrop
    /// counterparts (`$3F00`, `$3F04`, `$3F08`, `$3F0C`) so the universal background color
    /// shows up in both.
    ///
    #[inline]
    const fn palette_index(address: u16) -> usize {
        let index = (address & 0x1F) as usize;

        if index >= 0x10 && index.trailing_zeros() >= 2 { index & 0x0F } else { index }
    }

    /// Reads a byte from palette RAM, with `$3F00`..`$3FFF` mirroring applied via
    /// [`Self::palette_index`].
    #[inline]
    pub(crate) fn palette_read(&self, address: u16) -> u8 {
        self.palette[Self::palette_index(address)]
    }

    /// Writes a byte to palette RAM, with `$3F00`..`$3FFF` mirroring applied via
    /// [`Self::palette_index`].
    #[inline]
    pub(crate) fn palette_write(&mut self, address: u16, value: u8) {
        self.palette[Self::palette_index(address)] = value;
    }

    /// Current level of the PPU's `/NMI` output line: `true` while the PPU is asking the
    /// CPU for an NMI, `false` otherwise. Modeled active-high (the real pin is active-low).
    ///
    /// The PPU drives this as `vblank_flag AND PPUCTRL bit 7`, so it goes high at `(241,
    /// 1)` if NMI is enabled, stays high through vblank, and falls again at `(261, 1)`
    /// when the vblank flag clears. A `$2002` read clears the flag and pulls the line low
    /// immediately. Writing `$2000` to flip NMI-enable on while vblank is still set
    /// re-asserts the line, the "multi-NMI" trick.
    ///
    /// The CPU does its own rising-edge latching on this; the level read here is just
    /// what the line *is*, not what the CPU has decided to do about it.
    ///
    #[must_use]
    pub const fn nmi(&self) -> bool {
        self.status.vblank() && self.control.nmi_enabled()
    }

    /// Advances the PPU by one dot.
    ///
    /// The host is expected to call this three times per CPU cycle on NTSC (interleaved
    /// with `Cpu::tick`) so the dot/scanline counters stay in step with CPU-side timing
    /// such as `$2002` race conditions and OAM DMA. `bus` carries CHR fetches, nametable
    /// reads, and mapper-visible bus traffic.
    ///
    /// See <https://www.nesdev.org/wiki/PPU_frame_timing> for the full dot timeline.
    ///
    pub fn tick<B: PpuReadWrite>(&mut self, _bus: &mut B) {
        // Edge events fire on the current (scanline, dot), then we advance the clock to
        // the next dot. So on tick N the PPU sees the state it should produce *for* dot
        // N; the next tick sees N+1. The 2C02 hardware reference uses this same
        // convention (work happens "during" a dot, then the counter increments).

        // Both flag edges fire at dot 1, so check dot first. 340 of every 341 dots fall
        // straight through on the first compare; only one in 341 even consults the
        // scanline. With this tick inlined 3× per CPU cycle, that's the difference
        // between ~6 branches and ~3 branches per cycle in the common case.
        if self.frame.dot == 1 {
            match self.frame.scanline {
                // Vblank entry: set the flag at dot 1 of scanline 241. NMI assertion is
                // wired up separately once the PPU exposes its output line; this just
                // owns the flag.
                241 => {
                    self.status.set_vblank(true);
                }

                // Pre-render: clear vblank, sprite-0 hit, and sprite overflow at dot 1
                // of scanline 261. Hardware clears all three on the same dot.
                261 => {
                    self.status.set_vblank(false);
                    self.status.set_sprite_0_hit(false);
                    self.status.set_sprite_overflow(false);
                }

                _ => {}
            }
        }

        // Advance one dot. The NTSC pre-render dot-skip at (261, 339) on odd frames is
        // gated by whether rendering is on *right now*, so we sample the mask at call
        // time rather than caching it.
        self.frame.advance(self.mask.rendering_enabled());
    }
}

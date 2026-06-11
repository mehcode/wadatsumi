// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Ppu2C02, PpuReadWrite};

impl Ppu2C02 {
    /// Advances the two-write latch and returns whether this is the second write.
    ///
    /// The latch starts at `false` (first write pending) and flips on every call. So this
    /// returns `false` on call 1, `true` on call 2, `false` on call 3, and so on. A read
    /// of `PPUSTATUS` (`$2002`) can reset it back to `false` at any time.
    ///
    /// `PPUSCROLL` (`$2005`) and `PPUADDR` (`$2006`) both need two CPU writes to set their
    /// 15-bit value, and they share this single latch. That's why games are religious
    /// about reading `$2002` at the top of vblank: it resets the latch so the next
    /// `$2005`/`$2006` write definitely lands as the "first" half.
    ///
    const fn cpu_second_write(&mut self) -> bool {
        let result = self.w;
        self.w = !self.w;
        result
    }

    /// Reads a CPU-facing PPU register without any side effects, for debuggers and memory
    /// viewers.
    ///
    /// `index` is the register index (0..7), i.e. the low three bits of a CPU address in
    /// `$2000`..`$3FFF`. Unlike [`Ppu2C02::cpu_read`], this never clears the vblank flag,
    /// resets `w`, or advances the read-ahead buffer.
    ///
    #[must_use]
    pub fn cpu_peek(&self, index: u8) -> u8 {
        match index {
            // Status register (`$2002`).
            // Mirrors what cpu_read would return, but without clearing vblank and without
            // touching `w`. We're observing state, not consuming it.
            2 => (self.status.0 & 0b1110_0000) | (self.io_latch & 0b0001_1111),

            // OAM data register (`$2004`). No side effects on read either way.
            4 => self.oam[self.oam_address as usize],

            // Data register (`$2007`).
            7 => {
                let address = self.v.bus_address();

                if address >= 0x3F00 {
                    self.palette_read(address)
                } else {
                    // For non-palette addresses cpu_read would hand back the current
                    // buffer contents (and refill it). We don't refill; the buffer value
                    // is the byte cpu_read would return, so this stays exact.
                    self.data_read_buffer
                }
            }

            _ => self.io_latch,
        }
    }

    /// Reads a CPU-facing PPU register, with all the hardware side effects.
    ///
    /// `index` is the register index (0..7), i.e. the low three bits of a CPU address in
    /// `$2000`..`$3FFF`. Most registers are write-only; reading them just returns
    /// `io_latch` (the last byte on the CPU data bus). The exceptions are `PPUSTATUS` (2),
    /// `OAMDATA` (4), and `PPUDATA` (7): each drives some or all bits of the result,
    /// merges the undriven bits in from `io_latch`, and then stores the whole byte back
    /// into `io_latch` before returning.
    ///
    #[expect(clippy::match_same_arms)]
    pub fn cpu_read<B: PpuReadWrite>(&mut self, index: u8, bus: &mut B) -> u8 {
        let value = match index {
            // Control register (`$2000`). Write-only; returns io_latch.
            0 => None,

            // Mask register (`$2001`). Write-only; returns io_latch.
            1 => None,

            // Status register (`$2002`).
            // <https://www.nesdev.org/wiki/PPU_registers#PPUSTATUS>
            2 => {
                // Only bits 7..5 are driven by the PPU. Bits 4..0 float and pick up
                // whatever io_latch holds from the last bus access.
                let value = (self.status.0 & 0b1110_0000) | (self.io_latch & 0b0001_1111);

                // Reading clears bit 7 (vblank) immediately. That's the standard way to
                // detect vblank entry while making sure software only sees it once per
                // frame.
                self.status.0 &= !0b1000_0000;

                // Reading also resets the `w` latch to "first write". Games rely on this
                // so the next `$2005`/`$2006` write lands as the first half of the pair
                // instead of finishing a stale one.
                self.w = false;

                Some(value)
            }

            // OAM address register (`$2003`). Write-only; returns io_latch.
            3 => None,

            // OAM data register (`$2004`).
            // <https://www.nesdev.org/wiki/PPU_registers#OAMDATA>
            // Returns the byte at the current OAM address. Only writes advance
            // oam_address; reads leave it alone.
            4 => Some(self.oam[self.oam_address as usize]),

            // Scroll register (`$2005`). Write-only; returns io_latch.
            5 => None,

            // Address register (`$2006`). Write-only; returns io_latch.
            6 => None,

            // Data register (`$2007`).
            // <https://www.nesdev.org/wiki/PPU_registers#PPUDATA>
            7 => {
                let address = self.v.bus_address();

                let value = if address >= 0x3F00 {
                    // Palette RAM is internal to the PPU, so palette reads come back
                    // immediately with no buffering delay. We *still* refresh the buffer
                    // with the nametable byte at the mirrored address (`v & $2FFF`),
                    // because the PPU's address bus also sees that during the access.
                    self.data_read_buffer = bus.ppu_read(address & 0x2FFF);
                    self.palette_read(address)
                } else {
                    // The CPU can't see the PPU's address/data buses directly, so
                    // non-palette reads come back one access late through a read-ahead
                    // buffer: the CPU gets the stale buffer contents this cycle while
                    // the buffer quietly refills from VRAM for next time.
                    let stale = self.data_read_buffer;
                    self.data_read_buffer = bus.ppu_read(address);
                    stale
                };

                self.v.advance(self.control.vram_increment());

                Some(value)
            }

            // Callers mask to 3 bits (`& 7`), so this is unreachable in practice.
            _ => None,
        };

        if let Some(value) = value {
            self.io_latch = value;
        }

        value.unwrap_or(self.io_latch)
    }

    /// Writes a CPU-facing PPU register.
    ///
    /// `index` is the register index (0..7), i.e. the low three bits of a CPU address in
    /// `$2000`..`$3FFF`. Callers are expected to mask the address themselves.
    ///
    #[expect(clippy::match_same_arms)]
    pub fn cpu_write<B: PpuReadWrite>(&mut self, index: u8, value: u8, bus: &mut B) {
        // Latch the written byte so subsequent reads of write-only registers (and the
        // undriven bits of `$2002`) echo it back.
        self.io_latch = value;

        match index {
            // Control register (`$2000`).
            0 => {
                self.control.0 = value;

                // Mirror the nametable select bits (0..1) into t[11:10] (nt_x, nt_y) so
                // the right base nametable is already encoded in `t` before the PPU
                // copies `t` into `v` at the next frame/scanline boundary. Without this,
                // a game would have to write `PPUADDR` just to change nametables.
                self.t.set_nametable(self.control.nametable_index());
            }

            // Mask register (`$2001`).
            1 => {
                self.mask.0 = value;
            }

            // Status register (`$2002`). Writes are ignored.
            2 => {}

            // OAM address register (`$2003`).
            3 => {
                self.oam_address = value;
            }

            // OAM data register (`$2004`).
            // Writes one byte to OAM at the current address, then post-increments.
            4 => {
                self.oam[self.oam_address as usize] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }

            // Scroll register (`$2005`).
            // <https://www.nesdev.org/wiki/PPU_registers#PPUSCROLL>
            // Two writes encode a full pixel scroll offset into the loopy `t` register
            // (and the separate fine-X register). The first write sets the horizontal
            // scroll, the second sets the vertical. The PPU uses `t` as the scroll
            // origin: copying it into `v` in full at frame start, and copying just the
            // horizontal bits at the start of each scanline.
            5 => {
                if self.cpu_second_write() {
                    // Second write: vertical scroll into `t`.
                    self.t.set_scroll_y(value);
                } else {
                    // First write: horizontal scroll into `t` and the fine-X register.
                    self.t.set_scroll_x(value);

                    // Fine X lives in its own 3-bit register instead of inside `t` because
                    // the renderer reads it every dot to pick a bit out of the tile shift
                    // registers.
                    //   value[2:0] -> x       fine X,   which pixel column inside the tile (0..7)
                    self.x = value & 0b0111;
                }
            }

            // Address register (`$2006`).
            // <https://www.nesdev.org/wiki/PPU_registers#PPUADDR>
            // Two writes set a 14-bit VRAM address in `t`, which then gets latched into
            // `v` on the second write. Every subsequent `PPUDATA` read or write uses `v`
            // as the address and advances it by the increment configured in `PPUCTRL`.
            6 => {
                if self.cpu_second_write() {
                    // Second write: low byte goes into t[7:0], then copy `t` into `v`.
                    self.t.set_address_low(value);

                    // We have the full VRAM address now, so latch it so PPUDATA takes
                    // effect immediately.
                    self.v = self.t;
                } else {
                    // First write: high 6 bits go into t[13:8], and t[14] is forced clear.
                    self.t.set_address_high(value);
                }
            }

            // Data register (`$2007`).
            // <https://www.nesdev.org/wiki/PPU_registers#PPUDATA>
            // Writes the byte to VRAM at the address in `v`, then advances `v` by 1 (one
            // tile across) or 32 (one tile down), depending on `PPUCTRL` bit 2. `v` gets
            // masked to 14 bits because VRAM is 14-bit, even though `v` itself is 15
            // bits wide.
            7 => {
                let address = self.v.bus_address();

                if address >= 0x3F00 {
                    self.palette_write(address, value);
                } else {
                    bus.ppu_write(address, value);
                }

                self.v.advance(self.control.vram_increment());
            }

            // Callers mask to 3 bits (`& 7`), so this is unreachable in practice.
            _ => {}
        }
    }
}

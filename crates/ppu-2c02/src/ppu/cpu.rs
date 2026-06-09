// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{Ppu2C02, PpuReadWrite};

impl Ppu2C02 {
    /// Advances the write latch and returns whether this is the second write.
    ///
    /// The latch starts `false` (first write pending) and toggles on each call.
    /// Returns `true` only on the second call, then resets to `false` for the next pair.
    /// Reading PPUSTATUS (`$2002`) also resets it to `false` at any time.
    fn cpu_second_write(&mut self) -> bool {
        // PPUSCROLL (`$2005`) and PPUADDR (`$2006`) each require two successive CPU writes
        // to set a 15-bit quantity. Games read PPUSTATUS at the top of vblank to reset
        // this latch, ensuring the first write of each pair lands in the correct half.
        let result = self.w;
        self.w = !self.w;
        result
    }

    /// Reads a CPU-facing PPU register without any side effects.
    ///
    /// Intended for debuggers and memory viewers that need to inspect register state
    /// without disturbing it. `address` is the register index (0–7), corresponding
    /// to the low three bits of the CPU address range `$2000`–`$3FFF`.
    #[must_use]
    pub fn cpu_peek(&self, address: u8) -> u8 {
        match address {
            // Status register (`$2002`).
            // Returns the same merged value cpu_read would, but does NOT clear the vblank
            // flag and does NOT reset w, the caller is observing state, not consuming it.
            2 => (self.status.0 & 0b1110_0000) | (self.io_latch & 0b0001_1111),

            // OAM data register (`$2004`). No side effects on read anyway.
            4 => self.oam[self.oam_address as usize],

            // Data register (`$2007`).
            7 => {
                let vram_address = self.v & 0b0011_1111_1111_1111;

                if vram_address >= 0x3F00 {
                    self.palette_read(vram_address)
                } else {
                    // Non-palette: return data_read_buffer, that is the value a real cpu_read
                    // would return (the buffer is not advanced, so this is exact).
                    self.data_read_buffer
                }
            }

            _ => self.io_latch,
        }
    }

    /// Reads a CPU-facing PPU register, applying all hardware side effects.
    ///
    /// `address` is the register index (0–7). Most registers are write-only; reading them
    /// returns `io_latch` (the last byte on the CPU data bus), unchanged. The exceptions
    /// are PPUSTATUS (`2`), OAMDATA (`4`), and PPUDATA (`7`): each drives some or all bits
    /// of the result, merges undriven bits from `io_latch`, and then stores the full byte
    /// back into `io_latch` before returning.
    #[expect(clippy::match_same_arms)]
    pub fn cpu_read<B: PpuReadWrite>(&mut self, address: u8, bus: &mut B) -> u8 {
        let value = match address {
            // Control register (`$2000`). Write-only; returns io_latch.
            0 => None,

            // Mask register (`$2001`). Write-only; returns io_latch.
            1 => None,

            // Status register (`$2002`).
            // https://www.nesdev.org/wiki/PPU_registers#PPUSTATUS
            2 => {
                // The PPU drives only bits [7:5]; bits [4:0] are not driven and return
                // whatever io_latch holds from the previous bus access.
                // whatever io_latch holds from the previous bus access.
                let value = (self.status.0 & 0b1110_0000) | (self.io_latch & 0b0001_1111);

                // Bit 7 (vblank flag) is cleared immediately after being captured.
                // Polling this bit is the standard way to detect vblank entry; clearing
                // it ensures software only sees it once per frame.
                self.status.0 &= !0b1000_0000;

                // The w latch is reset to false (first-write state). Games read PPUSTATUS
                // before the first PPUSCROLL or PPUADDR write to guarantee they are
                // starting a fresh two-write sequence rather than completing a stale one.
                self.w = false;

                Some(value)
            }

            // OAM address register (`$2003`). Write-only; returns io_latch.
            3 => None,

            // OAM data register (`$2004`).
            // https://www.nesdev.org/wiki/PPU_registers#OAMDATA
            // Returns the byte at the current OAM address without advancing it.
            // Only writes post-increment oam_address; reads leave it unchanged.
            4 => Some(self.oam[self.oam_address as usize]),

            // Scroll register (`$2005`). Write-only; returns io_latch.
            5 => None,

            // Address register (`$2006`). Write-only; returns io_latch.
            6 => None,

            // Data register (`$2007`).
            // https://www.nesdev.org/wiki/PPU_registers#PPUDATA
            7 => {
                let vram_address = self.v & 0b0011_1111_1111_1111;

                let value = if vram_address >= 0x3F00 {
                    // Palette reads are immediate: palette RAM is internal to the PPU.
                    // The buffer is still updated with the nametable byte at the mirrored
                    // address (`v & $2FFF`) as a side effect, because the PPU bus observes
                    // that address even during a palette access.
                    self.data_read_buffer = bus.ppu_read(vram_address & 0x2FFF);
                    self.palette_read(vram_address)
                } else {
                    // The PPU's address and data buses are not directly accessible to the CPU,
                    // so non-palette reads are delayed by one access through a read-ahead buffer:
                    // the CPU receives the stale buffer contents while the buffer refills from VRAM.
                    let stale = self.data_read_buffer;
                    self.data_read_buffer = bus.ppu_read(vram_address);
                    stale
                };

                self.v = self.v.wrapping_add(self.control.vram_increment());

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
    /// `address` is the register index (0–7), corresponding to the low three bits
    /// of the CPU address range `$2000`–`$3FFF`. Callers are expected to have already
    /// masked the address to three bits.
    #[expect(clippy::match_same_arms)]
    pub fn cpu_write<B: PpuReadWrite>(&mut self, address: u8, value: u8, bus: &mut B) {
        // Store the value in the IO latch.
        self.io_latch = value;

        match address {
            // Control register (`$2000`).
            0 => {
                self.control.0 = value;

                // https://www.nesdev.org/wiki/PPU_scrolling#$2000_(PPUCTRL)_write
                // The nametable select bits [1:0] are mirrored into t[11:10] (nt_x, nt_y).
                // This ensures the correct base nametable is already encoded in t before
                // the PPU copies t → v at the start of each frame, so the game doesn't
                // need a separate PPUADDR write just to change which nametable is active.
                self.t = (self.t & !0b1100_0000_0000) | (self.control.nametable_index() << 10);
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
            // https://www.nesdev.org/wiki/PPU_registers#PPUSCROLL
            // Two-write protocol: the first write sets the horizontal scroll position and
            // the second sets the vertical. Together they encode a full pixel offset into
            // the Loopy t register. The PPU uses t as the scroll origin when it copies
            // t → v at the start of each frame (vertical bits) and each scanline (horizontal bits).
            5 => {
                if self.cpu_second_write() {
                    // https://www.nesdev.org/wiki/PPU_scrolling#$2005_(PPUSCROLL)_second_write_(w_is_1)
                    // Second write: encode vertical scroll into t.
                    self.t = (self.t & !0b0111_0011_1110_0000)
                        //   value[7:3] → t[9:5]    coarse Y  — which tile row (0–29)
                        | ((u16::from(value) & 0b1111_1000) << 2)
                        //   value[2:0] → t[14:12]  fine Y    — which pixel row within the tile (0–7)
                        | ((u16::from(value) & 0b0000_0111) << 12);
                } else {
                    // https://www.nesdev.org/wiki/PPU_scrolling#$2005_(PPUSCROLL)_first_write_(w_is_0)
                    // First write: encode horizontal scroll into t and the fine-X register.
                    //   value[7:3] → t[4:0]  coarse X — which tile column (0–31)
                    self.t = (self.t & !0b0001_1111) | (u16::from(value) >> 3);

                    // Fine X lives in its own 3-bit register rather than in t because the PPU
                    // reads it separately on every dot to select a bit from the tile shift registers.
                    //   value[2:0] → x       fine X   — which pixel column within the tile (0–7)
                    self.x = value & 0b0111;
                }
            }

            // Address register (`$2006`).
            // https://www.nesdev.org/wiki/PPU_registers#PPUADDR_-_VRAM_address_($2006_write)
            // Two-write protocol: together the two writes set a 14-bit VRAM address in t,
            // which is then latched into v. All subsequent PPUDATA reads and writes use v
            // as the address and advance it by the increment configured in PPUCTRL.
            6 => {
                if self.cpu_second_write() {
                    // https://www.nesdev.org/wiki/PPU_scrolling#$2006_(PPUADDR)_second_write_(w_is_1)
                    // Second write: low byte → t[7:0], then copy t into v.
                    self.t = (self.t & 0b0111_1111_0000_0000) | u16::from(value);

                    // The complete VRAM address is now known; latch it so PPUDATA takes effect immediately.
                    self.v = self.t;
                } else {
                    // https://www.nesdev.org/wiki/PPU_scrolling#$2006_(PPUADDR)_first_write_(w_is_0)
                    // First write: high 6 bits → t[13:8], t[14] is always cleared.
                    // VRAM is 14-bit (0x0000–0x3FFF), so the top 2 bits of the byte are discarded.
                    self.t =
                        (self.t & 0b0000_0000_1111_1111) | ((u16::from(value) & 0b0011_1111) << 8);
                }
            }

            // Data register (`$2007`).
            // https://www.nesdev.org/wiki/PPU_registers#PPUDATA_-_VRAM_data_($2007_read/write)
            // Writes the byte to VRAM at the address in v, then advances v by 1 (moving
            // across a row) or 32 (moving down a column), as configured by PPUCTRL bit 2.
            // v is masked to 14 bits because VRAM is 14-bit even though v is 15 bits wide.
            7 => {
                let vram_address = self.v & 0b0011_1111_1111_1111;

                if vram_address >= 0x3F00 {
                    self.palette_write(vram_address, value);
                } else {
                    bus.ppu_write(vram_address, value);
                }

                self.v = self.v.wrapping_add(self.control.vram_increment());
            }

            // Callers mask to 3 bits (`& 7`), so this is unreachable in practice.
            _ => {}
        }
    }
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_ppu_2c02::{PpuPeek, PpuReadWrite};

use crate::pak::Pak;

/// The PPU's view of the NES address bus — shared (peek) half.
///
/// Routes side-effect-free PPU memory accesses to the correct physical hardware. CHR pattern
/// fetches (`$0000–$1FFF`) go to the pak's CHR-ROM or CHR-RAM, and nametable accesses
/// (`$2000–$3EFF`) go to the pak, which applies its mirroring arrangement to route the
/// access into the console's CIRAM or its own pak-side nametable RAM.
pub struct SystemNesPpuPeek<'s> {
    pub(super) ciram: &'s [u8; 2048],
    pub(super) pak: Option<&'s Pak>,
}

/// The PPU's view of the NES address bus — mutable (read/write) half.
///
/// Routes the 2C02's memory accesses to the correct physical hardware. CHR pattern fetches
/// (`$0000–$1FFF`) go through the mapper's `read_chr` so MMC2/MMC4-style bank latches fire
/// at the correct PPU cycle, and nametable accesses (`$2000–$3EFF`) go to the pak, which
/// applies its mirroring arrangement to route the access into the console's CIRAM or its
/// own pak-side nametable RAM.
pub struct SystemNesPpuReadWrite<'s> {
    pub(super) ciram: &'s mut [u8; 2048],
    pub(super) pak: Option<&'s mut Pak>,
}

impl PpuPeek for SystemNesPpuPeek<'_> {
    fn ppu_peek(&self, address: u16) -> u8 {
        match address {
            // Pattern tables ($0000–$1FFF): 8 KB of CHR data supplied by the pak.
            // The low 4 KB ($0000–$0FFF) is the sprite pattern table and the high 4 KB
            // ($1000–$1FFF) is the background pattern table, though PPUCTRL bit 3/4 can
            // swap which table each uses. Returns 0 when no pak is present.
            0x0000..=0x1FFF => self.pak.as_ref().map_or(0, |pak| pak.peek_chr(address)),

            // Nametable space ($2000–$3EFF).
            // $2000–$2FFF are the four logical nametables; $3000–$3EFF mirrors $2000–$2EFF.
            // The mapper resolves mirroring and routes to CIRAM or pak-side nametable RAM.
            0x2000..=0x3EFF => {
                self.pak.as_ref().map_or(0, |pak| pak.nametable_peek(self.ciram, address))
            }

            _ => 0,
        }
    }
}

impl PpuReadWrite for SystemNesPpuReadWrite<'_> {
    fn ppu_read(&mut self, address: u16) -> u8 {
        match address {
            // Some mappers (e.g. MMC2, MMC4) update their CHR bank latch on pattern fetches,
            // so the rendering pipeline's tile and attribute reads must go through read_chr
            // (which takes `&mut self` on the mapper) rather than peek_chr. This ensures
            // mid-frame bank switches fire at the correct PPU cycle.
            0x0000..=0x1FFF => self.pak.as_mut().map_or(0, |pak| pak.read_chr(address)),

            // Nametable reads via the mapper's `&mut self` path; reserved for future mappers
            // (MMC5 EXNT) that latch on nametable fetches. NROM and similar delegate to peek.
            0x2000..=0x3EFF => {
                self.pak.as_mut().map_or(0, |pak| pak.nametable_read(self.ciram, address))
            }

            _ => 0,
        }
    }

    #[expect(clippy::match_same_arms)]
    fn ppu_write(&mut self, address: u16, value: u8) {
        match address {
            // CHR-ROM is read-only; PPU writes to this range are silently discarded.

            // TODO: CHR-RAM, when the iNES chr_banks field is 0 the pak supplies
            //  8 KB of writable RAM here instead of ROM, and writes should be forwarded
            //  to the mapper.
            0x0000..=0x1FFF => {}

            // Nametable writes are routed by the mapper into either CIRAM or pak-side
            // nametable RAM, applying the current mirroring arrangement.
            0x2000..=0x3FFF => {
                if let Some(pak) = self.pak.as_mut() {
                    pak.nametable_write(self.ciram, address, value);
                }
            }

            _ => {}
        }
    }
}

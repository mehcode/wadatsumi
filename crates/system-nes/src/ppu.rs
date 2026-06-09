// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_ppu_2c02::{PpuPeek, PpuReadWrite};

use crate::pak::{Mirroring, Pak};

/// The PPU's view of the NES address bus — shared (peek) half.
///
/// Routes side-effect-free PPU memory accesses to the correct physical hardware.
pub struct SystemNesPpuPeek<'s> {
    pub(super) ciram: &'s [u8; 2048],
    pub(super) pak: Option<&'s Pak>,
}

/// The PPU's view of the NES address bus — mutable (read/write) half.
///
/// Routes the 2C02's memory accesses to the correct physical hardware: CHR pattern
/// fetches (`$0000–$1FFF`) go to the pak's CHR-ROM or CHR-RAM, and nametable accesses
/// (`$2000–$3EFF`) go to internal CIRAM or pak VRAM, with mirroring applied according
/// to the pak's wiring.
pub struct SystemNesPpuReadWrite<'s> {
    pub(super) ciram: &'s mut [u8; 2048],
    pub(super) pak: Option<&'s mut Pak>,
}

/// A physical PPU nametable address, resolved from a logical PPU bus address through
/// the pak's mirroring arrangement.
///
/// The NES contains 2 KB of internal CIRAM (Character Internal RAM), enough to back
/// exactly two of the four logical 1 KB nametables. The pak's mirroring mode
/// controls which logical nametables fold onto which physical banks. Four-screen mode
/// is the exception: it requires a full 4 KB of distinct nametable storage, so the
/// upper two logical nametables (`$2800–$2FFF`) overflow into a 2 KB VRAM chip on the
/// pak PCB, represented here as [`NametableAddress::Pak`].
enum NametableAddress {
    /// An address into the NES's internal 2 KB CIRAM chip (`0x0000–0x07FF`).
    ///
    /// Used by horizontal, vertical, and single-screen mirroring. All retail NES
    /// hardware has this chip; it requires no pak support.
    Ciram(u16),

    /// An address into the pak's own 2 KB VRAM chip (`0x0000–0x07FF`).
    ///
    /// Only produced in four-screen mirroring mode, where logical nametables C
    /// (`$2800`) and D (`$2C00`) cannot be backed by the internal 2 KB and instead
    /// map to a dedicated VRAM chip on the pak PCB (e.g. Gauntlet, Rad Racer II).
    Pak(u16),
}

impl SystemNesPpuPeek<'_> {
    /// Resolves a logical PPU nametable address (`$2000–$3EFF`) to its physical
    /// [`NametableAddress`] by applying the pak's mirroring arrangement.
    ///
    /// The four logical 1 KB nametables fold onto two physical 1 KB banks of internal
    /// CIRAM for horizontal and vertical mirroring, or split between internal CIRAM and
    /// pak VRAM in four-screen mode. This is the single point where all mirroring
    /// logic lives; `nametable_read` and `nametable_write` dispatch on the result.
    #[inline]
    fn nametable_address(&self, address: u16) -> NametableAddress {
        // Physical layout in CIRAM:
        //   bank 0 → $0000–$03FF
        //   bank 1 → $0400–$07FF

        // Bits [11:10] of the address (relative to $2000) identify the logical nametable:
        //   00 → A ($2000)
        //   01 → B ($2400)
        //   10 → C ($2800)
        //   11 → D ($2C00)

        // Strip the $3000–$3EFF mirror before any bit work. Bit 12 is the only difference
        // between the primary range ($2000–$2FFF) and its mirror ($3000–$3EFF); masking to
        // 12 bits folds both down to a common $0000–$0FFF offset without touching
        // the nametable-select bits [11:10] or the within-table offset [9:0].
        let offset = address & 0x0FFF;

        match self.pak.as_ref().map(|p| &p.mirroring) {
            None | Some(Mirroring::Horizontal) => {
                // A ($2000) and C ($2800) share physical bank 0.
                // B ($2400) and D ($2C00) share physical bank 1.

                // Bit 10 of the offset distinguishes A/C (0) from B/D (1) and is used
                // directly as the bank selector. Bit 11, which would distinguish the A/B
                // pair from the C/D pair within the same bank, is discarded by the mask.
                NametableAddress::Ciram((offset & 0x0400) | (offset & 0x03FF))
            }

            Some(Mirroring::Vertical) => {
                // A ($2000) and B ($2400) share physical bank 0.
                // C ($2800) and D ($2C00) share physical bank 1.

                // Bit 11 of the offset distinguishes A/B (0) from C/D (1). It is shifted
                // right by one to produce a bank select at bit 10 (0 → bank 0, 1 → bank 1).
                // Bit 10, which would distinguish A from B or C from D within the same bank,
                // is the mirror bit and is discarded by the mask.
                NametableAddress::Ciram(((offset & 0x0800) >> 1) | (offset & 0x03FF))
            }

            Some(Mirroring::FourScreen) => {
                // All four nametables are distinct: A and B use the internal CIRAM, while
                // C and D use the pak's own VRAM chip.

                if offset < 0x0800 {
                    // A ($2000) and B ($2400) map directly into internal CIRAM (offset $000–$7FF
                    // already addresses the full 2 KB without any remapping).
                    NametableAddress::Ciram(offset)
                } else {
                    // C ($2800) and D ($2C00) fold into the pak's 2 KB at offset $000–$7FF
                    // by stripping bit 11, which encodes the C/D distinction relative to A/B.
                    NametableAddress::Pak(offset & 0x07FF)
                }
            }
        }
    }

    /// Reads one byte from the nametable at the given PPU address.
    #[inline]
    fn nametable_read(&self, address: u16) -> u8 {
        match self.nametable_address(address) {
            NametableAddress::Ciram(address) => self.ciram[address as usize],

            NametableAddress::Pak(_) => {
                // TODO: read from pak.vram once the field exists on Pak
                0
            }
        }
    }
}

impl SystemNesPpuReadWrite<'_> {
    /// Writes one byte to the nametable at the given PPU address.
    #[inline]
    fn nametable_write(&mut self, address: u16, value: u8) {
        let peek = SystemNesPpuPeek { ciram: self.ciram, pak: self.pak.as_deref() };

        match peek.nametable_address(address) {
            NametableAddress::Ciram(address) => {
                self.ciram[address as usize] = value;
            }

            NametableAddress::Pak(_) => {
                // TODO: write to pak.vram once the field exists on Pak
            }
        }
    }
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
            // The physical VRAM address is determined by the pak's mirroring arrangement.
            0x2000..=0x3EFF => self.nametable_read(address),

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

            // Nametable reads and their mirrors have no read side-effects.
            _ => SystemNesPpuPeek { ciram: self.ciram, pak: self.pak.as_deref() }.ppu_peek(address),
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        match address {
            // CHR-ROM is read-only; PPU writes to this range are silently discarded.

            // TODO: CHR-RAM, when the iNES chr_banks field is 0 the pak supplies
            //  8 KB of writable RAM here instead of ROM, and writes should be forwarded
            //  to the mapper.
            0x0000..=0x1FFF => {}

            // Nametable writes are stored in the physical VRAM address selected by the
            // pak's mirroring mode (see nametable_address).
            0x2000..=0x3FFF => {
                self.nametable_write(address, value);
            }

            _ => {}
        }
    }
}

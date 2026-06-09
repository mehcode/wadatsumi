// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// How the pak wires its four logical nametables onto physical RAM.
///
/// The console provides 2 KB of internal CIRAM, enough for two nametables. The mapper
/// chooses which logical nametable (A/B/C/D) maps onto which physical CIRAM bank,
/// optionally extending into pak-side nametable RAM for four-screen boards.
///
/// On many mappers (MMC1, MMC3, FME-7) this is mutable at runtime via register writes,
/// which is why it lives on the mapper rather than on the [`Pak`][crate::pak::Pak].
#[derive(Clone, Copy)]
pub enum Mirroring {
    /// Nametables A/C share CIRAM bank 0; B/D share CIRAM bank 1. Scrolling wraps left-right.
    Horizontal,

    /// Nametables A/B share CIRAM bank 0; C/D share CIRAM bank 1. Scrolling wraps top-bottom.
    Vertical,

    /// All four nametables map to distinct memory: A/B in CIRAM, C/D in pak-side
    /// nametable RAM (requires `nt_ram_size() >= 2048`).
    FourScreen,
}

/// A nametable address (`$2000–$3EFF`) resolved through a [`Mirroring`] arrangement to
/// the physical memory that backs it.
///
/// Produced by [`Mirroring::nametable_address`] and consumed by the default
/// [`Mapper::nametable_peek`] / [`Mapper::nametable_write`] impls. Public so debug
/// tooling (memory viewers, disassemblers) can ask "where does PPU `$2A00` actually
/// live?" without reimplementing the routing.
pub enum NametableAddress {
    /// An index into the console's internal 2 KB CIRAM (always `0..2048`).
    Ciram(usize),

    /// An index into the pak's pak-side nametable RAM (range `0..nt_ram_size()` for the
    /// active mapper). Only produced when the mirroring arrangement routes the access
    /// off-console, currently only [`Mirroring::FourScreen`] for the upper two
    /// logical nametables.
    PakNtRam(usize),
}

impl Mirroring {
    /// The default size of pak-side nametable RAM implied by this mirroring arrangement.
    ///
    /// [`FourScreen`] paks ship with a 2 KB chip on the PCB to back the upper two
    /// logical nametables that CIRAM cannot hold; [`Horizontal`] and [`Vertical`] need
    /// no pak-side nametable RAM at all.
    ///
    /// This is the default that [`Mapper::nt_ram_size`] returns; mappers that need
    /// pak-side nametable RAM independent of mirroring (e.g. MMC5 `ExRAM` provides 1 KB
    /// regardless) override `nt_ram_size` directly.
    ///
    /// [`Horizontal`]: Mirroring::Horizontal
    /// [`Vertical`]: Mirroring::Vertical
    /// [`FourScreen`]: Mirroring::FourScreen
    /// [`Mapper::nt_ram_size`]: crate::pak::mapper::Mapper::nt_ram_size
    #[must_use]
    pub const fn default_nt_ram_size(self) -> usize {
        match self {
            Mirroring::FourScreen => 2048,
            Mirroring::Horizontal | Mirroring::Vertical => 0,
        }
    }

    /// Resolves a logical PPU nametable address (`$2000–$3EFF`) to a [`NametableAddress`]
    /// by applying this mirroring arrangement.
    ///
    /// The four logical 1 KB nametables — A (`$2000`), B (`$2400`), C (`$2800`),
    /// D (`$2C00`) — fold onto two physical 1 KB banks of CIRAM for [`Horizontal`] and
    /// [`Vertical`] mirroring, or split between CIRAM and pak-side nametable RAM in
    /// [`FourScreen`] mode.
    ///
    /// [`Horizontal`]: Mirroring::Horizontal
    /// [`Vertical`]: Mirroring::Vertical
    /// [`FourScreen`]: Mirroring::FourScreen
    #[must_use]
    pub const fn nametable_address(self, address: u16) -> NametableAddress {
        // Strip the $3000–$3EFF mirror before any bit work. Bit 12 is the only difference
        // between the primary range ($2000–$2FFF) and its mirror ($3000–$3EFF); masking to
        // 12 bits folds both down to a common $0000–$0FFF offset without touching the
        // nametable-select bits [11:10] or the within-table offset [9:0].
        let offset = (address & 0x0FFF) as usize;

        match self {
            Mirroring::Horizontal => {
                // A ($2000) and C ($2800) share CIRAM bank 0.
                // B ($2400) and D ($2C00) share CIRAM bank 1.
                //
                // Bit 10 distinguishes A/C (0) from B/D (1) and is used directly as the bank
                // selector. Bit 11 (which would distinguish the A/B pair from the C/D pair
                // within the same bank) is discarded by the mask.
                NametableAddress::Ciram((offset & 0x0400) | (offset & 0x03FF))
            }

            Mirroring::Vertical => {
                // A ($2000) and B ($2400) share CIRAM bank 0.
                // C ($2800) and D ($2C00) share CIRAM bank 1.
                //
                // Bit 11 distinguishes A/B (0) from C/D (1) and is shifted right by one to
                // produce the bank select at bit 10. Bit 10 (mirror bit) is discarded.
                NametableAddress::Ciram(((offset & 0x0800) >> 1) | (offset & 0x03FF))
            }

            Mirroring::FourScreen => {
                // All four nametables are distinct. A and B map into CIRAM directly; C and D
                // fold into the pak's 2 KB pak_nt_ram by stripping bit 11 (which encoded the
                // C/D distinction relative to A/B).
                if offset < 0x0800 {
                    NametableAddress::Ciram(offset)
                } else {
                    NametableAddress::PakNtRam(offset & 0x07FF)
                }
            }
        }
    }
}

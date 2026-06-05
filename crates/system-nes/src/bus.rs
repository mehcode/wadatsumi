// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_cpu_2a03::Bus;

use crate::pak::Pak;

/// Owns all system components except the CPU and routes memory-mapped I/O
/// across the NES address space.
pub struct SystemNesBus {
    pub(super) pak: Option<Pak>,
    wram: Box<[u8; 2048]>,
}

impl SystemNesBus {
    pub(crate) fn new() -> Self {
        Self { pak: None, wram: Box::new([0u8; 2048]) }
    }
}

impl Bus for SystemNesBus {
    #[allow(clippy::match_same_arms)]
    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.wram[(address & 0x7ff) as usize],

            0x2000..=0x3fff => {
                // TODO: PPU registers
                0
            }

            0x4000..=0x401f => {
                // TODO: APU registers
                0
            }

            0x4020..=0x5fff => {
                // TODO: Pak expansion area (?)
                0
            }

            0x6000..=0x7fff => self.pak.as_ref().map_or(0, |pak| pak.read_sram(address)),

            0x8000..=0xffff => self.pak.as_ref().map_or(0, |pak| pak.read_prg(address)),
        }
    }

    #[allow(clippy::match_same_arms)]
    fn read(&mut self, address: u16) -> u8 {
        // Once PPU/APU registers are implemented, handle their side effects here
        // before falling through to peek.
        self.peek(address)
    }

    #[allow(clippy::match_same_arms)]
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                self.wram[(address & 0x7ff) as usize] = value;
            }

            0x2000..=0x3fff => {
                // TODO: PPU registers
            }

            0x4000..=0x401f => {
                // TODO: APU registers
            }

            0x4020..=0x5fff => {
                // TODO: Pak expansion area (?)
            }

            0x6000..=0x7fff => {
                if let Some(pak) = self.pak.as_mut() {
                    pak.write_sram(address, value);
                }
            }

            0x8000..=0xffff => {
                // TODO: Pak PRG-RAM (?)
            }
        }
    }
}

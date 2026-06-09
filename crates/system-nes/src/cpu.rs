// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_cpu_2a03::{CpuPeek, CpuReadWrite};
use wadatsumi_ppu_2c02::Ppu2C02;

use crate::pak::Pak;
use crate::ppu::SystemNesPpuReadWrite;

pub struct SystemNesCpuPeek<'s> {
    pub(super) wram: &'s [u8; 2048],
    pub(super) ciram: &'s [u8; 2048],
    pub(super) ppu: &'s Ppu2C02,
    pub(super) pak: Option<&'s Pak>,
}

impl CpuPeek for SystemNesCpuPeek<'_> {
    #[allow(clippy::match_same_arms)]
    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.wram[(address & 0x7ff) as usize],

            0x2000..=0x3fff => self.ppu.cpu_peek((address & 7) as u8),

            0x4000..=0x401f => {
                // TODO: APU registers
                0
            }

            0x4020..=0x5fff => {
                // TODO: Pak expansion area (?)
                0
            }

            0x6000..=0x7fff => self.pak.as_ref().map_or(0, |pak| pak.read_sram(address)),

            0x8000..=0xffff => self.pak.as_ref().map_or(0, |pak| pak.peek_prg(address)),
        }
    }
}

pub struct SystemNesCpuReadWrite<'s> {
    pub(super) wram: &'s mut [u8; 2048],
    pub(super) ciram: &'s mut [u8; 2048],
    pub(super) ppu: &'s mut Ppu2C02,
    pub(super) pak: Option<&'s mut Pak>,
}

impl CpuReadWrite for SystemNesCpuReadWrite<'_> {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x2000..=0x3fff => self.ppu.cpu_read(
                (address & 7) as u8,
                &mut SystemNesPpuReadWrite { pak: self.pak.as_deref_mut(), ciram: self.ciram },
            ),

            // TODO: 0x4016, 0x4017
            _ => SystemNesCpuPeek {
                wram: self.wram,
                ciram: self.ciram,
                ppu: self.ppu,
                pak: self.pak.as_deref(),
            }
            .peek(address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                self.wram[(address & 0x7ff) as usize] = value;
            }

            0x2000..=0x3fff => {
                self.ppu.cpu_write(
                    (address & 7) as u8,
                    value,
                    &mut SystemNesPpuReadWrite { pak: self.pak.as_deref_mut(), ciram: self.ciram },
                );
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

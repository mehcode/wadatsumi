// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_cpu_2a03::{CpuBus, CpuPeek, CpuReadWrite};
use wadatsumi_ppu_2c02::Ppu2C02;

use crate::pak::Pak;
use crate::ppu::SystemNesPpuReadWrite;

pub struct SystemNesCpuPeek<'s> {
    pub(super) wram: &'s [u8; 2048],
    pub(super) ciram: &'s [u8; 2048],
    pub(super) ppu: &'s Ppu2C02,
    pub(super) pak: Option<&'s Pak>,
}

impl SystemNesCpuPeek<'_> {
    #[inline(never)]
    fn ppu_peek(&self, address: u16) -> u8 {
        self.ppu.cpu_peek((address & 7) as u8)
    }

    fn pak_read_sram(&self, address: u16) -> u8 {
        self.pak.as_ref().map_or(0, |pak| pak.read_sram(address))
    }

    fn pak_read_prg(&self, address: u16) -> u8 {
        self.pak.as_ref().map_or(0, |pak| pak.peek_prg(address))
    }
}

impl CpuPeek for SystemNesCpuPeek<'_> {
    #[allow(clippy::match_same_arms)]
    #[inline(always)]
    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => self.wram[(address & 0x7ff) as usize],

            0x2000..=0x3fff => self.ppu_peek(address),

            0x4000..=0x401f => {
                // TODO: APU registers
                0
            }

            0x4020..=0x5fff => {
                // TODO: Pak expansion area (?)
                0
            }

            0x6000..=0x7fff => self.pak_read_sram(address),

            0x8000..=0xffff => self.pak_read_prg(address),
        }
    }
}

pub struct SystemNesCpuBus<'s> {
    pub(super) wram: &'s mut [u8; 2048],
    pub(super) ciram: &'s mut [u8; 2048],
    pub(super) ppu: &'s mut Ppu2C02,
    pub(super) pak: Option<&'s mut Pak>,
}

impl SystemNesCpuBus<'_> {
    #[inline(never)]
    fn ppu_read(&mut self, address: u16) -> u8 {
        self.ppu.cpu_read(
            (address & 7) as u8,
            &mut SystemNesPpuReadWrite { pak: self.pak.as_deref_mut(), ciram: self.ciram },
        )
    }

    #[inline(never)]
    fn ppu_write(&mut self, address: u16, value: u8) {
        self.ppu.cpu_write(
            (address & 7) as u8,
            value,
            &mut SystemNesPpuReadWrite { pak: self.pak.as_deref_mut(), ciram: self.ciram },
        );
    }

    #[inline(never)]
    fn pak_write_sram(&mut self, address: u16, value: u8) {
        if let Some(pak) = self.pak.as_mut() {
            pak.write_sram(address, value);
        }
    }
}

impl CpuReadWrite for SystemNesCpuBus<'_> {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x2000..=0x3fff => self.ppu_read(address),

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

    #[expect(clippy::match_same_arms)]
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                self.wram[(address & 0x7ff) as usize] = value;
            }

            0x2000..=0x3fff => {
                self.ppu_write(address, value);
            }

            0x4000..=0x401f => {
                // TODO: APU registers
            }

            0x4020..=0x5fff => {
                // TODO: Pak expansion area (?)
            }

            0x6000..=0x7fff => {
                self.pak_write_sram(address, value);
            }

            0x8000..=0xffff => {
                // TODO: Pak PRG-RAM (?)
            }
        }
    }
}

impl CpuBus for SystemNesCpuBus<'_> {
    #[inline(always)]
    fn nmi(&self) -> bool {
        // The PPU is the only NMI source on a stock NES; the line is just
        // `vblank_flag AND PPUCTRL bit 7`, exposed directly by the chip.
        self.ppu.nmi()
    }

    #[inline(always)]
    fn irq(&self) -> bool {
        // Wired-OR of every IRQ source on the CPU's /IRQ pin. Today only the mapper
        // can drive it (and NROM never does); the APU's frame-counter and DMC IRQs
        // get OR'd in here once the APU exists.
        self.pak.as_deref().is_some_and(Pak::irq)
    }

    #[inline(always)]
    fn rdy(&self) -> bool {
        // No DMA controller wired in yet, so the CPU is always free to run. Once the
        // APU exists, OAM DMA (`$4014`) and DMC DMA will pull this low to steal cycles.
        true
    }
}

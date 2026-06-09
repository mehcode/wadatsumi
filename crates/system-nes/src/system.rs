// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_cpu_2a03::{Cpu2A03, CpuPeek};
use wadatsumi_ppu_2c02::Ppu2C02;
use wadatsumi_system::System;

use crate::cpu::{SystemNesCpuPeek, SystemNesCpuReadWrite};
use crate::pak::Pak;
use crate::ppu::SystemNesPpuReadWrite;

/// The top-level NES system, tying together the CPU, WRAM, PPU and APU.
pub struct SystemNes {
    pub cpu: Cpu2A03,
    pub ppu: Ppu2C02,
    pak: Option<Pak>,
    wram: Box<[u8; 2048]>,
    ciram: Box<[u8; 2048]>,
}

impl SystemNes {
    #[must_use]
    fn new() -> Self {
        Self {
            cpu: Cpu2A03::new(),
            ppu: Ppu2C02::new(),
            pak: None,
            wram: Box::new([0u8; 2048]),
            ciram: Box::new([0u8; 2048]),
        }
    }

    /// Open and parse an iNES `.nes` ROM file.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPak`] if the file is missing or malformed,
    /// [`Error::UnsupportedMapper`] if the mapper is not implemented,
    /// or an I/O error if the file cannot be read.
    pub fn open(pak: Vec<u8>) -> crate::Result<Self> {
        let mut system = Self::new();

        system.pak = Some(Pak::open(pak)?);

        system.reset();

        Ok(system)
    }

    /// Reads one byte from `address`, without triggering hardware side-effects.
    pub fn cpu_peek(&self, address: u16) -> u8 {
        SystemNesCpuPeek {
            ciram: &self.ciram,
            wram: &self.wram,
            ppu: &self.ppu,
            pak: self.pak.as_ref(),
        }
        .peek(address)
    }
}

impl System for SystemNes {
    fn tick(&mut self) {
        for _ in 0..3 {
            self.ppu.tick(&mut SystemNesPpuReadWrite {
                pak: self.pak.as_mut(),
                ciram: &mut self.ciram,
            });
        }

        self.cpu.tick(&mut SystemNesCpuReadWrite {
            ciram: &mut self.ciram,
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            pak: self.pak.as_mut(),
        });
    }

    fn reset(&mut self) {
        self.cpu.reset();

        while self.cpu.resetting() {
            self.tick();
        }
    }
}

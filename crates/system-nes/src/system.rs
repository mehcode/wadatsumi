// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_cpu_2a03::Cpu2A03;
use wadatsumi_system::System;

use crate::bus::SystemNesBus;
use crate::pak::Pak;

/// The top-level NES system, tying together the CPU, WRAM, PPU and APU.
pub struct SystemNes {
    pub cpu: Cpu2A03,
    pub bus: SystemNesBus,
}

impl SystemNes {
    #[must_use]
    fn new() -> Self {
        Self { cpu: Cpu2A03::new(), bus: SystemNesBus::new() }
    }

    /// Open and parse an iNES `.nes` ROM file.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPak`] if the file is missing or malformed,
    /// [`Error::UnsupportedMapper`] if the mapper is not implemented,
    /// or an I/O error if the file cannot be read.
    pub fn open(pak: Vec<u8>) -> crate::Result<Self> {
        let mut system = Self::new();

        system.bus.pak = Some(Pak::open(pak)?);

        system.reset();

        Ok(system)
    }
}

impl System for SystemNes {
    fn tick(&mut self) {
        self.cpu.tick(&mut self.bus);

        // TODO: self.ppu.tick() x 3
        // TODO: self.apu.tick()
    }

    fn reset(&mut self) {
        self.cpu.reset();

        while self.cpu.resetting() {
            self.tick();
        }
    }
}

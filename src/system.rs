// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use crate::cpu::Cpu;
use crate::pak::Pak;
use crate::system::bus::SystemBus;

mod bus;

/// The top-level NES system, tying together the CPU, WRAM, PPU and APU.
pub struct System {
    pub cpu: Cpu<SystemBus>,
    pub bus: SystemBus,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl System {
    #[must_use]
    pub fn new() -> Self {
        Self { cpu: Cpu::new(), bus: SystemBus::new() }
    }

    /// Open and parse an iNES `.nes` ROM file.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPak`] if the file is missing or malformed,
    /// [`Error::UnsupportedMapper`] if the mapper is not implemented,
    /// or an I/O error if the file cannot be read.
    pub fn open(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        self.bus.pak = Some(Pak::open(path)?);

        self.cpu.reset(&mut self.bus);

        Ok(())
    }

    /// Advances all system components by one clock cycle.
    ///
    /// # Errors
    /// Returns an error if the CPU encounters an unknown opcode.
    pub fn tick(&mut self) -> crate::Result<()> {
        self.cpu.tick(&mut self.bus)?;

        // TODO: self.ppu.tick() x 3
        // TODO: self.apu.tick()

        Ok(())
    }
}

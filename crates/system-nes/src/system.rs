// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use wadatsumi_cpu_2a03::{Cpu2A03, CpuPeek};
use wadatsumi_ppu_2c02::Ppu2C02;
use wadatsumi_system::System;

use crate::cpu::{SystemNesCpuBus, SystemNesCpuPeek};
use crate::pak::Pak;
use crate::ppu::SystemNesPpuReadWrite;

/// The top-level NES system, tying together the CPU, WRAM, PPU and APU.
pub struct SystemNes {
    /// The 2A03 CPU core. Exposed so frontends and debuggers can read register state
    /// and check `cpu.halted()` to know when the system has jammed.
    pub cpu: Cpu2A03,

    /// The 2C02 PPU core. Exposed so frontends can pull the framebuffer and debuggers
    /// can inspect rendering state.
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
        // https://www.nesdev.org/wiki/PPU_frame_timing#CPU-PPU_Clock_Alignment

        // The NTSC master clock (21.477 MHz) divides by 12 for the CPU (1.79 MHz)
        // and by 4 for the PPU (5.37 MHz), so each CPU cycle overlaps exactly three
        // PPU dots. On real hardware the two domains run independently from the
        // same master clock with a fixed sub-cycle phase relationship; here we
        // interleave one CPU cycle and three PPU dots within a single `tick`, so
        // the order in which they run decides which dot's state the CPU sees on a
        // `$2000`–`$3FFF` access and which dot's edges (vblank, sprite-0 hit,
        // MMC3 A12) are detected on this cycle vs. the next.

        // The chosen order is **one PPU dot, then the CPU cycle, then two more
        // PPU dots**, equivalently, the PPU leads the CPU by one dot per tick.
        // This is a coarse stand-in for the true master-clock alignment, where
        // the PPU's dot edges lead the CPU's φ2 (sample) edge by a fraction of a
        // CPU cycle rather than by a full dot. At cycle granularity, placing one
        // PPU dot before the CPU is the standard approximation: an edge raised
        // on the leading dot is visible to the same-tick CPU access, while edges
        // raised on the trailing two dots only become visible on the *next* tick.

        // Leading PPU dot. Any flag this dot raises, vblank set at (
        // sprite-0 hit, MMC3 IRQ counter clocked by a CHR fetch's A12
        // is observable by the CPU access that follows on this same tick.
        self.ppu
            .tick(&mut SystemNesPpuReadWrite { pak: self.pak.as_mut(), ciram: &mut self.ciram });

        // CPU cycle. Bus reads of PPU-mapped addresses observe the state left
        // by the leading dot above; register writes take effect before the
        // trailing dots run, so PPUCTRL/PPUMASK changes are immediately visible
        // to the rest of this tick's rendering work.
        self.cpu.tick(&mut SystemNesCpuBus {
            ciram: &mut self.ciram,
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            pak: self.pak.as_mut(),
        });

        // Trailing two dots complete the 3:1 PPU-to-CPU ratio. Edges they
        // raise become visible to the *next* tick's CPU cycle, one cycle later
        // than edges from the leading dot.
        for _ in 0..2 {
            self.ppu.tick(&mut SystemNesPpuReadWrite {
                pak: self.pak.as_mut(),
                ciram: &mut self.ciram,
            });
        }
    }

    fn reset(&mut self) {
        self.cpu.reset();

        while self.cpu.resetting() {
            self.tick();
        }
    }
}

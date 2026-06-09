// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Side-effect-free read access to the PPU address bus, intended for debuggers and memory viewers.
pub trait PpuPeek {
    fn ppu_peek(&self, address: u16) -> u8;
}

/// Read/write access to the PPU address bus.
pub trait PpuReadWrite {
    fn ppu_read(&mut self, address: u16) -> u8;

    fn ppu_write(&mut self, address: u16, value: u8);
}

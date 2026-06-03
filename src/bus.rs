// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Abstraction over the address/data bus.
///
/// Allows the CPU to read and write memory-mapped addresses without knowing
/// the underlying hardware topology.
pub trait Bus {
    /// Reads one byte from `address`, without triggering hardware side-effects.
    fn peek(&self, address: u16) -> u8;

    /// Reads one byte from `address`, with hardware side-effects.
    fn read(&mut self, address: u16) -> u8;

    /// Writes `value` to `address`.
    fn write(&mut self, address: u16, value: u8);
}

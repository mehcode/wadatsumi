// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::marker::ConstParamTy;

/// Processor status register (P) flags for the 6502 CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub struct CpuStatus(pub u8);

impl CpuStatus {
    /// Carry.
    /// Set if the last operation produced a carry out of bit 7 (or a borrow into bit 7 for subtraction).
    pub const C: Self = Self(0b0000_0001);

    /// Zero.
    /// Set if the result of the last operation was zero.
    pub const Z: Self = Self(0b0000_0010);

    /// Interrupt disable.
    /// When set, IRQ is disabled and cannot be triggered.
    pub const I: Self = Self(0b0000_0100);

    /// Decimal.
    /// Enables binary-coded decimal (BCD) mode for ADC/SBC.
    /// Ignored in the NES 2A03.
    pub const D: Self = Self(0b0000_1000);

    /// Bit 4 is not stored in the CPU, it has no physical register counterpart.
    /// OR'd into P only when pushing to the stack: set by `PHP`/`BRK`, clear for `IRQ`/`NMI`.
    /// This lets an interrupt handler distinguish software from hardware interrupts by inspecting the stack byte.
    pub const B: u8 = 0b0001_0000;

    /// Bit 5 is unused and hardwired on the physical chip.
    /// Must be OR'd into P whenever it is pushed to the stack or exposed on the bus.
    pub const U: u8 = 0b0010_0000;

    /// Overflow.
    /// Set if the last operation produced a signed overflow.
    pub const V: Self = Self(0b0100_0000);

    /// Negative.
    /// Set if bit 7 of the result is set (result is negative in two's complement).
    pub const N: Self = Self(0b1000_0000);

    /// Constructs a `CpuStatus` from a raw byte, dropping bits that are not stored
    /// in the physical register (`B` and `U`).
    #[must_use]
    pub const fn new(bits: u8) -> Self {
        Self(bits & !Self::B & !Self::U)
    }

    /// Returns `true` if any bit in `flag` is set.
    ///
    /// Callers pass single-bit flag constants (`C`, `Z`, …), for which "any bit set"
    /// and "all bits set" coincide; this is not a general subset test.
    #[must_use]
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    /// Sets or clears `flag` based on `value`.
    pub const fn set(&mut self, flag: Self, value: bool) {
        if value {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }

    /// Sets `flag`.
    pub const fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    /// Clears `flag`.
    pub const fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }

    /// Sets or clears Z based on whether `result` is zero.
    pub const fn update_z(&mut self, result: u8) {
        self.set(Self::Z, result == 0);
    }

    /// Sets or clears N from bit 7 of `result`.
    pub const fn update_n(&mut self, result: u8) {
        self.set(Self::N, result & 0x80 != 0);
    }

    /// Sets or clears Z and N from `result`.
    /// Equivalent to calling [`update_z`] and [`update_n`].
    pub const fn update_zn(&mut self, result: u8) {
        self.update_z(result);
        self.update_n(result);
    }
}

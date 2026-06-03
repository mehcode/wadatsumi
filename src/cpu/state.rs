// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::marker::ConstParamTy;

#[derive(Debug, Clone, Copy, ConstParamTy, PartialEq, Eq)]
pub enum Register {
    A,
    X,
    Y,
    SP,
}

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

    /// Returns `true` if all bits in `flag` are set.
    #[must_use]
    pub const fn contains(&self, flag: Self) -> bool {
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
    pub fn update_z(&mut self, result: u8) {
        self.set(Self::Z, result == 0);
    }

    /// Sets or clears N from bit 7 of `result`.
    pub fn update_n(&mut self, result: u8) {
        self.set(Self::N, result & 0x80 != 0);
    }

    /// Sets or clears Z and N from `result`.
    /// Equivalent to calling [`update_z`] and [`update_n`].
    pub fn update_zn(&mut self, result: u8) {
        self.update_z(result);
        self.update_n(result);
    }
}

/// Snapshot of all 6502 CPU registers.
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    /// Accumulator (A).
    ///
    /// The main register for arithmetic and logic operations.
    /// Unlike the X and Y registers, it has a direct connection to the Arithmetic and Logic Unit (ALU).
    pub a: u8,

    /// X index register.
    pub x: u8,

    /// Y index register.
    pub y: u8,

    /// Program counter (PC).
    ///
    /// This register points the address from which the next instruction
    /// byte (opcode or parameter) will be fetched.
    pub pc: u16,

    /// Stack pointer (SP).
    ///
    /// The NMOS 65xx processors have 256 bytes of stack memory, ranging from `$0100` to `$01FF`.
    /// The S register is a 8-bit offset to the stack page.
    pub sp: u8,

    /// Processor (P) status register.
    pub p: CpuStatus,
}

impl Default for CpuState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl CpuState {
    /// Returns the CPU register state at power-on.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,

            // After the reset sequence the 6502 performs three phantom stack writes,
            // decrementing S from 0xFF to 0xFD.
            sp: 0xFD,

            // I is set by the reset sequence.
            p: CpuStatus::I,
        }
    }

    /// Writes `value` to register `R`.
    #[inline(always)]
    pub fn set<const R: Register>(&mut self, value: u8) {
        match R {
            Register::A => {
                self.a = value;
            }

            Register::X => {
                self.x = value;
            }

            Register::Y => {
                self.y = value;
            }

            Register::SP => {
                self.sp = value;
            }
        }
    }

    /// Reads the value of register `R`.
    #[inline(always)]
    #[must_use]
    pub const fn get<const R: Register>(&self) -> u8 {
        match R {
            Register::A => self.a,
            Register::X => self.x,
            Register::Y => self.y,
            Register::SP => self.sp,
        }
    }

    /// Returns the full 16-bit address of the current stack top: `$0100 | SP`.
    #[inline(always)]
    #[must_use]
    pub const fn stack_address(&self) -> u16 {
        0x0100 | self.sp as u16
    }
}

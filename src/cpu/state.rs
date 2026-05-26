// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::marker::ConstParamTy;

use bitflags::bitflags;

#[derive(Debug, Clone, Copy, ConstParamTy, PartialEq, Eq)]
pub enum Register {
    A,
    X,
    Y,
    SP,
}

bitflags! {
    /// Processor status register (P) flags for the 6502 CPU.
    #[derive(Debug, Clone, Copy)]
    pub struct CpuStatus: u8 {
        /// Carry.
        /// Set if the last operation produced a carry out of bit 7 (or a borrow into bit 7 for subtraction).
        const C = 0b0000_0001;

        /// Zero.
        /// Set if the result of the last operation was zero.
        const Z = 0b0000_0010;

        /// Interrupt disable.
        /// When set, IRQ is disabled and cannot be triggered.
        const I = 0b0000_0100;

        /// Decimal.
        /// Enables binary-coded decimal (BCD) mode for ADC/SBC.
        /// Ignored in the NES 2A03.
        const D = 0b0000_1000;

        /// Break.
        /// Set by the BRK instruction; distinguishes software from hardware interrupts on the stack.
        const B = 0b0001_0000;

        /// Overflow.
        /// Set if the last operation produced a signed overflow.
        const V = 0b0100_0000;

        /// Negative.
        /// Set if bit 7 of the result is set (result is negative in two's complement).
        const N = 0b1000_0000;
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

impl CpuState {
    /// Returns the CPU register state at power-on.
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,

            // After the reset sequence the 6502 performs three phantom stack writes,
            // decrementing S from 0xFF to 0xFD.
            sp: 0xFD,

            // I is set by the reset sequence; B reflects the physical pin state on power-on.
            p: CpuStatus::I | CpuStatus::B,
        }
    }

    /// Writes `value` to register `R`.
    pub const fn set<const R: Register>(&mut self, value: u8) {
        let target = match R {
            Register::A => &mut self.a,
            Register::X => &mut self.x,
            Register::Y => &mut self.y,
            Register::SP => &mut self.sp,
        };

        *target = value;
    }

    /// Reads the value of register `R`.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub const fn get<const R: Register>(&self) -> u8 {
        match R {
            Register::A => self.a,
            Register::X => self.x,
            Register::Y => self.y,
            Register::SP => self.sp,
        }
    }
}

impl CpuStatus {
    /// Bit 5 is unused and hardwired on the physical chip.
    /// Must be OR'd into P whenever it is pushed to the stack or exposed on the bus.
    pub const U: u8 = 0b0010_0000;

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

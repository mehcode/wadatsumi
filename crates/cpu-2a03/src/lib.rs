// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![feature(min_adt_const_params)]

mod addressing;
mod bus;
mod cpu;
mod instruction;
mod interrupt;
mod operation;
mod reset;
mod status;
mod table;

pub use bus::{CpuBus, CpuPeek, CpuReadWrite};
pub use cpu::Cpu2A03;
pub use status::CpuStatus;

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cpu;
mod error;
mod pak;
mod ppu;
mod system;

pub use error::{Error, Result};
pub use system::SystemNes;

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

#![feature(min_adt_const_params)]

mod bus;
mod cpu;
mod error;
mod pak;
mod system;

pub use error::{Error, Result};
pub use system::System;

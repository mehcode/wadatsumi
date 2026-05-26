// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("open")]
    Open(#[from] std::io::Error),

    #[error("invalid pak")]
    InvalidPak,

    #[error("unsupported mapper: {0}")]
    UnsupportedMapper(u8),

    #[error("unknown opcode ${opcode:02X} at ${pc:04X}")]
    UnknownOpcode { opcode: u8, pc: u16 },
}

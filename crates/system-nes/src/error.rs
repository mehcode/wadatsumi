// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{self, Debug, Formatter};

use thiserror::Error;

/// The result type returned by fallible operations in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced while opening or running a [`SystemNes`](crate::SystemNes).
#[derive(Error)]
pub enum Error {
    /// I/O failure while reading the ROM file from disk.
    #[error("open")]
    Open(#[from] std::io::Error),

    /// The supplied bytes are not a valid iNES `.nes` ROM (bad header, truncated, …).
    #[error("invalid pak")]
    InvalidPak,

    /// The ROM's iNES mapper number is not implemented. The unsupported mapper id is
    /// included as the payload.
    #[error("unsupported mapper: {0}")]
    UnsupportedMapper(u8),
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(error) => f.debug_tuple("Open").field(error).finish(),

            Self::InvalidPak => write!(f, "InvalidPak"),

            Self::UnsupportedMapper(mapper) => {
                f.debug_tuple("UnsupportedMapper").field(mapper).finish()
            }
        }
    }
}

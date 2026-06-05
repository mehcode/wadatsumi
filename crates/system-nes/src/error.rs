// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{self, Debug, Formatter};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error)]
pub enum Error {
    #[error("open")]
    Open(#[from] std::io::Error),

    #[error("invalid pak")]
    InvalidPak,

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

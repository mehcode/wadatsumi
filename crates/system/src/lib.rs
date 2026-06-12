// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared frontend-facing interface for every emulated system in wadatsumi.
//!
//! [`System`] is the common trait each console implementation (NES, GB, …) satisfies:
//! frontends construct a concrete system through its own constructor, then drive it
//! exclusively through [`System::tick`] and [`System::reset`] without caring which chip
//! set lives behind it.

mod system;

pub use system::System;

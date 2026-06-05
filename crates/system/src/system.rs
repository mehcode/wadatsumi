// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Common interface implemented by every emulated system.
///
/// A `System` represents a fully loaded, running machine. Construction is
/// system-specific; once built, the frontend drives it exclusively through
/// this trait.
pub trait System {
    /// Advances all system components by one clock cycle.
    fn tick(&mut self);
}

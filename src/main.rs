// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Command-line frontend for the wadatsumi emulator.
//!
//! Parses a path to an iNES `.nes` ROM, opens it as a [`SystemNes`], and drives the
//! system tick-by-tick until the CPU halts.

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use wadatsumi_system::System;
use wadatsumi_system_nes::SystemNes;

#[derive(Parser)]
struct Args {
    pak: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let pak = std::fs::read(&args.pak)?;

    let mut system = SystemNes::open(pak)?;

    while !system.cpu.halted() {
        system.tick();
    }

    Ok(())
}

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use wadatsumi::System;

#[derive(Parser)]
struct Args {
    pak: PathBuf,
}

fn main() -> wadatsumi::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let mut system = System::new();

    system.open(&args.pak)?;

    while !system.cpu.halted() {
        system.tick();
    }

    Ok(())
}

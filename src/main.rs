// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use clap::Parser;
use wadatsumi::System;

#[derive(Parser)]
struct Args {
    pak: PathBuf,
}

fn main() -> wadatsumi::Result<()> {
    let args = Args::parse();

    let mut system = System::new();

    system.open(&args.pak)?;

    loop {
        system.tick()?;
    }
}

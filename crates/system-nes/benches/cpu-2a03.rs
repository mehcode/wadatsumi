// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use divan::counter::CyclesCount;
use wadatsumi_system::System;
use wadatsumi_system_nes::SystemNes;

fn main() {
    divan::main();
}

/// Runs the official-opcode section of the nestest ROM (26,554 CPU cycles).
///
/// This mirrors the setup in tests/nestest.rs, PC forced to $C000 to bypass
/// the PPU init, but without log comparison, so the hot path is pure
/// decode/execute/bus overhead.
#[divan::bench(min_time = 1)]
fn nestest(bencher: divan::Bencher) {
    bencher
        .counter(CyclesCount::new(26_554u64))
        .with_inputs(|| {
            let pak = std::fs::read("tests/nestest/nestest.nes").unwrap();
            let mut system = SystemNes::open(pak).unwrap();
            system.cpu.state.pc = 0xc000;

            system
        })
        .bench_local_values(|mut system| {
            for _ in 0..26_554 {
                system.tick();
            }
        });
}

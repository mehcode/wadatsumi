// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Throughput benchmarks for the 2A03 CPU core, driven through a full [`SystemNes`].
//!
//! Each bench loads a ROM, forces the CPU into the target code path, and times the
//! resulting tick loop with [`divan`], reporting cycles/sec via [`CyclesCount`] so
//! results stay comparable across runs of different lengths.

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
            // Each sample mutates the entire system, so divan rebuilds one per iteration.
            // PC is retargeted to $C000 to skip the interactive front-end and run the same
            // automation entry point that `tests::nestest` covers (see that module's docs).
            let pak = std::fs::read("tests/nestest/nestest.nes").unwrap();
            let mut system = SystemNes::open(pak).unwrap();
            system.cpu.pc = 0xc000;

            system
        })
        .bench_local_values(|mut system| {
            // Same cycle budget as the conformance test: the official-opcode section ends
            // here with the ROM falling into an infinite loop, so a fixed count keeps the
            // sample bounded.
            for _ in 0..26_554 {
                system.tick();
            }
        });
}

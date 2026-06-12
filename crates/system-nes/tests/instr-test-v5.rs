// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Runs each ROM in blargg's `instr_test-v5` suite and reads its pass/fail verdict back
//! out of cartridge SRAM.
//!
//! The suite covers every official 6502 instruction and the common set of unofficial
//! opcodes, checking that flags, cycle counts, and memory effects all line up with what
//! real silicon does. Each sub-ROM is self-contained: it runs on the target, writes a
//! status byte and a human-readable message into the `$6000` SRAM window, then either
//! halts or loops forever depending on the result.
//!
//! The `$6000` result protocol is the de-facto convention shared by every blargg test ROM
//! (the same one used by the APU, PPU, and CPU dummy-read suites). See
//! <https://www.nesdev.org/wiki/Emulator_tests#Blargg.27s_tests> for the full description
//! and the canonical archive.

use std::fs;
use std::path::Path;

use wadatsumi_system::System;
use wadatsumi_system_nes::SystemNes;

datatest_stable::harness! {
    {
        test = instr_test_v5,
        root = "tests/instr-test-v5",
        pattern = r"^.*\.nes$",
    },
}

/// Cycle budget before we call a hang. At the 2A03's ~1.79 MHz this is roughly 56 s of
/// emulated time, comfortably over the slowest sub-test in the suite (well under 30 s).
const MAX_CYCLES: u64 = 100_000_000;

fn instr_test_v5(path: &Path) -> datatest_stable::Result<()> {
    let pak = fs::read(path)?;
    let mut system = SystemNes::open(pak)?;

    let mut initialized = false;

    for _ in 0..MAX_CYCLES {
        system.tick();

        // The ROM's startup routine writes a three-byte magic signature `DE B0 61` to
        // $6001-$6003 once it has copied its result-handling code into RAM and is ready to
        // report. Before that, SRAM is zero-initialised and a naive read of $6000 would
        // see `0x00` — the protocol's "pass" code — and we'd report a false success.
        if !initialized {
            initialized = system.cpu_peek(0x6001) == 0xDE
                && system.cpu_peek(0x6002) == 0xB0
                && system.cpu_peek(0x6003) == 0x61;

            continue;
        }

        // $80 is the "still running" sentinel. Anything else is the final result code:
        // $00 means pass, any other value is a failure.
        let status = system.cpu_peek(0x6000);

        if status == 0x80 {
            continue;
        }

        // Failure path. Alongside the status code the ROM writes a null-terminated ASCII
        // description at $6004 explaining which sub-test failed and how — surface that in
        // the panic message so the report names the specific instruction.
        if status != 0x00 {
            let msg: String = (0x6004..)
                .map(|a| system.cpu_peek(a))
                .take_while(|&b| b != 0)
                .map(|b| b as char)
                .collect();

            return Err(format!("status={status:#04x}\n{msg}").into());
        }

        return Ok(());
    }

    Err(format!("timed out after {MAX_CYCLES} cycles (initialized={initialized})").into())
}

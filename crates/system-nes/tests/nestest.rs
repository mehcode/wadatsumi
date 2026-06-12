// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Runs kevtris's `nestest` conformance ROM against the 2A03 core and compares CPU state,
//! cycle-for-cycle, against the golden log shipped with the ROM.
//!
//! `nestest.nes` walks every official 6502 opcode and addressing mode, including the
//! page-cross, dummy-read, and branch-timing edge cases that distinguish a passing
//! interpreter from a "looks right" one. The companion `nestest.log` records the chip's
//! state at the instruction-fetch cycle of each instruction; if our `(PC, A, X, Y, P, SP,
//! cycles)` ever drifts from that log we've broken something, and the line number names
//! the offending instruction.
//!
//! See <https://www.nesdev.org/wiki/Emulator_tests#NES_Test_ROM> for the ROM's history and
//! the `nestest.txt` companion file that decodes the result codes left in `$0002`/`$0003`.

use std::fs;
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use wadatsumi_cpu_2a03::CpuStatus;
use wadatsumi_system::System;
use wadatsumi_system_nes::SystemNes;

#[test]
fn nestest() -> anyhow::Result<()> {
    let pak = fs::read("tests/nestest/nestest.nes")?;
    let mut system = SystemNes::open(pak)?;

    // Jump straight to the automation entry point at $C000. The ROM's normal reset vector
    // points at an interactive front-end that needs PPU input and controller input; the
    // $C000 entry runs the same test battery non-interactively and is what the golden log
    // captures. Reset has already run inside `SystemNes::open`, so this just retargets PC.
    system.cpu.pc = 0xc000;

    let log = parse_log(include_str!("nestest/nestest.log"))?;
    let mut expected = log.iter().enumerate();

    // Cycle budget for the official-opcode section. The ROM falls into an infinite loop at
    // this point on a passing run, so a fixed budget keeps the test deterministic and
    // prevents us from sliding into the unofficial-opcode section (which we don't yet
    // implement, and which the log doesn't cover past this point either).
    for _ in 0..26_554 {
        // `t() == 0` is the SYNC cycle: the core is at an instruction boundary and has not
        // yet fetched the next opcode. That's the exact moment the nestest log samples, so
        // it's the only cycle on which a comparison is meaningful.
        if system.cpu.t() == 0
            && let Some((line, entry)) = expected.next()
        {
            let cpu = &system.cpu;

            // Bit 5 (U) of P is hardwired high on the physical chip: it isn't backed by a
            // real flip-flop, so reads always see a 1 there. The log records it as such;
            // we OR it in so internal state and log state agree.
            let p = cpu.p.0 | CpuStatus::U;

            assert!(
                cpu.pc == entry.pc
                    && cpu.a == entry.a
                    && cpu.x == entry.x
                    && cpu.y == entry.y
                    && p == entry.p
                    && cpu.sp == entry.sp
                    && cpu.cycles == entry.cycle,
                "diverged on line {}:\n  expected: PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}\n    actual: PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                line + 1,
                entry.pc,
                entry.a,
                entry.x,
                entry.y,
                entry.p,
                entry.sp,
                entry.cycle,
                cpu.pc,
                cpu.a,
                cpu.x,
                cpu.y,
                p,
                cpu.sp,
                cpu.cycles,
            );
        }

        system.tick();
    }

    // After the official-opcode battery the ROM leaves a result code at $0002 and a
    // sub-test code at $0003. Both zero means every group passed; non-zero values are
    // documented in nestest.txt (see the nesdev wiki link in the module docs).
    assert_eq!(system.cpu_peek(0x0002), 0x00, "nestest result code");
    assert_eq!(system.cpu_peek(0x0003), 0x00, "nestest sub-test code");

    Ok(())
}

/// One entry from the nestest golden log: the CPU's architectural state at the SYNC /
/// opcode-fetch cycle of a single instruction.
#[expect(unused)]
struct LogEntry {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,

    // TODO: assert PPU state
    ppu_scanline: u16,
    ppu_dot: u16,

    /// Absolute CPU cycle count at which this instruction begins.
    cycle: u64,
}

/// Parses the full golden log into one [`LogEntry`] per executed instruction.
fn parse_log(contents: &str) -> anyhow::Result<Vec<LogEntry>> {
    contents.lines().map(parse_log_entry).collect()
}

/// Parses a single log line. Only the trailing register/PPU/cycle block is captured; the
/// disassembly column is decorative and we don't compare against it.
fn parse_log_entry(line: &str) -> anyhow::Result<LogEntry> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        // C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7
        Regex::new(r"^([0-9A-F]{4}).*A:([0-9A-F]{2}) X:([0-9A-F]{2}) Y:([0-9A-F]{2}) P:([0-9A-F]{2}) SP:([0-9A-F]{2}) PPU:\s*(\d+),\s*(\d+) CYC:(\d+)$").unwrap()
    });

    let c = RE
        .captures(line)
        .with_context(|| format!("line did not match nestest log format: {line:?}"))?;

    Ok(LogEntry {
        pc: u16::from_str_radix(&c[1], 16)?,
        a: u8::from_str_radix(&c[2], 16)?,
        x: u8::from_str_radix(&c[3], 16)?,
        y: u8::from_str_radix(&c[4], 16)?,
        p: u8::from_str_radix(&c[5], 16)?,
        sp: u8::from_str_radix(&c[6], 16)?,
        ppu_scanline: c[7].parse()?,
        ppu_dot: c[8].parse()?,
        cycle: c[9].parse()?,
    })
}

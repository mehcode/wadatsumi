use std::fs;
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use wadatsumi_cpu_2a03::Bus;
use wadatsumi_system::System;
use wadatsumi_system_nes::SystemNes;

/// The canonical NES CPU conformance ROM, authored by kevtris.
///
/// The ROM exhaustively exercises every official 2A03 opcode across all
/// addressing modes, verifying cycle counts, flag effects, and memory
/// read/write behavior.
#[test]
fn nestest() -> anyhow::Result<()> {
    let pak = fs::read("tests/nestest/nestest.nes")?;
    let mut system = SystemNes::open(pak)?;

    // Skip the reset vector and jump straight to the automation entry point.
    // The normal reset vector initialises the PPU, which we have not
    // implemented yet; $C000 bypasses all of that.
    system.cpu.pc = 0xc000;

    let log = parse_log(include_str!("nestest/nestest.log"))?;
    let mut expected = log.iter().enumerate();

    // Run for exactly the number of cycles the official-opcode section
    // requires. The ROM halts itself via an infinite loop at this point, so
    // running additional cycles would be harmless, but the fixed budget makes
    // the test deterministic and prevents us from accidentally executing the
    // unofficial-opcode section.
    //
    // The log's cycle counter starts at 7 because the 2A03 reset sequence
    // consumes 7 CPU cycles; since we bypass it with a direct PC write we
    // add 7 to `i` to reproduce that base offset.
    for i in 0..26_554 {
        // t() == 0 is the SYNC cycle: the CPU is at an instruction boundary
        // and has not yet fetched the next opcode.  The nestest log records
        // state at exactly this moment, so it is the right point to compare.
        if system.cpu.t() == 0 {
            if let Some((line, entry)) = expected.next() {
                let cpu = &system.cpu;

                // Bit 5 (U) is hardwired high on the physical chip; OR it in
                // so our comparison matches the log which always has it set.
                let p = cpu.p.0 | 0b0010_0000;

                assert!(
                    cpu.pc == entry.pc
                        && cpu.a == entry.a
                        && cpu.x == entry.x
                        && cpu.y == entry.y
                        && p == entry.p
                        && cpu.sp == entry.sp
                        && 7 + i == entry.cycle,
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
                    7 + i,
                );
            }
        }

        system.tick();
    }

    // A non-zero value here means at least one opcode produced the wrong
    // result; the value itself is a lookup key in nestest.txt.
    assert_eq!(system.bus.read(0x0002), 0x00, "nestest result code");

    // Non-zero only when $0002 is also non-zero; narrows the failure to a
    // specific sub-test within the failing group.
    assert_eq!(system.bus.read(0x0003), 0x00, "nestest sub-test code");

    Ok(())
}

/// One entry from the nestest golden log, representing CPU state at the
/// start of an instruction (i.e. the SYNC / opcode-fetch cycle).
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

/// Parses the full nestest golden log into a [`Vec`] of [`LogEntry`] values,
/// one per logged instruction.
fn parse_log(contents: &str) -> anyhow::Result<Vec<LogEntry>> {
    contents.lines().map(parse_log_entry).collect()
}

/// Parses a single line of the nestest golden log into a [`LogEntry`].
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

// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Drives the 2A03 core against the `SingleStepTests/65x02` `nes6502/v1` corpus: 10 000
//! per-instruction test cases for every one of the 256 opcodes (official and unofficial),
//! checking final register state, modified RAM bytes, and the exact per-cycle bus trace.
//!
//! Each fixture file covers one opcode and was generated from the visual6502 transistor-level
//! simulation, so the expected traces are what real silicon does — not what a reference
//! emulator claims it does. That makes this the strictest CPU conformance test we run:
//! `nestest` checks the architectural state at instruction boundaries, but `SingleStepTests`
//! checks every read, every write, every dummy cycle, in order. A single misplaced dummy
//! read on a page-crossed indexed addressing mode fails here even when nestest passes.
//!
//! Fixtures are cloned into the cargo target directory on first run, so the test is hermetic
//! after the initial download. See <https://github.com/SingleStepTests/65x02> for the corpus
//! and <https://www.nesdev.org/wiki/Emulator_tests#SingleStepTests> for context.

use std::cell::LazyCell;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use libtest_mimic::Trial;
use wadatsumi_cpu_2a03::{Cpu2A03, CpuBus, CpuPeek, CpuReadWrite};

static FIXTURES: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("single-step-nes6502"));

fn main() -> anyhow::Result<()> {
    download_processor_tests()?;

    let args = libtest_mimic::Arguments::from_args();

    let trials = collect_trials()?;

    libtest_mimic::run(&args, trials).exit();
}

/// Exercises a single opcode by running every test case in its fixture file.
///
/// Each case carries an exact initial machine state (registers plus a sparse RAM snapshot),
/// the expected final state after one instruction, and the cycle-by-cycle bus trace
/// `(address, value, read/write)` the real 6502 performs. We run the CPU through that many
/// ticks, then compare all three.
fn exercise_opcode(path: &Path) -> Result<(), libtest_mimic::Failed> {
    let fixture = fs::read_to_string(path)?;
    let cases: Vec<TestCase> = serde_json::from_str(&fixture)?;

    // One bus allocated up front and reused across cases. A naive per-case allocation would
    // be 10 000 × 64 KB on every opcode file, totalling ~2.5 M heap allocations across the
    // whole run — measurable in test wall time.
    let mut bus = FlatCpuReadWrite { ram: vec![0u8; 65536].into_boxed_slice(), trace: Vec::new() };

    for case in cases {
        // Wipe RAM back to zero, then stamp in only the addresses the case declares. The
        // fixture's `initial.ram` is sparse but exhaustive — it lists every address the
        // instruction will read or write — so anything absent can safely stay zero.
        bus.ram.fill(0);
        bus.trace.clear();

        for &(address, val) in &case.initial.ram {
            bus.ram[address as usize] = val;
        }

        // `Cpu2A03::new` parks the core in `Phase::Fetch`, so the very first `tick` executes
        // the instruction sitting at PC instead of replaying the hardware reset sequence.
        // That matches what the fixtures expect — each case starts at an instruction boundary.
        let mut cpu = Cpu2A03::new();

        // The "magic" byte models open-bus / unstable behavior on unofficial opcodes that
        // mix an internal register value into ALU operands. visual6502 (and therefore the
        // SingleStepTests corpus) settles on `0xEE` for this value; we match it so the
        // unofficial-opcode cases line up.
        cpu.magic = 0xEE;

        cpu.pc = case.initial.pc;
        cpu.a = case.initial.a;
        cpu.x = case.initial.x;
        cpu.y = case.initial.y;
        cpu.sp = case.initial.sp;
        cpu.p.0 = case.initial.p;

        // Tick for at most the cycle count the case lists. A normal instruction returns to
        // `t() == 0` exactly at its final cycle, so we can break early. KIL (a.k.a. JAM /
        // HLT) deliberately never reaches another instruction boundary — it stalls the
        // bus forever — so `halted()` keeps us from mistaking that for a clean exit.
        for _ in 0..case.cycles.len() {
            cpu.tick(&mut bus);

            if cpu.t() == 0 && !cpu.halted() {
                break;
            }
        }

        // Bit 5 (U) of P is hardwired high on the physical chip and the fixture's expected
        // P always sets it. Our internal `CpuStatus` doesn't track it, so we OR it back in
        // before comparing.

        let p = cpu.p.0 | 0b0010_0000;
        let fin = &case.r#final;

        if cpu.pc != fin.pc
            || cpu.a != fin.a
            || cpu.x != fin.x
            || cpu.y != fin.y
            || cpu.sp != fin.sp
            || p != fin.p
        {
            return Err(format!(
                "`{}`: mismatched registers\n  expected PC:{:04X} SP:{:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X}\n       got PC:{:04X} SP:{:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X}",
                case.name,
                fin.pc, fin.sp, fin.a, fin.x, fin.y, fin.p,
                cpu.pc, cpu.sp, cpu.a, cpu.x, cpu.y, p,
            )
            .into());
        }

        // `final.ram` is sparse the same way `initial.ram` is — it lists only the addresses
        // the instruction is expected to have touched. We check each declared address;
        // anything absent from the list is intentionally untested.
        for &(address, expected) in &fin.ram {
            let got = bus.ram[address as usize];
            if got != expected {
                return Err(format!(
                    "`{}`: mismatched ram[{address:04X}] expected={expected:02X} got={got:02X}",
                    case.name,
                )
                .into());
            }
        }

        // The cycle trace records every bus transaction the CPU performed, in order: the
        // opcode fetch, address-resolution reads, operand reads/writes, and any dummy
        // cycles. Comparing it against the fixture catches the subtle bugs no other test
        // sees — missing dummy reads, wrong addresses on indexed page-crosses, or a write
        // where the silicon does a read-modify-write.

        if bus.trace.len() != case.cycles.len() {
            return Err(format!(
                "`{}`: mismatched cycle count expected={} got={}",
                case.name,
                case.cycles.len(),
                bus.trace.len(),
            )
            .into());
        }

        for (i, (exp, &(addr, val, is_read))) in
            case.cycles.iter().zip(bus.trace.iter()).enumerate()
        {
            let exp_read = exp.2 == "read";
            if addr != exp.0 || val != exp.1 || is_read != exp_read {
                return Err(format!(
                    "`{}`: mismatched bus cycle {i} expected=({:04X},{:02X},{}) got=({addr:04X},{val:02X},{})",
                    case.name,
                    exp.0,
                    exp.1,
                    exp.2,
                    if is_read { "read" } else { "write" },
                )
                .into());
            }
        }
    }

    Ok(())
}

/// A flat 64 KB address space with a side-channel bus trace, used as the CPU's memory backend
/// during single-step tests.
///
/// Every [`CpuReadWrite::read`] and [`CpuReadWrite::write`] call appends `(address, value,
/// is_read)` to `trace`, giving the test an exact record of what the CPU put on the bus
/// each cycle. [`CpuPeek::peek`] is side-effect-free and does not record — matching the
/// debugger-vs-CPU split the real bus traits draw elsewhere in the crate.
struct FlatCpuReadWrite {
    ram: Box<[u8]>,
    trace: Vec<(u16, u8, bool)>,
}

impl CpuPeek for FlatCpuReadWrite {
    fn peek(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }
}

impl CpuReadWrite for FlatCpuReadWrite {
    fn read(&mut self, address: u16) -> u8 {
        let value = self.peek(address);
        self.trace.push((address, value, true));
        value
    }

    fn write(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
        self.trace.push((address, value, false));
    }
}

// SingleStepTests fixtures cover one instruction in isolation with no interrupt or DMA
// activity, so the trait's defaults (`nmi`/`irq` low, `rdy` high) are exactly what the
// corpus assumes. This empty impl just opts in to the wider bus bound on `tick`.
impl CpuBus for FlatCpuReadWrite {}

/// A snapshot of 6502 machine state at a single point in time, as it appears in the
/// fixture JSON. Field names mirror the on-disk schema (`s` for SP).
#[derive(serde::Deserialize)]
struct CpuState {
    pc: u16,
    a: u8,
    #[serde(rename = "s")]
    sp: u8,
    x: u8,
    y: u8,
    p: u8,

    /// Sparse: only the addresses the case cares about, not the full 64 KB.
    ram: Vec<(u16, u8)>,
}

/// One case from a `SingleStepTests` fixture file: a named scenario for executing a single
/// instruction, with the before/after state and the cycle trace the real chip produces.
#[derive(serde::Deserialize)]
struct TestCase {
    name: String,

    /// Machine state at the instruction-fetch cycle, before the opcode runs.
    initial: CpuState,

    /// Machine state at the next instruction-fetch cycle, after the opcode completes.
    r#final: CpuState,

    /// Every bus transaction the real 6502 performs during execution, in order. The third
    /// tuple field is `"read"` or `"write"`, kept as a `String` because that's how the
    /// JSON encodes it; we compare against it directly rather than translating to an enum.
    cycles: Vec<(u16, u8, String)>,
}

/// Walks the cloned fixture directory and produces one `Trial` per opcode (one JSON file
/// per opcode), letting `libtest_mimic` run them in parallel and report each opcode as its
/// own test in cargo's output.
fn collect_trials() -> anyhow::Result<Vec<Trial>> {
    let mut entries = fs::read_dir(FIXTURES.join("nes6502/v1"))?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .collect::<Vec<_>>();

    entries.sort_by_key(DirEntry::path);

    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_stem()?.to_string_lossy().into_owned();

            Some(Trial::test(format!("single_step::{name}.json"), move || exercise_opcode(&path)))
        })
        .collect())
}

/// Clones the `SingleStepTests/65x02` corpus into the cargo target directory on first run,
/// using a blobless sparse checkout to fetch only `nes6502/v1` (the 6502 variant the 2A03
/// implements). The full repo carries fixtures for every 65x02-family CPU and runs to a
/// gigabyte+; the slice we need is tens of megabytes.
///
/// See <https://github.com/SingleStepTests/65x02/tree/main/nes6502/v1> for the upstream
/// path layout.
fn download_processor_tests() -> anyhow::Result<()> {
    if FIXTURES.exists() {
        // Fixtures are append-only once cloned; if the directory exists we trust it.
        return Ok(());
    }

    let ok = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--filter=blob:none",
            "--sparse",
            "https://github.com/SingleStepTests/65x02",
        ])
        .arg(&*FIXTURES)
        .status()?
        .success();

    anyhow::ensure!(ok, "failed to clone: github.com/SingleStepTests/65x02");

    let ok = Command::new("git")
        .args(["sparse-checkout", "set", "nes6502/v1"])
        .current_dir(&*FIXTURES)
        .status()?
        .success();

    anyhow::ensure!(ok, "failed to sparse-checkout: nes6502/v1");

    Ok(())
}

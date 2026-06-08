use std::cell::LazyCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use libtest_mimic::Trial;
use wadatsumi_cpu_2a03::{Bus, Cpu2A03};

const FIXTURES: LazyCell<PathBuf> =
    LazyCell::new(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("single-step-nes6502"));

fn main() -> anyhow::Result<()> {
    download_processor_tests()?;

    let args = libtest_mimic::Arguments::from_args();

    let trials = collect_trials()?;

    libtest_mimic::run(&args, trials).exit();
}

/// Exercises a single opcode by running ten thousand test cases from SingleStepTests.
///
/// Each test case supplies an exact initial machine state (registers + a sparse RAM snapshot),
/// the expected final state after one instruction, and the exact sequence of bus transactions
/// (address, value, read/write) that the real 6502 performs on every clock cycle.
fn exercise_opcode(path: &Path) -> Result<(), libtest_mimic::Failed> {
    let fixture = fs::read_to_string(path)?;
    let cases: Vec<TestCase> = serde_json::from_str(&fixture)?;

    // Allocate the bus once and reset it between cases to avoid a 64 KB heap allocation
    // per test case (10,000 cases × one file per opcode = 2.56 M potential allocations).

    let mut bus = FlatBus { ram: Box::new([0; 65536]), trace: Vec::new() };

    for case in cases {
        // Restore the bus to a known-zero state, then stamp in only the locations
        // the test cares about. The sparse initial.ram list covers every address the
        // instruction will touch, so anything not listed can safely stay zero.

        bus.ram.fill(0);
        bus.trace.clear();

        for &(address, val) in &case.initial.ram {
            bus.ram[address as usize] = val;
        }

        // Cpu2A03::new() starts in Phase::Fetch so the first tick executes the instruction
        // at PC rather than running the hardware reset sequence.

        let mut cpu = Cpu2A03::new();

        // SingleStepTests is generated from the visual6502 simulation, which uses 0xEE.
        cpu.magic = 0xEE;

        cpu.pc = case.initial.pc;
        cpu.a = case.initial.a;
        cpu.x = case.initial.x;
        cpu.y = case.initial.y;
        cpu.sp = case.initial.sp;
        cpu.p.0 = case.initial.p;

        // Tick for exactly as many cycles as the test case specifies. Normal instructions
        // exit early when t() returns to 0 (instruction boundary). KIL never exits, it jams
        // the CPU in a halt loop, so the halted() check prevents a premature break.
        for _ in 0..case.cycles.len() {
            cpu.tick(&mut bus);

            if cpu.t() == 0 && !cpu.halted() {
                break;
            }
        }

        // U (bit 5) is hardwired high on the physical chip and always set in the test's
        // expected P value, but our internal CpuStatus strips it. OR it back in before
        // comparing so the representations agree.

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

        // final.ram is sparse: it lists only the addresses the instruction is expected to
        // have read or written. We verify each one but leave unmentioned addresses alone.

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

        // The cycle trace records every bus transaction in order: the opcode fetch, any
        // address resolution reads, operand reads/writes, and dummy cycles. A mismatch here
        // catches missing dummy reads, wrong addresses, or incorrect read/write direction.

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
/// Every [`Bus::read`] and [`Bus::write`] call appends an entry to `trace` so the test can
/// verify the exact sequence of bus transactions the CPU performed against the golden cycles
/// list from SingleStepTests. [`Bus::peek`] is side-effect-free and does not record anything.
struct FlatBus {
    ram: Box<[u8; 65536]>,
    trace: Vec<(u16, u8, bool)>,
}

impl Bus for FlatBus {
    fn peek(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

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

/// A snapshot of the 6502 machine state at a single point in time.
#[derive(serde::Deserialize)]
struct CpuState {
    pc: u16,
    a: u8,
    #[serde(rename = "s")]
    sp: u8,
    x: u8,
    y: u8,
    p: u8,

    /// Lists only the addresses relevant to the instruction being tested, not the full 64 KB.
    ram: Vec<(u16, u8)>,
}

/// One test case from SingleStepTests: a named scenario for a single instruction execution.
#[derive(serde::Deserialize)]
struct TestCase {
    name: String,

    /// The machine state before the instruction runs.
    initial: CpuState,

    /// The machine state after the instruction completes.
    r#final: CpuState,

    /// The ordered list of every bus transaction the real 6502 performs during execution.
    cycles: Vec<(u16, u8, String)>,
}

/// Collects each test case (provided from SingleStepTests) into
/// an array of `Trials`.
fn collect_trials() -> anyhow::Result<Vec<Trial>> {
    let mut entries = fs::read_dir(FIXTURES.join("nes6502/v1"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.path());

    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_stem()?.to_string_lossy().into_owned();

            Some(Trial::test(format!("single_step::{name}.json"), move || exercise_opcode(&path)))
        })
        .collect())
}

/// Download the 6502 processor tests from SingleStepTests.
// https://github.com/SingleStepTests/65x02/tree/main/nes6502/v1
fn download_processor_tests() -> anyhow::Result<()> {
    if FIXTURES.exists() {
        // Assume the tests are already downloaded.
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

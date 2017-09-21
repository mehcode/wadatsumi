extern crate flate2;
extern crate reqwest;
extern crate tar;
extern crate time;
extern crate wadatsumi;

use std::fs;
use std::path::Path;
use time::{precise_time_ns, Duration};
use flate2::read::GzDecoder;
use tar::Archive;

const ITERATIONS: u64 = 250_000_000;

use std::sync::{Once, ONCE_INIT};

static START: Once = ONCE_INIT;

// Download the test roms to .cache/ (only if we don't have them already)
// FIXME: Duplicated code with benches/
fn before() {
    const URI_GB_TEST_ROMS: &str = "https://github.com/mehcode/gb-test-roms/archive/master.tar.gz";

    START.call_once(|| {
        if !Path::new(".cache/gb-test-roms-master").exists() {
            let response = reqwest::get(URI_GB_TEST_ROMS).unwrap();
            let mut archive = Archive::new(GzDecoder::new(response).unwrap());

            fs::create_dir_all(".cache").unwrap();

            archive.unpack(".cache").unwrap();
        }
    });
}

#[test]
fn cpu_instrs() {
    before();

    let roms = &[
        // "01-special",
        // "02-interrupts",
        // "03-op sp,hl",
        // "04-op r,imm",
        "05-op rp",
        "06-ld r,r",
        "07-jr,jp,call,ret,rst",
        "08-misc instrs",
        // "09-op r,r",
        "10-bit ops",
        // "11-op a,(hl)",
    ];

    println!(
        "\nrunning {} benchmark{}\n",
        roms.len(),
        if roms.len() == 1 { ' ' } else { 's' }
    );

    for file_name in roms {
        println!("bench {} ...", file_name);

        let mut cpu = wadatsumi::cpu::Cpu::new();

        let file = fs::File::open(&format!(".cache/gb-test-roms-master/cpu_instrs/individual/{}.gb", file_name)).unwrap();
        let cartridge = wadatsumi::cartridge::Cartridge::from_reader(file).unwrap();

        let work_ram = wadatsumi::work_ram::WorkRam::new();
        let high_ram = wadatsumi::high_ram::HighRam::new();

        let mut bus = (cartridge, work_ram, high_ram);
        let start = precise_time_ns();

        for _ in 0..ITERATIONS {
            cpu.run_next(&mut bus);
        }

        let elapsed = Duration::nanoseconds((precise_time_ns() - start) as i64)
            .num_microseconds()
            .unwrap();

        println!(" > elapsed: {}μs", elapsed);
        println!(
            " > instructions/μs: {}\n",
            (ITERATIONS as f64) / (elapsed as f64)
        );
    }
}

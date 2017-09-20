extern crate wadatsumi;
extern crate time;

use std::fs;
use time::{Duration, precise_time_ns};

const ITERATIONS: u64 = 250_000_000;

#[test]
fn cpu_instrs() {
    let benchmarks = &[
        // "01-special.gb",
        // "02-interrupts.gb",
        // "03-op sp,hl.gb",
        // "04-op r,imm.gb",
        "05-op rp.gb",
        "06-ld r,r.gb",
        "07-jr,jp,call,ret,rst.gb",
        "08-misc instrs.gb",
        // "09-op r,r.gb",
        "10-bit ops.gb",
        // "11-op a,(hl).gb",
    ];

    println!("\nrunning {} benchmark{}\n", benchmarks.len(), if benchmarks.len() == 1 { ' ' } else { 's' });

    for benchmark in benchmarks {
        println!("bench {} ...", benchmark);

        let mut cpu = wadatsumi::cpu::Cpu::new();
        let f = fs::File::open(&format!("/home/mehcode/Workspace/github.com/retrio/gb-test-roms/cpu_instrs/individual/{}", benchmark)).unwrap();

        let cartridge = wadatsumi::cartridge::Cartridge::from_reader(f).unwrap();
        let work_ram = wadatsumi::work_ram::WorkRam::new();
        let high_ram = wadatsumi::high_ram::HighRam::new();

        let mut bus = (cartridge, work_ram, high_ram);
        let start = precise_time_ns();

        for _ in 0..ITERATIONS {
            cpu.run_next(&mut bus);
        }

        let elapsed = Duration::nanoseconds((precise_time_ns() - start) as i64).num_microseconds().unwrap();

        println!(" > elapsed: {}μs", elapsed);
        println!(" > instructions/μs: {}", (ITERATIONS as f64) / (elapsed as f64));
        println!("");
    }
}

// /*
//
// running 1 test
// Gnuplot not found, disabling plotting
// Benchmarking fib5
// > Warming up for 3.0000 s
// > Collecting 100 samples in estimated 5.0001 s
// > Performing linear regression
//   >  slope [23.231 ns 23.393 ns]
//   >    R^2  0.9717595 0.9717867
// > Estimating the statistics of the sample
//   >   mean [23.170 ns 23.306 ns]
//   > median [23.038 ns 23.261 ns]
//   >    MAD [245.17 ps 522.79 ps]
//   >     SD [314.85 ps 375.77 ps]
//
// test fib5 ... ok
//
//  */

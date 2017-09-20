extern crate criterion;
extern crate wadatsumi;

use std::fs;
use criterion::Bencher;

#[test]
fn cpu_instrs() {
    //let mut cpu = wadatsumi::cpu::Cpu::new();
    //// TODO: Make individual benchmarks for each test
    //let f = fs::File::open("/home/mehcode/Workspace/github.com/retrio/gb-test-roms/cpu_instrs/individual/10-bit ops.gb").unwrap();
    //
    //let cartridge = wadatsumi::cartridge::Cartridge::from_reader(f).unwrap();
    //let work_ram = wadatsumi::work_ram::WorkRam::new();
    //let high_ram = wadatsumi::high_ram::HighRam::new();
    //
    //let mut bus = (cartridge, (work_ram, high_ram));
    //
    //b.iter(|| cpu.run_next(&mut bus));
}

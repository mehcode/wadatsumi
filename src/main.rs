extern crate ansi_term;
extern crate chrono;
extern crate env_logger;
extern crate log;
extern crate wadatsumi;

mod logger;

use std::fs;
use log::LogLevelFilter;

fn main() {
    // TODO: Allow configuration in command line options
    logger::init(LogLevelFilter::Trace).unwrap();

    let mut cpu = wadatsumi::cpu::Cpu::new();

    cpu.reset();

    let f = fs::File::open("tests/cpu_instrs/individual/06-ld r,r.gb").unwrap();
    let mut cartridge = wadatsumi::cartridge::Cartridge::from_reader(f).unwrap();

    loop {
        cpu.run_next(&mut cartridge);
    }
}

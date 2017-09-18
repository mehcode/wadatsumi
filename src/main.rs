extern crate ansi_term;
extern crate chrono;
extern crate env_logger;
extern crate log;
extern crate wadatsumi;

mod logger;

use std::env;
use std::fs;
use log::LogLevelFilter;
use wadatsumi::*;

fn main() {
    // TODO: Allow configuration in command line options
    logger::init(LogLevelFilter::Trace).unwrap();

    // TODO: Parse arguments properly
    let argv: Vec<_> = env::args().collect();

    let mut cpu = cpu::Cpu::new();

    cpu.reset();

    let f = fs::File::open(&argv[1]).unwrap();

    let cartridge = cartridge::Cartridge::from_reader(f).unwrap();
    let work_ram = work_ram::WorkRam::new();
    let mut bus = (cartridge, work_ram);

    loop {
        cpu.run_next(&mut bus);
    }
}

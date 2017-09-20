extern crate ansi_term;
extern crate chrono;
extern crate env_logger;
extern crate log;
extern crate wadatsumi;

mod logger;

use std::time;
use std::env;
use std::fs;
use log::LogLevelFilter;
use wadatsumi::*;

#[derive(Default)]
struct SerialDataCapture;

impl bus::Bus for SerialDataCapture {
    fn contains(&self, address: u16) -> bool {
        0xFF01 == address
    }

    fn write8(&mut self, _: u16, value: u8) {
    }
}

fn main() {
    // TODO: Allow configuration in command line options
    // logger::init(LogLevelFilter::Trace).unwrap();

    // TODO: Parse arguments properly
    let argv: Vec<_> = env::args().collect();

    let mut cpu = cpu::Cpu::new();

    cpu.reset();

    let f = fs::File::open(&argv[1]).unwrap();

    let cartridge = cartridge::Cartridge::from_reader(f).unwrap();
    let work_ram = work_ram::WorkRam::new();
    let high_ram = high_ram::HighRam::new();
    let serial_data_capture = SerialDataCapture;

    let mut bus = (cartridge, (work_ram, (high_ram, serial_data_capture)));
    let now = time::Instant::now();
    let instructions = 100_000_000;

    for _ in 0..instructions {
        cpu.run_next(&mut bus);
    }

    let elapsed = now.elapsed();
    let microseconds = ((elapsed.as_secs() * 1_000_000) + (elapsed.subsec_nanos() as u64)) / 1_000;

    println!("elapsed: {} µs", microseconds);
    println!("ipµs: {}", (instructions as f64) / (microseconds as f64));
}

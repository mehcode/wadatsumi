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

#[derive(Default)]
struct SerialDataCapture;

impl bus::Bus for SerialDataCapture {
    fn contains(&self, address: u16) -> bool {
        0xFF02 == address
    }

    fn read8(&self, _: u16) -> u8 {
        0xff
    }

    fn write8(&mut self, _: u16, value: u8) {
        print!("{}", value as char)
    }
}

fn main() {
    // TODO: Allow configuration in command line options
    // logger::init(LogLevelFilter::Warn).unwrap();

    // TODO: Parse arguments properly
    let argv: Vec<_> = env::args().collect();

    let mut cpu = cpu::Cpu::new();

    cpu.reset();

    let f = fs::File::open(&argv[1]).unwrap();

    let cartridge = cartridge::Cartridge::from_reader(f).unwrap();
    let work_ram = work_ram::WorkRam::new();
    let serial_data_capture = SerialDataCapture;

    let mut bus = (cartridge, (work_ram, serial_data_capture));

    loop {
        cpu.run_next(&mut bus);
    }
}

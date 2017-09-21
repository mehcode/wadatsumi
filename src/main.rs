extern crate wadatsumi;
extern crate chrono;
extern crate env_logger;
extern crate log;
extern crate ansi_term;

mod logger;

use std::{env, fs};

#[derive(Default)]
struct SerialDataPrint;

impl wadatsumi::Bus for SerialDataPrint {
    fn contains(&self, address: u16) -> bool {
        0xFF01 == address
    }

    fn write8(&mut self, _: u16, value: u8) {
        print!("{}", value as char);
    }
}

fn main() {
    logger::init(log::LogLevelFilter::Error).unwrap();

    let argv: Vec<_> = env::args().skip(1).collect();

    let mut cpu = wadatsumi::Cpu::new();

    let file = fs::File::open(&argv[0]).unwrap();
    let cartridge = wadatsumi::Cartridge::from_reader(file).unwrap();

    let work_ram = wadatsumi::WorkRam::new();
    let high_ram = wadatsumi::HighRam::new();

    let mut bus = (cartridge, (work_ram, (high_ram, SerialDataPrint)));

    loop {
        cpu.run_next(&mut bus);
    }
}

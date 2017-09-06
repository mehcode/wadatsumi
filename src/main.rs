extern crate env_logger;
extern crate log;
extern crate chrono;
extern crate wadatsumi;
extern crate ansi_term;

use std::fs;
use env_logger::{LogBuilder};
use log::{LogLevel, LogLevelFilter};
use ansi_term::Colour;

fn main() {
    // Logger

    let mut log_builder = LogBuilder::new();
    log_builder.format(|record| {
        const LOG_LEVEL_SHORT_NAMES: [&'static str; 6] =
            ["OFF", "ERRO", "WARN", "INFO", "DEBG", "TRCE"];

        let lvl = record.level();
        let lvl_color = match lvl {
            LogLevel::Error => 9,
            LogLevel::Warn => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 6,
            LogLevel::Trace => 4,
        };

        let lvl_s = LOG_LEVEL_SHORT_NAMES[lvl as usize];

        format!("{} {} {}",
            chrono::Local::now().format("%H:%M:%S"),
            // record.target(),
            Colour::Fixed(lvl_color).paint(lvl_s),
            Colour::Fixed(15).paint(record.args().to_string()),
        )
    });

    // TODO: Allow configuring from a cli option
    log_builder.filter(Some("wadatsumi"), LogLevelFilter::Trace);

    log_builder.init().unwrap();

    let mut cpu = wadatsumi::cpu::Cpu::new();

    cpu.reset();

    let f = fs::File::open("tests/cpu_instrs/individual/06-ld r,r.gb").unwrap();
    let mut cartridge = wadatsumi::cartridge::Cartridge::from_reader(f).unwrap();

    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
    cpu.run_next(&mut cartridge);
}

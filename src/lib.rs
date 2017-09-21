#![allow(unused_extern_crates, unused_imports)]
#![feature(range_contains, inclusive_range_syntax, box_syntax)]

extern crate ansi_term;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

pub mod errors;
pub mod cartridge;
pub mod cpu;
pub mod bus;
pub mod work_ram;
pub mod high_ram;

pub use self::cpu::Cpu;
pub use self::bus::Bus;
pub use self::cartridge::Cartridge;
pub use self::high_ram::HighRam;
pub use self::work_ram::WorkRam;

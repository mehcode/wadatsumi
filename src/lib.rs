#![allow(unused_extern_crates, unused_imports)]
#![feature(range_contains, inclusive_range_syntax)]

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

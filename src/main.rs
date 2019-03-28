#![allow(unused)]

mod decode;
mod instruction;
mod state;
mod memory;
mod execute;
mod cpu;

use self::instruction::Instruction;
use byteorder::{ByteOrder, LE};
use failure::Error;
use std::fs;
use self::cpu::Cpu;
use self::state::State;
use self::memory::Memory;

fn main() -> Result<(), Error> {
    let mut rom = fs::read("./hello_world.gba")?;
    let mut mem = Memory::new(rom);
    let mut cpu = Cpu::default();

    let mut index = 0;

    cpu.state.r15 = 0x0800_0000;

    loop {
        index += 1;

        cpu.run_next(&mut mem);

        if index > 50 {
            break;
        }
    }

    Ok(())
}

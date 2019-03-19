#![allow(unused)]

mod decode;
mod instruction;

use self::instruction::Instruction;
use byteorder::{ByteOrder, LE};
use failure::Error;
use std::fs;

fn main() -> Result<(), Error> {
    let mut r15: u32 = 0x0800_0000;
    let mut rom = fs::read("./Advance Wars (USA).gba")?;
    let mut index = 0;

    loop {
        index += 1;

        let opcode = LE::read_u32(&rom[(r15 - 0x0800_0000) as usize..]);
        let ix = decode::decode(opcode);

        println!("{:5} [0x{:x}] {}", index, r15, ix);

        if let Instruction::Branch { offset, .. } = ix {
            r15 = r15.wrapping_add(offset) + 8;
        } else {
            r15 += 4;
        }

        if index > 50 {
            break;
        }
    }

    Ok(())
}

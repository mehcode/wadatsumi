use crate::chip_8::Chip8;
use clap::Parser;

mod chip_8;
mod instruction;
mod opcode;

#[derive(Parser)]
struct Args {
    // the ROM file to open
    input: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut chip8 = Chip8::new();

    chip8.open(&args.input)?;

    // DEBUG: run 50 steps
    for _ in 0..50 {
        chip8.step()?;
    }

    Ok(())
}

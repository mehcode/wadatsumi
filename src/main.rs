use crate::opcode::Opcode;
use crate::state::State;
use clap::Parser;
use std::fs::File;
use std::io::Read;

mod opcode;
mod state;

#[derive(Parser)]
struct Args {
    // the ROM file to open
    input: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Read in the ROM
    let mut file = File::open(args.input)?;
    let mut rom = Vec::new();
    file.read_to_end(&mut rom)?;

    let mut state = State::new();

    // Copy the ROM into RAM at the entry point
    state.ram[state.pc..][..rom.len()].copy_from_slice(&rom);

    loop {
        let opcode = fetch(&mut state);
        execute(&mut state, opcode)?;
    }

    Ok(())
}

fn fetch(state: &mut State) -> Opcode {
    Opcode::fetch(&*state.ram, &mut state.pc)
}

fn execute(state: &mut State, opcode: Opcode) -> anyhow::Result<()> {
    match opcode.digits() {
        // Clear the display (`00E0`).
        (0x0, 0x0, 0xE, 0x0) => {
            state.display.clear();
        }

        // Jump to location `nnn` (`1nnn`).
        (0x1, ..) => {
            state.pc = opcode.nnn().into();
        }

        // Skip next instruction if `Vx` is equal to `kk` (`3xkk`).
        (0x3, x, ..) => {
            let x = usize::from(x);

            if state.v[x] == opcode.kk() {
                state.pc += 2;
            }
        }

        // Skip next instruction if `Vx` is not equal to `kk` (`4xkk`).
        (0x4, x, ..) => {
            let x = usize::from(x);

            if state.v[x] != opcode.kk() {
                state.pc += 2;
            }
        }

        // Skip next instruction if `Vx` is equal to `Vy` (`5xy0`).
        (0x5, x, y, 0) => {
            let x = usize::from(x);
            let y = usize::from(y);

            if state.v[x] == state.v[y] {
                state.pc += 2;
            }
        }

        // Puts the value `kk` into the register `Vx` (`6xkk`).
        (0x6, x, ..) => {
            let x = usize::from(x);

            state.v[x] = opcode.kk();
        }

        // Sets `Vx` to the result of `Vx` and `kk` (`7xkk`).
        (0x7, x, ..) => {
            let x = usize::from(x);

            state.v[x] = state.v[x].wrapping_add(opcode.kk());
        }

        // Sets `Vx` to the bitwise OR of `Vx` and `Vy` (`8xy1`).
        (0x8, x, y, 0x1) => {
            let x = usize::from(x);
            let y = usize::from(y);

            state.v[x] |= state.v[y];
        }

        // Sets `Vx` to the bitwise AND of `Vx` and `Vy` (`8xy2`).
        (0x8, x, y, 0x2) => {
            let x = usize::from(x);
            let y = usize::from(y);

            state.v[x] &= state.v[y];
        }

        // Sets `Vx` to the bitwise exclusive OR of `Vx` and `Vy` (`8xy3`).
        (0x8, x, y, 0x3) => {
            let x = usize::from(x);
            let y = usize::from(y);

            state.v[x] ^= state.v[y];
        }

        // Sets `Vx` to the result of subtracting `Vy` from `Vx` (`8xy5`).
        (0x8, x, y, 0x5) => {
            let x = usize::from(x);
            let y = usize::from(y);

            let vx = state.v[x];
            let vy = state.v[y];

            state.v[0xf] = (vy <= vx) as u8;
            state.v[x] = vx.wrapping_sub(vy);
        }

        // Sets `Vx` to the result of subtracting `Vx` from `Vy` (`8xy7`).
        (0x8, x, y, 0x7) => {
            let x = usize::from(x);
            let y = usize::from(y);

            let vx = state.v[x];
            let vy = state.v[y];

            state.v[0xf] = (vx <= vy) as u8;
            state.v[x] = vy.wrapping_sub(vx);
        }

        // Sets `Vx` to the result of `Vy` shifted left by 1 (`8xyE`).
        (0x8, x, y, 0xE) => {
            let x = usize::from(x);
            let y = usize::from(y);

            state.v[0xf] = (state.v[y] >> 7) & 0b1;
            state.v[x] = state.v[y] << 1;
        }

        _ => anyhow::bail!("unknown opcode {}", opcode),
    }

    Ok(())
}

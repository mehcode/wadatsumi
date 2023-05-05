use crate::instruction::Instruction;
use crate::opcode::Opcode;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const RAM_SIZE: usize = 0x1000;
const ENTRY_POINT: usize = 0x200;

pub struct Chip8 {
    /// A copy of the ROM, used to quickly reset the state.
    rom: Vec<u8>,

    /// CHIP-8 has direct access to up to 4 kB of RAM.
    ram: Box<[u8; RAM_SIZE]>,

    /// A program counter (PC) which points at the current instruction in memory.
    pc: usize,

    /// CHIP-8 has 16 8-bit (1 byte) general-purpose variable registers called
    /// `V0` through `VF`.
    v: [u8; 0x10],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            v: [0; 0x10],
            pc: ENTRY_POINT,
            rom: Vec::new(),
            // do the dance to get a heap-allocated fixed-size array
            // sure would be nice if we had a better way to write that
            ram: vec![0; RAM_SIZE].into_boxed_slice().try_into().unwrap(),
        }
    }

    pub fn open(&mut self, filename: impl AsRef<Path>) -> anyhow::Result<()> {
        // Read in the ROM
        let mut file = File::open(filename)?;

        self.rom.clear();
        file.read_to_end(&mut self.rom)?;

        // Initialize the state
        self.reset();

        Ok(())
    }

    pub fn reset(&mut self) {
        self.v.fill(0);
        self.ram.fill(0);
        self.pc = ENTRY_POINT;

        // Copy the ROM into RAM at the entry point
        self.ram[self.pc..][..self.rom.len()].copy_from_slice(&self.rom);
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        let opcode = Opcode::fetch(&*self.ram, &mut self.pc);
        let instr = Instruction::decode(opcode)?;

        #[allow(unreachable_patterns)]
        match instr {
            Instruction::ClearScreen => {
                // TODO: clear screen
            }

            Instruction::LoadRegister { x, kk } => {
                self.v[x as usize] = kk;
            }

            Instruction::AddValue { x, kk } => {
                self.v[x as usize] += kk;
            }

            Instruction::SkipIfEqualToValue { x, kk } => {
                if self.v[x as usize] == kk {
                    self.pc += 2;
                }
            }

            Instruction::SkipIfNotEqualToValue { x, kk } => {
                if self.v[x as usize] != kk {
                    self.pc += 2;
                }
            }

            Instruction::SkipIfEqualToRegister { x, y } => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }

            _ => todo!("unimplemented instruction {:?}", instr),
        }

        Ok(())
    }
}

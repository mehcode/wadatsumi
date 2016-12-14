use std::fmt;

use ::bus;
use ::operation;

bitflags!(
    #[derive(Default)]
    pub flags Flags: u8 {
        const ZERO         = 0b_1000_0000,      // Z
        const ADD_SUBTRACT = 0b_0100_0000,      // N
        const HALF_CARRY   = 0b_0010_0000,      // H
        const CARRY        = 0b_0001_0000       // C
    }
);

impl fmt::UpperHex for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.bits, f)
    }
}

// TODO(rust): Better name than `Context` here?
#[derive(Default)]
pub struct Context {
    /// Registers (8-bit)
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    /// Flags (F)
    pub f: Flags,

    /// Program Counter (PC)
    pub pc: u16,

    /// Stack Pointer (SP)
    pub sp: u16,

    /// Interrupt Master Enable (ime)
    ///  -1 - Pending state that goes to ON (on next cycle)
    ///   0 - OFF
    ///  +1 - ON
    pub ime: i8,

    /// STOP state; true/false to indicate if CPU is in STOP mode
    pub stop: bool,

    /// HALT state
    ///   0 - OFF
    ///   1 - ON
    ///  -1 - Funny bug state that will replay the next opcode
    pub halt: i8,

    /// Current instruction M-cycle counter
    cycles: u32,

    /// Total/Running M-cycle counter
    total_cycles: u32,
}

impl Context {
    /// Step the machine for a single M-cycle
    pub fn step(&mut self, bus: &mut bus::Bus) {
        // The idea of this is simple cooperative multitasking. During
        // execution in the CPU; when it decides a M-cycle of time has passed it invokes
        // this method which delegates to `Machine::step` which then steps the rest of
        // the system exactly 1 M-cycle. This keeps the machine in synch with the CPU.

        bus.step();
        self.cycles += 1;
        self.total_cycles += 1;
    }

    /// Set flag to passed value
    #[inline]
    pub fn set_flag(&mut self, flag: Flags, value: bool) {
        if value {
            self.f |= flag;
        } else {
            self.f &= !flag;
        }
    }

    /// Get 16-bit Register: BC
    #[inline]
    pub fn get_bc(&self) -> u16 {
        self.c as u16 | ((self.b as u16) << 8)
    }

    /// Get 16-bit Register: DE
    #[inline]
    pub fn get_de(&self) -> u16 {
        self.d as u16 | ((self.e as u16) << 8)
    }

    /// Get 16-bit Register: HL
    #[inline]
    pub fn get_hl(&self) -> u16 {
        self.h as u16 | ((self.l as u16) << 8)
    }

    /// Get 16-bit Register: AF
    #[inline]
    pub fn get_af(&self) -> u16 {
        self.a as u16 | ((self.f.bits as u16) << 8)
    }

    /// Set 16-bit Register: BC
    #[inline]
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    /// Set 16-bit Register: DE
    #[inline]
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    /// Set 16-bit Register: HL
    #[inline]
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    /// Set 16-bit Register: AF
    #[inline]
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = Flags::from_bits_truncate((value & 0xFF) as u8);
    }
}

#[derive(Default)]
pub struct CPU {
    /// Context
    ctx: Context,

    /// Operation table
    table: operation::Table,

    /// [0xFFFF] Interrupt Enable (IE) R/W
    ie: u8,

    /// [0xFF0F] Interrupt Flag (IF) R/W
    if_: u8,
}

impl CPU {
    pub fn reset(&mut self) {
        // Registers
        // TODO(gameboy): Dependent on model/variant
        self.ctx.a = 0x01;
        self.ctx.b = 0x00;
        self.ctx.c = 0x13;
        self.ctx.d = 0x00;
        self.ctx.e = 0xD8;
        self.ctx.h = 0x01;
        self.ctx.l = 0x4D;
        self.ctx.f = Flags::from_bits_truncate(0xB0);
        self.ctx.sp = 0xFFFE;

        // Program counter
        // TODO(gameboy): Dependent on BIOS on/off
        self.ctx.pc = 0x0100;

        // Cycles
        self.ctx.cycles = 0;
        self.ctx.total_cycles = 0;

        // Stop/Halt states
        self.ctx.stop = false;
        self.ctx.halt = 0;

        // Interrupt Enable/Flag
        self.ie = 0;
        self.if_ = 0;
    }

    /// Run next instruction
    pub fn run_next(&mut self, bus: &mut bus::Bus) -> u32 {
        // Reset "current" cycle counter
        self.ctx.cycles = 0;

        // If CPU is currently in STOP mode;
        // Or, If CPU is currently in HALT mode with no pending interrupts
        if self.ctx.stop || (self.ctx.halt == 1 && (self.ie & self.if_ == 0)) {
            // Step a single M-cycle and return
            self.ctx.step(bus);
            self.ctx.cycles = 1;

            return self.ctx.cycles;
        }

        // Leave HALT mode if interrupts are disabled
        if self.ctx.halt == 1 && self.ctx.ime == 0 {
            self.ctx.halt = 0;
        }

        // TODO: Service interrupt (if needed)

        // Enable IME if pending
        if self.ctx.ime == -1 {
            self.ctx.ime = 1;
        }

        // Operation: decode
        let pc = self.ctx.pc;
        let mut opcode = bus.read(self.ctx.pc) as usize;
        self.ctx.pc += 1;
        self.ctx.step(bus);

        // TODO: On HALT bug; replay PC value here

        // On 0xCB; offset our opcode and read the next byte to determine the final opcode
        if opcode == 0xCB {
            opcode += 0x100;
            opcode |= bus.read(self.ctx.pc) as usize;
            self.ctx.pc += 1;
            self.ctx.step(bus);
        }

        let ref op = self.table[opcode as usize];
        if op.is_empty() {
            panic!(if opcode < 0x100 {
                format!("unknown opcode: {:#02X}", opcode & 0xFF)
            } else {
                format!("unknown opcode: 0xCB {:#02X}", opcode & 0xFF)
            });
        }

        // Trace: Operation
        trace!("{:>10}: {:<40} PC: 0x{:04X} AF: 0x{:02X}{:02X} BC: 0x{:02X}{:02X} DE: 0x{:02X}{:02X} HL: 0x{:02X}{:02X} SP: 0x{:04X}",
                 self.ctx.total_cycles,
                 op.format(&self.ctx, bus).unwrap(),
                 pc,
                 self.ctx.a,
                 self.ctx.f,
                 self.ctx.b,
                 self.ctx.c,
                 self.ctx.d,
                 self.ctx.e,
                 self.ctx.h,
                 self.ctx.l,
                 self.ctx.sp);

        // Operation: execute
        (op.handle)(&mut self.ctx, bus);

        self.ctx.cycles
    }
}

bitflags!(
    pub flags Flags: u8 {
        const ZERO         = 0b_1000_0000,
        const ADD_SUBTRACT = 0b_0100_0000,
        const HALF_CARRY   = 0b_0010_0000,
        const CARRY        = 0b_0001_0000
    }
);

pub struct CPU {
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

    /// Current instruction M-cycle counter
    cycles: u32,

    /// STOP state; true/false to indicate if CPU is in STOP mode
    stop: bool,

    /// HALT state
    ///   0 - OFF
    ///   1 - ON
    ///  -1 - Funny bug state that will replay the next opcode
    halt: i8,

    /// Interrupt Master Enable (ime)
    ///  -1 - Pending state that goes to ON (on next cycle)
    ///   0 - OFF
    ///  +1 - ON
    ime: i8,

    /// [0xFFFF] Interrupt Enable (IE) R/W
    ie: u8,

    /// [0xFF0F] Interrupt Flag (IF) R/W
    if_: u8,
}

impl CPU {
    pub fn new() -> Self {
        // TODO(rust): Any way to default all this default init ?
        CPU {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: Flags::empty(),
            pc: 0,
            sp: 0,
            cycles: 0,
            stop: false,
            halt: 0,
            ime: 0,
            ie: 0,
            if_: 0,
        }
    }

    pub fn reset(&mut self) {
        // Registers
        // TODO(gameboy): Dependent on model/variant
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.f = Flags::empty();
        self.sp = 0;

        // Program counter
        // TODO(gameboy): Dependent on BIOS on/off
        self.pc = 0x0100;

        // Cycles (for current instruction)
        self.cycles = 0;

        // Stop/Halt states
        self.stop = false;
        self.halt = 0;

        // Interrupt Enable/Flag
        self.ie = 0;
        self.if_ = 0;
    }

    /// Step the machine for a single M-cycle
    pub fn step(&mut self) {
        // The idea of this is simple cooperative multitasking. During
        // execution in the CPU; when it decides a M-cycle of time has passed it invokes
        // this method which delegates to `Machine::step` which then steps the rest of
        // the system exactly 1 M-cycle. This keeps the machine in synch with the CPU.

        // TODO: self.machine.step();
        self.cycles += 1;
    }

    /// Run next instruction
    pub fn run_next(&mut self) -> u32 {
        // Reset "current" cycle counter
        self.cycles = 0;

        // If CPU is currently in STOP mode;
        // Or, If CPU is currently in HALT mode with no pending interrupts
        if self.stop || (self.halt == 1 && (self.ie & self.if_ == 0)) {
            // Step a single M-cycle and return
            self.step();
            self.cycles = 1;

            return self.cycles;
        }

        // Leave HALT mode if interrupts are disabled
        if self.halt == 1 && self.ime == 0 {
            self.halt = 0;
        }

        // TODO: Service interrupt (if needed)

        // Enable IME if pending
        if self.ime == -1 {
            self.ime = 1;
        }

        // TODO: Operation: decode

        // TODO: Operation: execute

        self.cycles
    }
}

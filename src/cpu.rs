
bitflags!(
    pub flags Flags: u8 {
        const ZERO         = 0b_1000_0000,
        const ADD_SUBTRACT = 0b_0100_0000,
        const HALF_CARRY   = 0b_0010_0000,
        const CARRY        = 0b_0001_0000
    }
);

pub struct CPU {
    // Registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Flags (F)
    pub f: Flags,

    // Program Counter (PC)
    pub pc: u16,

    // Stack Pointer (SP)
    pub sp: u16,
}

impl CPU {
    // fn new() -> Self {}

    fn reset(&mut self) {
        // Registers
        // TODO: Dependent on model/variant

        // Program counter
        // TODO: Dependent on BIOS on/off
        self.pc = 0x0100;
    }
}

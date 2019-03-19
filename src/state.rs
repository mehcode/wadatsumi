use std::ops::{Index, IndexMut};
use crate::instruction::Register;

#[derive(Debug, Default)]
pub struct
State {
    // When the R15 (PC) register is written to during instruction execution, a pipeline flush
    // needs to be requested that will wipe pending instructions
    pub needs_pipeline_flush: bool,

    //
    // General Purpose Registers
    //

    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,

    // Stack Pointer (SP)
    pub r13: u32,

    // Link Register (LR)
    pub r14: u32,

    // Program Counter (PC)
    pub r15: u32,

    // Current Program Status Register - CPSR
    pub cpsr: u32,

    //
    // Banked Fast Interrupt [FIQ] Registers
    //

    pub r8_fiq: u32,
    pub r9_fiq: u32,
    pub r10_fiq: u32,
    pub r11_fiq: u32,
    pub r12_fiq: u32,
    pub r13_fiq: u32,
    pub r14_fiq: u32,
    pub spsr_fiq: u32,

    //
    // Banked Supervisor [SVC] Registers
    //

    pub r13_svc: u32,
    pub r14_svc: u32,
    pub spsr_svc: u32,

    //
    // Banked Abort [ABT] Registers
    //

    pub r13_abt: u32,
    pub r14_abt: u32,
    pub spsr_abt: u32,

    //
    // Banked Interrupt [IRQ] Registers
    //

    pub r13_irq: u32,
    pub r14_irq: u32,
    pub spsr_irq: u32,

    //
    // Banked Undefined [UNK] Registers
    //

    pub r13_und: u32,
    pub r14_und: u32,
    pub spsr_und: u32,
}

impl State {
    pub fn r(&self, index: Register) -> &u32 {
        // todo: handle CPU modes
        match index {
            Register::R0 => &self.r0,
            Register::R1 => &self.r1,
            Register::R2 => &self.r2,
            Register::R3 => &self.r3,
            Register::R4 => &self.r4,
            Register::R5 => &self.r5,
            Register::R6 => &self.r6,
            Register::R7 => &self.r7,
            Register::R8 => &self.r8,
            Register::R9 => &self.r9,
            Register::R10 => &self.r10,
            Register::R11 => &self.r11,
            Register::R12 => &self.r12,
            Register::R13 => &self.r13,
            Register::R14 => &self.r14,
            Register::R15 => &self.r15,
        }
    }

    pub fn r_mut(&mut self, index: Register) -> &mut u32 {
        // todo: handle CPU modes
        match index {
            Register::R0 => &mut self.r0,
            Register::R1 => &mut self.r1,
            Register::R2 => &mut self.r2,
            Register::R3 => &mut self.r3,
            Register::R4 => &mut self.r4,
            Register::R5 => &mut self.r5,
            Register::R6 => &mut self.r6,
            Register::R7 => &mut self.r7,
            Register::R8 => &mut self.r8,
            Register::R9 => &mut self.r9,
            Register::R10 => &mut self.r10,
            Register::R11 => &mut self.r11,
            Register::R12 => &mut self.r12,
            Register::R13 => &mut self.r13,
            Register::R14 => &mut self.r14,
            Register::R15 => &mut self.r15,
        }
    }
}

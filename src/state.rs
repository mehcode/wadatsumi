// Current Program Status Register (CPSR)

// [31 - 28] Condition Code Flags (N, Z, C, V)
//  These bits reflect results of logical or arithmetic operations.

/// CPSR Sign (N) Flag
/// 0 = Not Signed, 1 = Signed
pub const CPSR_N_FLAG: u32 = 1 << 31;

/// CPSR Zero (Z) Flag
/// 0 = Not Zero, 1 = Zero
pub const CPSR_Z_FLAG: u32 = 1 << 30;

/// CPSR Carry (C) Flag
/// 0 = Borrow / No Carry, 1 = Carry / No Borrow
pub const CPSR_C_FLAG: u32 = 1 << 29;

/// CPSR Overflow (V) Flag
/// 0 = No Overflow, 1 = Overflow
pub const CPSR_V_FLAG: u32 = 1 << 28;

/// CPSR Sticky Overflow (Q) Flag
/// 1 = Sticky Overflow (ARMv5TE+)
///
/// Used by `QADD`, `QSUB`, `QDADD`, `QDSUB`, `SMLAxy`, and `SMLAWy` only.
/// These opcodes set the Q-flag in case of overflows, but leave it unchanged otherwise.
/// The Q-flag can be tested/reset by MSR/MRS opcodes only.
pub const CPSR_Q_FLAG: u32 = 1 << 27;

// [26 - 8] Reserved

// [7 - 0] Control Bits (I, F, T, M)
//  These bits may change when an exception occurs. In privileged modes (non-user modes)
//  they may be also changed manually.

/// CPSR IRQ Disable (I) Flag
/// 0 = Enable, 1 = Disable
const CPSR_IRQ_DISABLE: u32 = 1 << 7;

/// CPSR FIQ Disable (F) Flag
/// 0 = Enable, 1 = Disable
const CPSR_FIQ_DISABLE: u32 = 1 << 6;

/// CPSR State (T)
/// 0 = ARM, 1 = THUMB
///
/// The T Bit signalizes the current state of the CPU, this bit should never be changed
/// manually - instead, changing between ARM and THUMB state must be done by BX instructions.
const CPSR_STATE: u32 = 1 << 5;

// The Mode bits 0-4 contain the current operating mode, which affects banked register usage
// and whether the CPU is operating under privileges.

/// CPSR Mode (M) Mask
const CPSR_MODE: u32 = 0b11111;

/// CPSR Mode - User (USR) (non-privileged)
pub const CPSR_MODE_USR: u32 = 0x10;

/// CPSR Mode - Fast Interrupt (FIQ)
pub const CPSR_MODE_FIQ: u32 = 0x11;

/// CPSR Mode - Interrupt (IRQ)
pub const CPSR_MODE_IRQ: u32 = 0x12;

/// CPSR Mode - Supervisor (SVC)
pub const CPSR_MODE_SVC: u32 = 0x13;

/// CPSR Mode - Abort (ABT)
pub const CPSR_MODE_ABT: u32 = 0x17;

/// CPSR Mode - Undefined (UND)
pub const CPSR_MODE_UND: u32 = 0x1b;

/// CPSR Mode - System (SYS) (privileged 'User' mode, ARMv4+)
pub const CPSR_MODE_SYS: u32 = 0x1f;

pub struct State {
    // General Registers
    //  R13 - Stack Pointer (SP)
    //  R14 - Link Register (LR)
    //  R15 - Program Counter (PC)
    r: [u32; 16],

    // Banked Registers - Fast Interrupt (FIQ)
    r8_fiq: u32,
    r9_fiq: u32,
    r10_fiq: u32,
    r11_fiq: u32,
    r12_fiq: u32,
    r13_fiq: u32,
    r14_fiq: u32,

    // Banked Registers - Supervisor (SVC)
    r13_svc: u32,
    r14_svc: u32,

    // Banked Registers - Abort (ABT)
    r13_abt: u32,
    r14_abt: u32,

    // Banked Registers - Undefined (UND)
    r13_und: u32,
    r14_und: u32,

    // Banked Registers - Interrupt (IRQ)
    r13_irq: u32,
    r14_irq: u32,

    // Current Process Status Register (CPSR)
    pub cpsr: u32,

    // Whenever the CPU enters an exception, the current status register (CPSR) is copied to
    // the respective SPSR_<mode> register. Note that there is only one SPSR for each mode.

    // Saved Program Status Registers (SPSR)
    spsr_fiq: u32,
    spsr_svc: u32,
    spsr_abt: u32,
    spsr_irq: u32,
    spsr_und: u32,
}

impl State {
    pub fn r(&self, num: usize) -> u32 {
        debug_assert!(num <= 15);

        match self.cpsr & CPSR_MODE {
            CPSR_MODE_USR | CPSR_MODE_SYS => self.r[num],

            CPSR_MODE_FIQ => match num {
                8 => self.r8_fiq,
                9 => self.r9_fiq,
                10 => self.r10_fiq,
                11 => self.r11_fiq,
                12 => self.r12_fiq,
                13 => self.r13_fiq,
                14 => self.r14_fiq,

                _ => self.r[num],
            },

            CPSR_MODE_SVC => match num {
                13 => self.r13_svc,
                14 => self.r14_svc,

                _ => self.r[num],
            },

            CPSR_MODE_ABT => match num {
                13 => self.r13_abt,
                14 => self.r14_abt,

                _ => self.r[num],
            },

            CPSR_MODE_IRQ => match num {
                13 => self.r13_irq,
                14 => self.r14_irq,

                _ => self.r[num],
            },

            CPSR_MODE_UND => match num {
                13 => self.r13_und,
                14 => self.r14_und,

                _ => self.r[num],
            },

            _ => unreachable!(),
        }
    }

    pub fn set_r(&mut self, num: usize, value: u32) {
        debug_assert!(num <= 15);

        *(match self.cpsr & CPSR_MODE {
            CPSR_MODE_USR | CPSR_MODE_SYS => &mut self.r[num],

            CPSR_MODE_FIQ => match num {
                8 => &mut self.r8_fiq,
                9 => &mut self.r9_fiq,
                10 => &mut self.r10_fiq,
                11 => &mut self.r11_fiq,
                12 => &mut self.r12_fiq,
                13 => &mut self.r13_fiq,
                14 => &mut self.r14_fiq,

                _ => &mut self.r[num],
            },

            CPSR_MODE_SVC => match num {
                13 => &mut self.r13_svc,
                14 => &mut self.r14_svc,

                _ => &mut self.r[num],
            },

            CPSR_MODE_ABT => match num {
                13 => &mut self.r13_abt,
                14 => &mut self.r14_abt,

                _ => &mut self.r[num],
            },

            CPSR_MODE_IRQ => match num {
                13 => &mut self.r13_irq,
                14 => &mut self.r14_irq,

                _ => &mut self.r[num],
            },

            CPSR_MODE_UND => match num {
                13 => &mut self.r13_und,
                14 => &mut self.r14_und,

                _ => &mut self.r[num],
            },

            _ => unreachable!(),
        }) = value;
    }
}

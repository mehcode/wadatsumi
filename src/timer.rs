use ::mode;

#[derive(Default)]
pub struct Timer {
    /// Divider     Register (R/W) — $FF04
    /// This register is incremented at rate of 16384Hz (~16779Hz on SGB).
    /// In CGB Double Speed Mode it is incremented twice as fast, ie. at 32768Hz.
    /// Writing any value to this register resets it to 00h.
    div: u16,

    /// Previous value of DIV register
    /// Used to detect 1->0 transitions for the rest of the system
    div_last: u16,

    /// Timer Counter (R/W) — $FF05
    /// This timer is incremented by a clock frequency specified by the TAC
    /// register ($FF07). When the value overflows (gets bigger than FFh)
    /// then it will be reset to the value specified in TMA (FF06), and an
    /// interrupt will be requested.
    tima: u8,

    /// Timer Modulo (R/W) — $FF06
    /// When the TIMA overflows, this data will be loaded.
    tma: u8,

    /// Timer Control (R/W) — $FF07
    ///   Bit 2    - Timer Stop  (0=Stop, 1=Start)
    ///   Bits 1-0 - Input Clock Select
    ///            00:   4096 Hz    (~4194 Hz SGB)
    ///            01: 262144 Hz  (~268400 Hz SGB)
    ///            10:  65536 Hz   (~67110 Hz SGB)
    ///            11:  16384 Hz   (~16780 Hz SGB)
    tac: u8,
}

impl Timer {
    /// Reset
    pub fn reset(&mut self, m: mode::Mode) {
        self.div = match m {
            mode::GB_AGB => 0x2680,
            mode::GB_CGB => 0x267C,
            mode::CGB_CGB => 0x1EA0,
            mode::CGB_AGB => 0x1EA4,
            _ => 0xABCC,
        };

        self.tima = 0;
        self.tma = 0;
        self.tac = 0;
    }

    /// Step
    pub fn step(&mut self, if_: &mut u8) {
        // If we have the TAC enable bit set, then we need to check for a 1 - 0
        // conversion on a specific bit. This figures out which bit.
        //  1 -> b03
        //  2 -> b05
        //  3 -> b07
        //  0 -> b09
        let freq = (self.tac & 0b11) as u16;
        let b = if freq == 0 {
            0x200u16
        } else {
            0x1u16 << ((freq << 1) + 1)
        };

        // The machine is stepped each M-cycle and the GPU needs to be stepped each T-cycle
        for _ in 1..5 {
            // Remember the value of our watched bit on DIV
            let value = self.div & b;

            // Increment DIV
            self.div = self.div.wrapping_add(1);

            if (self.tac & 0b100) != 0 && value > 0 && (self.div & b) == 0 {
                // Check for 8-bit overflow
                if ((self.tima as u16) + 1) & 0xFF == 0 {
                    // Set the overflow sentinel
                    self.tima = self.tma;

                    // Flag the interrupt
                    (*if_) |= 0b100;
                } else {
                    // Increment TIMA
                    self.tima += 1;
                }
            }
        }
    }

    /// Read
    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => (self.tac | 0xF8),

            _ => {
                // Unhandled
                0xFF
            }
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                // Any value written to DIV is ignored and DIV is set to 0
                self.div = 0;
            }

            0xFF05 => {
                self.tima = value;
            }

            0xFF06 => {
                self.tma = value;
            }

            0xFF07 => {
                self.tac = value & 0b111;
            }

            _ => {
                // Unhandled
            }
        }
    }
}

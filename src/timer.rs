use ::mode;

#[derive(Default)]
pub struct Timer {
    /// Divider     Register (R/W) — $FF04
    /// This register is incremented at rate of 16384Hz (~16779Hz on SGB).
    /// In CGB Double Speed Mode it is incremented twice as fast, ie. at 32768Hz.
    /// Writing any value to this register resets it to 00h.
    pub div: u16,

    /// Previous value of DIV register
    /// Used to detect 1->0 transitions for the rest of the system
    pub div_last: u16,

    /// Timer Counter (R/W) — $FF05
    /// This timer is incremented by a clock frequency specified by the TAC
    /// register ($FF07). When the value overflows (gets bigger than FFh)
    /// then it will be reset to the value specified in TMA (FF06), and an
    /// interrupt will be requested.
    tima: u8,

    /// When TIMA is reloaded by TMA and then TMA is set within the same T-cycle; TIMA is set
    /// to the new value of TMA.
    tima_timer: u8,

    /// When TIMA overflows; there is a 4 T-cycle timer before something happens
    tima_reload_timer: u8,

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
        self.tima_reload_timer = 0;
    }

    /// Step
    pub fn step(&mut self) {
        // TIMA weird state (able to be set by loading TMA) lasts for 1 T-cycle
        if self.tima_timer > 0 {
            self.tima_timer -= 1;
        }

        // Check for a queued TIMA reload
        if self.tima_reload_timer > 0 {
            self.tima_reload_timer -= 1;
            if self.tima_reload_timer == 0 {
                // Reload TIMA (with TMA)
                self.tima = self.tma;

                // If TMA is set in this T-cycle, TIMA gets loaded with it
                self.tima_timer = 1;
            }
        }

        // Increment DIV (remember previous div)
        self.div_last = self.div;
        self.div = self.div.wrapping_add(1);
    }

    pub fn on_change_div(&mut self, prev: u16, cur: u16, if_: &mut u8) {
        if (self.tac & 0b100) != 0 {
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

            if (prev & b) != 0 && (cur & b) == 0 {
                // Check for 8-bit overflow
                if ((self.tima as u16) + 1) & 0xFF == 0 {
                    // Enqueue the TIMA reload
                    self.tima = 0;
                    self.tima_reload_timer = 4;

                    // Note that the timer interrupt is still fired here
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
    pub fn write(&mut self, address: u16, value: u8, if_: &mut u8) {
        match address {
            0xFF04 => {
                // Any value written to DIV is ignored and DIV is set to 0
                self.div_last = self.div;
                self.div = 0;
            }

            0xFF05 => {
                // If you write to TIMA during the cycle that TMA is being loaded to it,
                // the write will be ignored and TMA value will be written to TIMA instead.
                if self.tima_timer == 0 {
                    self.tima = value;

                    // Any other write to TIMA will reset a pending reload
                    self.tima_reload_timer = 0;
                }
            }

            0xFF06 => {
                self.tma = value;

                if self.tima_timer > 0 {
                    self.tima = self.tma;
                }
            }

            0xFF07 => {
                // When disabling the timer, if the corresponding bit in the system counter
                // is set to 1, the falling edge detector will see a change from 1 to 0,
                // so TIMA will increase. This means that whenever half the clocks of the
                // count are reached, TIMA will increase when disabling the timer.
                if (self.tac & 0b100 != 0) && (value & 0b100 == 0) {
                    // TIMA is being disabled
                    let div = self.div;
                    self.on_change_div(div, 0, if_);
                }

                self.tac = value & 0b111;
            }

            _ => {
                // Unhandled
            }
        }
    }
}

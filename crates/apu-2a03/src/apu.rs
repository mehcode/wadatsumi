pub struct Apu2A03 {
    seq: u16,
}

impl Default for Apu2A03 {
    fn default() -> Self {
        Self::new()
    }
}

impl Apu2A03 {
    pub fn new() -> Self {
        Self { seq: 0 }
    }

    pub fn tick(&mut self) {
        self.seq = self.seq.wrapping_add(1);

        match self.seq {
            7457 => {
                // TODO: Clock Envelopes
                // TODO: Clock Triangle Linear Counter
            }

            14913 => {
                // TODO: Clock Envelopes
                // TODO: Clock Triangle Linear Counter
                // TODO: Clock Length Counter
                // TODO: Clock Sweep Units
            }

            22371 => {
                // TODO: Clock Envelopes
                // TODO: Clock Triangle Linear Counter
            }

            29828 => {
                // TODO: Set frame interrupt if interrupt inhibit is clear
            }

            29829 => {
                // TODO: Clock Envelopes
                // TODO: Clock Triangle Linear Counter
                // TODO: Clock Length Counter
                // TODO: Clock Sweep Units
                // TODO: Set frame interrupt if interrupt inhibit is clear
            }

            29830 => {
                // TODO: Set frame interrupt if interrupt inhibit is clear
                self.seq = 0;
            }

            _ => {}
        }
    }
}

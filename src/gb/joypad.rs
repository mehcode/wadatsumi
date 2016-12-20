use sdl2::keyboard::Scancode;
use ::bits;

#[derive(Default)]
pub struct Joypad {
    /// [FF00] - P1/JOYP - Joypad (R/W)
    ///   Bit 5 - P15 Select Button Keys      (0=Select)
    sel_button: bool,

    /// [FF00] - P1/JOYP - Joypad (R/W)
    ///   Bit 4 - P14 Select Direction Keys   (0=Select)
    sel_direction: bool,

    /// [FF00] - P1/JOYP - Joypad (R/W)
    ///   Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
    st_down: bool,
    st_start: bool,

    /// [FF00] - P1/JOYP - Joypad (R/W)
    ///   Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
    st_up: bool,
    st_sel: bool,

    /// [FF00] - P1/JOYP - Joypad (R/W)
    ///    Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
    st_left: bool,
    st_b: bool,

    /// [FF00] - P1/JOYP - Joypad (R/W)
    ///   Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
    st_right: bool,
    st_a: bool,
}

impl Joypad {
    /// Reset
    pub fn reset(&mut self) {
        self.sel_button = true;
        self.sel_direction = true;
    }

    /// Read
    pub fn read(&mut self, address: u16) -> u8 {
        if address == 0xFF00 {
            (0xC0 | bits::bit(!self.sel_button, 5) | bits::bit(!self.sel_direction, 4) |
             bits::bit(!((self.sel_direction && self.st_down) ||
                         (self.sel_button && self.st_start)),
                       3) |
             bits::bit(!((self.sel_direction && self.st_up) || (self.sel_button && self.st_sel)),
                       2) |
             bits::bit(!((self.sel_direction && self.st_left) || (self.sel_button && self.st_b)),
                       1) |
             bits::bit(!((self.sel_direction && self.st_right) || (self.sel_button && self.st_a)),
                       0))
        } else {
            0xFF
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8) {
        if address == 0xFF00 {
            self.sel_button = !bits::test(value, 5);
            self.sel_direction = !bits::test(value, 4);
        }
    }

    fn on_key(&mut self, scancode: Scancode, pressed: bool) {
        match scancode {
            Scancode::Z => {
                self.st_a = pressed;
            }

            Scancode::X => {
                self.st_b = pressed;
            }

            Scancode::LShift => {
                self.st_sel = pressed;
            }

            Scancode::Return => {
                self.st_start = pressed;
            }

            Scancode::Up => {
                self.st_up = pressed;
            }

            Scancode::Down => {
                self.st_down = pressed;
            }

            Scancode::Left => {
                self.st_left = pressed;
            }

            Scancode::Right => {
                self.st_right = pressed;
            }

            _ => {}
        }
    }

    pub fn on_key_down(&mut self, scancode: Scancode) {
        self.on_key(scancode, true);
    }

    pub fn on_key_up(&mut self, scancode: Scancode) {
        self.on_key(scancode, false);
    }
}

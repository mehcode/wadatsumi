use ::cpu;

// Decrement 8-bit register
macro_rules! dec8 { ($c:ident, $x:ident) => ($c.$x -= 1); }

// 00 — NOP {1}
pub fn _00(_: &mut cpu::CPU) {
    // Do nothing
}

// 01 nn nn — LD BC, u16 {3}
pub fn _01(_: &mut cpu::CPU) {
    // TODO(gameboy)
}

// 05 — DEC B {1}
pub fn _05(c: &mut cpu::CPU) {
    dec8!(c, b);
}

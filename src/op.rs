use ::cpu;

// 00 — NOP {1}
pub fn _00(_: &mut cpu::CPU) {
    // Do nothing
}

// 01 nn nn — LD BC, u16 {3}
pub fn _01(c: &mut cpu::CPU) {}

// 05 — DEC B {1}
pub fn _05(c: &mut cpu::CPU) {
    c.B = om.dec8(&mut c.B);
}

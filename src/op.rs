use ::cpu;
use ::cpu::Context;
use ::bus::Bus;

// 00 — NOP {1}
pub fn _00(_: &mut Context, _: &mut Bus) {
    // Do nothing
}

// 01 nn nn — LD BC, u16 {3}
pub fn _01(c: &mut Context, b: &mut Bus) {
    let r = om_read_next16!(c, b);
    c.set_bc(r);
}

// 02 — LD (BC), A {2}
pub fn _02(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_bc(), c.a);
}

// 03 — INC BC {2}
pub fn _03(c: &mut Context, b: &mut Bus) {
    om_inc16!(c, b; get_bc, set_bc);
}

// 04 — INC B {1}
pub fn _04(c: &mut Context, _: &mut Bus) {
    om_inc8!(c; c.b);
}

// 05 — DEC B {1}
pub fn _05(c: &mut Context, _: &mut Bus) {
    om_dec8!(c; c.b);
}

// 06 nn — LD B, u8 {2}
pub fn _06(c: &mut Context, b: &mut Bus) {
    c.b = om_read_next8!(c, b);
}

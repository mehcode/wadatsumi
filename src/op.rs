#![allow(non_snake_case)]

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

// 07 — RLCA {1}
pub fn _07(c: &mut Context, _: &mut Bus) {
    om_rlca8!(c);
}

// 08 nn nn — LD (u16), SP {5}
pub fn _08(c: &mut Context, b: &mut Bus) {
    let address = om_read_next16!(c, b);
    om_write16!(c, b; address, c.sp);
}

// 09 — ADD HL, BC {2}
pub fn _09(c: &mut Context, b: &mut Bus) {
    om_add16_hl!(c, b; get_bc);
}

// 0A — LD A, (BC) {2}
pub fn _0A(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_bc());
    c.a = r;
}

// 0B — DEC BC {2}
pub fn _0B(c: &mut Context, b: &mut Bus) {
    om_dec16!(c, b; get_bc, set_bc);
}

// 0C — INC C {1}
pub fn _0C(c: &mut Context, _: &mut Bus) {
    om_inc8!(c; c.c);
}

// 0D — DEC C {1}
pub fn _0D(c: &mut Context, _: &mut Bus) {
    om_dec8!(c; c.c);
}

// 0E nn — LD C, u8 {2}
pub fn _0E(c: &mut Context, b: &mut Bus) {
    c.c = om_read_next8!(c, b);
}

// 0F — RRCA {1}
pub fn _0F(c: &mut Context, _: &mut Bus) {
    om_rrca8!(c);
}

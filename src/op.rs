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
    om_inc8_r!(c; b);
}

// 05 — DEC B {1}
pub fn _05(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; b);
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
    om_add16_hl!(c, b; c.get_bc());
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
    om_inc8_r!(c; c);
}

// 0D — DEC C {1}
pub fn _0D(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; c);
}

// 0E nn — LD C, u8 {2}
pub fn _0E(c: &mut Context, b: &mut Bus) {
    c.c = om_read_next8!(c, b);
}

// 0F — RRCA {1}
pub fn _0F(c: &mut Context, _: &mut Bus) {
    om_rrca8!(c);
}

// 10 — STOP
pub fn _10(_: &mut Context, _: &mut Bus) {
    // TODO: STOP
    warn!("unsupported: STOP");
}

// 11 nn nn — LD DE, u16 {3}
pub fn _11(c: &mut Context, b: &mut Bus) {
    let r = om_read_next16!(c, b);
    c.set_de(r);
}

// 12 — LD (DE), A {2}
pub fn _12(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_de(), c.a);
}

// 13 — INC DE {2}
pub fn _13(c: &mut Context, b: &mut Bus) {
    om_inc16!(c, b; get_de, set_de);
}

// 14 — INC D {1}
pub fn _14(c: &mut Context, _: &mut Bus) {
    om_inc8_r!(c; d);
}

// 15 — DEC D {1}
pub fn _15(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; d);
}

// 16 nn — LD D, u8 {2}
pub fn _16(c: &mut Context, b: &mut Bus) {
    c.d = om_read_next8!(c, b);
}

// 17 — RLA {1}
pub fn _17(c: &mut Context, _: &mut Bus) {
    om_rla8!(c);
}

// 18 nn — JR i8 {3}
pub fn _18(c: &mut Context, b: &mut Bus) {
    om_jr!(c, b);
}

// 19 — ADD HL, DE {2}
pub fn _19(c: &mut Context, b: &mut Bus) {
    om_add16_hl!(c, b; c.get_de());
}

// 1A — LD A, (DE) {2}
pub fn _1A(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_de());
    c.a = r;
}

// 1B — DEC DE {2}
pub fn _1B(c: &mut Context, b: &mut Bus) {
    om_dec16!(c, b; get_de, set_de);
}

// 1C — INC E {1}
pub fn _1C(c: &mut Context, _: &mut Bus) {
    om_inc8_r!(c; e);
}

// 1D — DEC E {1}
pub fn _1D(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; e);
}

// 1E nn — LD E, u8 {2}
pub fn _1E(c: &mut Context, b: &mut Bus) {
    c.e = om_read_next8!(c, b);
}

// 1F — RRA {1}
pub fn _1F(c: &mut Context, _: &mut Bus) {
    om_rra8!(c);
}

// 20 nn — JR NZ, i8 {3/2}
pub fn _20(c: &mut Context, b: &mut Bus) {
    om_jr_unless!(c, b; cpu::ZERO);
}

// 21 nn nn — LD HL, u16 {3}
pub fn _21(c: &mut Context, b: &mut Bus) {
    let r = om_read_next16!(c, b);
    c.set_hl(r);
}

// 22 — LDI (HL), A {2}
pub fn _22(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_write8!(c, b; hl, c.a);

    c.set_hl(hl + 1);
}

// 23 — INC HL {2}
pub fn _23(c: &mut Context, b: &mut Bus) {
    om_inc16!(c, b; get_hl, set_hl);
}

// 24 — INC H {1}
pub fn _24(c: &mut Context, _: &mut Bus) {
    om_inc8_r!(c; h);
}

// 25 — DEC H {1}
pub fn _25(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; h);
}

// 26 nn — LD H, u8 {2}
pub fn _26(c: &mut Context, b: &mut Bus) {
    c.h = om_read_next8!(c, b);
}

// 27 — DAA {1}
pub fn _27(c: &mut Context, _: &mut Bus) {
    // REF: http://stackoverflow.com/a/29990058
    //
    // When this instruction is executed, the A register is BCD corrected
    // using the contents of the flags. The exact process is the following:
    // if the least significant four bits of A contain a non-BCD digit (i. e.
    // it is greater than 9) or the H flag is set, then $06 is added to the
    // register. Then the four most significant bits are checked. If this
    // more significant digit also happens to be greater than 9 or the C
    // flag is set, then $60 is added.
    //
    // If the N flag is set, subtract instead of add.
    //
    // If the lower 4 bits form a number greater than 9 or H is set,
    // add $06 to the accumulator

    let mut r = c.a as u16;
    let mut correction = if c.f.contains(cpu::CARRY) {
        0x60u16
    } else {
        0x00u16
    };

    if c.f.contains(cpu::HALF_CARRY) || ((!c.f.contains(cpu::ADD_SUBTRACT)) && ((r & 0x0F) > 9)) {
        correction |= 0x06;
    }

    if c.f.contains(cpu::CARRY) || ((!c.f.contains(cpu::ADD_SUBTRACT)) && (r > 0x99)) {
        correction |= 0x60;
    }

    if c.f.contains(cpu::ADD_SUBTRACT) {
        r -= correction;
    } else {
        r += correction;
    }

    if ((correction << 2) & 0x100) != 0 {
        c.set_flag(cpu::CARRY, true);
    }

    // Half-carry is always unset (unlike a Z-80)
    c.set_flag(cpu::HALF_CARRY, false);
    c.set_flag(cpu::ZERO, (r & 0xFF) == 0);

    c.a = (r & 0xFF) as u8;
}

// 28 nn — JR Z, i8 {3/2}
pub fn _28(c: &mut Context, b: &mut Bus) {
    om_jr_if!(c, b; cpu::ZERO);
}

// 29 — ADD HL, HL {2}
pub fn _29(c: &mut Context, b: &mut Bus) {
    om_add16_hl!(c, b; c.get_hl());
}

// 2A — LDI A, (HL) {2}
pub fn _2A(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    let r = om_read8!(c, b; hl);

    c.a = r;
    c.set_hl(hl + 1);
}

// 2B — DEC HL {2}
pub fn _2B(c: &mut Context, b: &mut Bus) {
    om_dec16!(c, b; get_hl, set_hl);
}

// 2C — INC L {1}
pub fn _2C(c: &mut Context, _: &mut Bus) {
    om_inc8_r!(c; l);
}

// 2D — DEC L {1}
pub fn _2D(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; l);
}

// 2E nn — LD L, u8 {2}
pub fn _2E(c: &mut Context, b: &mut Bus) {
    c.l = om_read_next8!(c, b);
}

// 2F — CPL {1}
pub fn _2F(c: &mut Context, _: &mut Bus) {
    c.a ^= 0xFF;

    c.set_flag(cpu::ADD_SUBTRACT, true);
    c.set_flag(cpu::HALF_CARRY, true);
}

// 30 nn nn — JR NC, u16 {3/2}
pub fn _30(c: &mut Context, b: &mut Bus) {
    om_jr_unless!(c, b; cpu::CARRY);
}

// 31 nn nn — LD SP, u16 {3}
pub fn _31(c: &mut Context, b: &mut Bus) {
    let r = om_read_next16!(c, b);
    c.sp = r;
}

// 32 — LDD (HL), A {2}
pub fn _32(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_write8!(c, b; hl, c.a);

    c.set_hl(hl - 1);
}

// 33 — INC SP {2}
pub fn _33(c: &mut Context, b: &mut Bus) {
    c.sp += 1;
    c.step(b);
}

// 34 — INC (HL) {3}
pub fn _34(c: &mut Context, b: &mut Bus) {
    let address = c.get_hl();
    let mut r = om_read8!(c, b; address);
    r = om_inc8!(c; r);

    om_write8!(c, b; address, r);
}

// 35 — DEC (HL) {3}
pub fn _35(c: &mut Context, b: &mut Bus) {
    let address = c.get_hl();
    let mut r = om_read8!(c, b; address);
    r = om_dec8!(c; r);

    om_write8!(c, b; address, r);
}

// 36 nn — LD (HL), u8 {3}
pub fn _36(c: &mut Context, b: &mut Bus) {
    let address = c.get_hl();
    let r = om_read_next8!(c, b);

    om_write8!(c, b; address, r);
}

// 37 — SCF {1}
pub fn _37(c: &mut Context, _: &mut Bus) {
    c.set_flag(cpu::CARRY, true);
    c.set_flag(cpu::HALF_CARRY, false);
    c.set_flag(cpu::ADD_SUBTRACT, false);
}

// 38 nn nn — JR C, u16 {3/2}
pub fn _38(c: &mut Context, b: &mut Bus) {
    om_jr_if!(c, b; cpu::CARRY);
}

// 39 — ADD HL, SP {2}
pub fn _39(c: &mut Context, b: &mut Bus) {
    om_add16_hl!(c, b; c.sp);
}

// 3A — LDD A, (HL) {2}
pub fn _3A(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    let r = om_read8!(c, b; hl);

    c.a = r;
    c.set_hl(hl - 1);
}

// 3B — DEC SP {2}
pub fn _3B(c: &mut Context, b: &mut Bus) {
    c.sp -= 1;
    c.step(b);
}

// 3C — INC A {1}
pub fn _3C(c: &mut Context, _: &mut Bus) {
    om_inc8_r!(c; a);
}

// 3D — DEC A {1}
pub fn _3D(c: &mut Context, _: &mut Bus) {
    om_dec8_r!(c; a);
}

// 3E nn — LD A, u8 {2}
pub fn _3E(c: &mut Context, b: &mut Bus) {
    c.a = om_read_next8!(c, b);
}

// 3F — CCF {1}
pub fn _3F(c: &mut Context, _: &mut Bus) {
    let carry = c.test_flag(cpu::CARRY);
    c.set_flag(cpu::CARRY, !carry);
    c.set_flag(cpu::HALF_CARRY, false);
    c.set_flag(cpu::ADD_SUBTRACT, false);
}

// 40 — LD B, B {1}
pub fn _40(c: &mut Context, _: &mut Bus) {
    c.b = c.b;
}

// 41 — LD B, C {1}
pub fn _41(c: &mut Context, _: &mut Bus) {
    c.b = c.c;
}

// 42 — LD B, D {1}
pub fn _42(c: &mut Context, _: &mut Bus) {
    c.b = c.d;
}

// 43 — LD B, E {1}
pub fn _43(c: &mut Context, _: &mut Bus) {
    c.b = c.e;
}

// 44 — LD B, H {1}
pub fn _44(c: &mut Context, _: &mut Bus) {
    c.b = c.h;
}

// 45 — LD B, L {1}
pub fn _45(c: &mut Context, _: &mut Bus) {
    c.b = c.l;
}

// 46 — LD B, (HL) {2}
pub fn _46(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.b = r;
}

// 47 — LD B, A {1}
pub fn _47(c: &mut Context, _: &mut Bus) {
    c.b = c.a;
}

// 48 — LD C, B {1}
pub fn _48(c: &mut Context, _: &mut Bus) {
    c.c = c.b;
}

// 49 — LD C, C {1}
pub fn _49(c: &mut Context, _: &mut Bus) {
    c.c = c.c;
}

// 4A — LD C, D {1}
pub fn _4A(c: &mut Context, _: &mut Bus) {
    c.c = c.d;
}

// 4B — LD C, E {1}
pub fn _4B(c: &mut Context, _: &mut Bus) {
    c.c = c.e;
}

// 4C — LD C, H {1}
pub fn _4C(c: &mut Context, _: &mut Bus) {
    c.c = c.h;
}

// 4D — LD C, L {1}
pub fn _4D(c: &mut Context, _: &mut Bus) {
    c.c = c.l;
}

// 4E — LD C, (HL) {2}
pub fn _4E(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.c = r;
}

// 4F — LD C, A {1}
pub fn _4F(c: &mut Context, _: &mut Bus) {
    c.c = c.a;
}

// 50 — LD D, B {1}
pub fn _50(c: &mut Context, _: &mut Bus) {
    c.d = c.b;
}

// 51 — LD D, C {1}
pub fn _51(c: &mut Context, _: &mut Bus) {
    c.d = c.c;
}

// 52 — LD D, D {1}
pub fn _52(c: &mut Context, _: &mut Bus) {
    c.d = c.d;
}

// 53 — LD D, E {1}
pub fn _53(c: &mut Context, _: &mut Bus) {
    c.d = c.e;
}

// 54 — LD D, H {1}
pub fn _54(c: &mut Context, _: &mut Bus) {
    c.d = c.h;
}

// 55 — LD D, L {1}
pub fn _55(c: &mut Context, _: &mut Bus) {
    c.d = c.l;
}

// 56 — LD D, (HL) {2}
pub fn _56(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.d = r;
}

// 57 — LD D, A {1}
pub fn _57(c: &mut Context, _: &mut Bus) {
    c.d = c.a;
}

// 58 — LD E, B {1}
pub fn _58(c: &mut Context, _: &mut Bus) {
    c.e = c.b;
}

// 59 — LD E, C {1}
pub fn _59(c: &mut Context, _: &mut Bus) {
    c.e = c.c;
}

// 5A — LD E, D {1}
pub fn _5A(c: &mut Context, _: &mut Bus) {
    c.e = c.d;
}

// 5B — LD E, E {1}
pub fn _5B(c: &mut Context, _: &mut Bus) {
    c.e = c.e;
}

// 5C — LD E, H {1}
pub fn _5C(c: &mut Context, _: &mut Bus) {
    c.e = c.h;
}

// 5D — LD E, L {1}
pub fn _5D(c: &mut Context, _: &mut Bus) {
    c.e = c.l;
}

// 5E — LD E, (HL) {2}
pub fn _5E(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.e = r;
}

// 5F — LD E, A {1}
pub fn _5F(c: &mut Context, _: &mut Bus) {
    c.e = c.a;
}

// 60 — LD H, B {1}
pub fn _60(c: &mut Context, _: &mut Bus) {
    c.h = c.b;
}

// 61 — LD H, C {1}
pub fn _61(c: &mut Context, _: &mut Bus) {
    c.h = c.c;
}

// 62 — LD H, D {1}
pub fn _62(c: &mut Context, _: &mut Bus) {
    c.h = c.d;
}

// 63 — LD H, E {1}
pub fn _63(c: &mut Context, _: &mut Bus) {
    c.h = c.e;
}

// 64 — LD H, H {1}
pub fn _64(c: &mut Context, _: &mut Bus) {
    c.h = c.h;
}

// 65 — LD H, L {1}
pub fn _65(c: &mut Context, _: &mut Bus) {
    c.h = c.l;
}

// 66 — LD H, (HL) {2}
pub fn _66(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.h = r;
}

// 67 — LD H, A {1}
pub fn _67(c: &mut Context, _: &mut Bus) {
    c.h = c.a;
}

// 68 — LD L, B {1}
pub fn _68(c: &mut Context, _: &mut Bus) {
    c.l = c.b;
}

// 69 — LD L, C {1}
pub fn _69(c: &mut Context, _: &mut Bus) {
    c.l = c.c;
}

// 6A — LD L, D {1}
pub fn _6A(c: &mut Context, _: &mut Bus) {
    c.l = c.d;
}

// 6B — LD L, E {1}
pub fn _6B(c: &mut Context, _: &mut Bus) {
    c.l = c.e;
}

// 6C — LD L, H {1}
pub fn _6C(c: &mut Context, _: &mut Bus) {
    c.l = c.h;
}

// 6D — LD L, L {1}
pub fn _6D(c: &mut Context, _: &mut Bus) {
    c.l = c.l;
}

// 6E — LD L, (HL) {2}
pub fn _6E(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.l = r;
}

// 6F — LD L, A {1}
pub fn _6F(c: &mut Context, _: &mut Bus) {
    c.l = c.a;
}

// 70 — LD (HL), B {2}
pub fn _70(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.b);
}

// 71 — LD (HL), C {2}
pub fn _71(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.c);
}

// 72 — LD (HL), D {2}
pub fn _72(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.d);
}

// 73 — LD (HL), E {2}
pub fn _73(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.e);
}

// 74 — LD (HL), H {2}
pub fn _74(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.h);
}

// 75 — LD (HL), L {2}
pub fn _75(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.l);
}

// 76 — HALT
pub fn _76(c: &mut Context, _: &mut Bus) {
    // If IME is NOT enabled but IE/IF indicate there is a pending interrupt;
    // set HALT to a funny state that will cause us to 'replay' the next
    // opcode
    c.halt = if (c.ime == 0) && (c.ie & c.if_ & 0x1F) != 0 {
        -1
    } else {
        1
    };
}

// 77 — LD (HL), A {2}
pub fn _77(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; c.get_hl(), c.a);
}

// 78 — LD A, B {1}
pub fn _78(c: &mut Context, _: &mut Bus) {
    c.a = c.b;
}

// 79 — LD A, C {1}
pub fn _79(c: &mut Context, _: &mut Bus) {
    c.a = c.c;
}

// 7A — LD A, D {1}
pub fn _7A(c: &mut Context, _: &mut Bus) {
    c.a = c.d;
}

// 7B — LD A, E {1}
pub fn _7B(c: &mut Context, _: &mut Bus) {
    c.a = c.e;
}

// 7C — LD A, H {1}
pub fn _7C(c: &mut Context, _: &mut Bus) {
    c.a = c.h;
}

// 7D — LD A, L {1}
pub fn _7D(c: &mut Context, _: &mut Bus) {
    c.a = c.l;
}

// 7E — LD A, (HL) {2}
pub fn _7E(c: &mut Context, b: &mut Bus) {
    let r = om_read8!(c, b; c.get_hl());
    c.a = r;
}

// 7F — LD A, A {1}
pub fn _7F(c: &mut Context, _: &mut Bus) {
    c.a = c.a;
}

// 80 — ADD A, B {1}
pub fn _80(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.b);
}

// 81 — ADD A, C {1}
pub fn _81(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.c);
}

// 82 — ADD A, D {1}
pub fn _82(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.d);
}

// 83 — ADD A, E {1}
pub fn _83(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.e);
}

// 84 — ADD A, H {1}
pub fn _84(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.h);
}

// 85 — ADD A, L {1}
pub fn _85(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.l);
}

// 86 — ADD A, (HL) {2}
pub fn _86(c: &mut Context, b: &mut Bus) {
    om_add8_a!(c; om_read8!(c, b; c.get_hl()));
}

// 87 — ADD A, A {1}
pub fn _87(c: &mut Context, _: &mut Bus) {
    om_add8_a!(c; c.a);
}

// 88 — ADC A, B {1}
pub fn _88(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.b);
}

// 89 — ADC A, C {1}
pub fn _89(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.c);
}

// 8A — ADC A, D {1}
pub fn _8A(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.d);
}

// 8B — ADC A, E {1}
pub fn _8B(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.e);
}

// 8C — ADC A, H {1}
pub fn _8C(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.h);
}

// 8D — ADC A, L {1}
pub fn _8D(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.l);
}

// 8E — ADC A, (HL) {2}
pub fn _8E(c: &mut Context, b: &mut Bus) {
    om_adc8_a!(c; om_read8!(c, b; c.get_hl()));
}

// 8F — ADC A, A {1}
pub fn _8F(c: &mut Context, _: &mut Bus) {
    om_adc8_a!(c; c.a);
}

// 90 — SUB A, B {1}
pub fn _90(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.b);
}

// 91 — SUB A, C {1}
pub fn _91(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.c);
}

// 92 — SUB A, D {1}
pub fn _92(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.d);
}

// 93 — SUB A, E {1}
pub fn _93(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.e);
}

// 94 — SUB A, H {1}
pub fn _94(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.h);
}

// 95 — SUB A, L {1}
pub fn _95(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.l);
}

// 96 — SUB A, (HL) {2}
pub fn _96(c: &mut Context, b: &mut Bus) {
    om_sub8_a!(c; om_read8!(c, b; c.get_hl()));
}

// 97 — SUB A, A {1}
pub fn _97(c: &mut Context, _: &mut Bus) {
    om_sub8_a!(c; c.a);
}

// 98 — SBC A, B {1}
pub fn _98(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.b);
}

// 99 — SBC A, C {1}
pub fn _99(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.c);
}

// 9A — SBC A, D {1}
pub fn _9A(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.d);
}

// 9B — SBC A, E {1}
pub fn _9B(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.e);
}

// 9C — SBC A, H {1}
pub fn _9C(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.h);
}

// 9D — SBC A, L {1}
pub fn _9D(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.l);
}

// 9E — SBC A, (HL) {2}
pub fn _9E(c: &mut Context, b: &mut Bus) {
    om_sbc8_a!(c; om_read8!(c, b; c.get_hl()));
}

// 9F — SBC A, A {1}
pub fn _9F(c: &mut Context, _: &mut Bus) {
    om_sbc8_a!(c; c.a);
}

// A0 — AND A, B {1}
pub fn _A0(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.b);
}

// A1 — AND A, C {1}
pub fn _A1(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.c);
}

// A2 — AND A, D {1}
pub fn _A2(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.d);
}

// A3 — AND A, E {1}
pub fn _A3(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.e);
}

// A4 — AND A, H {1}
pub fn _A4(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.h);
}

// A5 — AND A, L {1}
pub fn _A5(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.l);
}

// A6 — AND A, (HL) {2}
pub fn _A6(c: &mut Context, b: &mut Bus) {
    om_and8_a!(c; om_read8!(c, b; c.get_hl()));
}

// A7 — AND A, A {1}
pub fn _A7(c: &mut Context, _: &mut Bus) {
    om_and8_a!(c; c.a);
}

// A8 — XOR A, B {1}
pub fn _A8(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.b);
}

// A9 — XOR A, C {1}
pub fn _A9(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.c);
}

// AA — XOR A, D {1}
pub fn _AA(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.d);
}

// AB — XOR A, E {1}
pub fn _AB(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.e);
}

// AC — XOR A, H {1}
pub fn _AC(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.h);
}

// AD — XOR A, L {1}
pub fn _AD(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.l);
}

// AE — XOR A, (HL) {2}
pub fn _AE(c: &mut Context, b: &mut Bus) {
    om_xor8_a!(c; om_read8!(c, b; c.get_hl()));
}

// AF — XOR A, A {1}
pub fn _AF(c: &mut Context, _: &mut Bus) {
    om_xor8_a!(c; c.a);
}

// B0 — OR A, B {1}
pub fn _B0(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.b);
}

// B1 — OR A, C {1}
pub fn _B1(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.c);
}

// B2 — OR A, D {1}
pub fn _B2(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.d);
}

// B3 — OR A, E {1}
pub fn _B3(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.e);
}

// B4 — OR A, H {1}
pub fn _B4(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.h);
}

// B5 — OR A, L {1}
pub fn _B5(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.l);
}

// B6 — OR A, (HL) {2}
pub fn _B6(c: &mut Context, b: &mut Bus) {
    om_or8_a!(c; om_read8!(c, b; c.get_hl()));
}

// B7 — OR A, A {1}
pub fn _B7(c: &mut Context, _: &mut Bus) {
    om_or8_a!(c; c.a);
}

// B8 — CP A, B {1}
pub fn _B8(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.b);
}

// B9 — CP A, C {1}
pub fn _B9(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.c);
}

// BA — CP A, D {1}
pub fn _BA(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.d);
}

// BB — CP A, E {1}
pub fn _BB(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.e);
}

// BC — CP A, H {1}
pub fn _BC(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.h);
}

// BD — CP A, L {1}
pub fn _BD(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.l);
}

// BE — CP A, (HL) {2}
pub fn _BE(c: &mut Context, b: &mut Bus) {
    om_cp8_a!(c; om_read8!(c, b; c.get_hl()));
}

// BF — CP A, A {1}
pub fn _BF(c: &mut Context, _: &mut Bus) {
    om_cp8_a!(c; c.a);
}

// C0 — RET NZ {5/2}
pub fn _C0(c: &mut Context, b: &mut Bus) {
    om_ret_unless!(c, b; cpu::ZERO);
}

// C1 — POP BC {3}
pub fn _C1(c: &mut Context, b: &mut Bus) {
    let r = om_pop16!(c, b);
    c.set_bc(r);
}

// C2 nn nn — JP NZ, u16 {4/3}
pub fn _C2(c: &mut Context, b: &mut Bus) {
    om_jp_unless!(c, b; cpu::ZERO);
}

// C3 nn nn — JP u16 {4}
pub fn _C3(c: &mut Context, b: &mut Bus) {
    om_jp!(c, b);
}

// C4 nn nn — CALL NZ, u16 {6/3}
pub fn _C4(c: &mut Context, b: &mut Bus) {
    om_call_unless!(c, b; cpu::ZERO);
}

// C5 — PUSH BC {4}
pub fn _C5(c: &mut Context, b: &mut Bus) {
    om_push16!(c, b; c.get_bc());
}

// C6 nn — ADD A, u8 {2}
pub fn _C6(c: &mut Context, b: &mut Bus) {
    om_add8_a!(c; om_read_next8!(c, b));
}

// C7 — RST $00 {4}
pub fn _C7(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x00);
}

// C8 — RET Z {5/2}
pub fn _C8(c: &mut Context, b: &mut Bus) {
    om_ret_if!(c, b; cpu::ZERO);
}

// C9 — RET {4}
pub fn _C9(c: &mut Context, b: &mut Bus) {
    om_ret!(c, b);
}

// CA nn nn — JP Z, u16 {4/3}
pub fn _CA(c: &mut Context, b: &mut Bus) {
    om_jp_if!(c, b; cpu::ZERO);
}

// CC nn nn — CALL Z, u16 {6/3}
pub fn _CC(c: &mut Context, b: &mut Bus) {
    om_call_if!(c, b; cpu::ZERO);
}

// CD nn nn — CALL u16 {6}
pub fn _CD(c: &mut Context, b: &mut Bus) {
    om_call!(c, b);
}

// CE nn — ADC A, u8 {2}
pub fn _CE(c: &mut Context, b: &mut Bus) {
    om_adc8_a!(c; om_read_next8!(c, b));
}

// CF — RST $08 {4}
pub fn _CF(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x08);
}

// D0 — RET NC {5/2}
pub fn _D0(c: &mut Context, b: &mut Bus) {
    om_ret_unless!(c, b; cpu::CARRY);
}

// D1 — POP DE {3}
pub fn _D1(c: &mut Context, b: &mut Bus) {
    let r = om_pop16!(c, b);
    c.set_de(r);
}

// D2 nn nn — JP NC, u16 {4/3}
pub fn _D2(c: &mut Context, b: &mut Bus) {
    om_jp_unless!(c, b; cpu::CARRY);
}

// D4 nn nn — CALL NC, u16 {6/3}
pub fn _D4(c: &mut Context, b: &mut Bus) {
    om_call_unless!(c, b; cpu::CARRY);
}

// D5 — PUSH DE {4}
pub fn _D5(c: &mut Context, b: &mut Bus) {
    om_push16!(c, b; c.get_de());
}

// D6 nn — SUB A, u8 {2}
pub fn _D6(c: &mut Context, b: &mut Bus) {
    om_sub8_a!(c; om_read_next8!(c, b));
}

// D7 — RST $10 {4}
pub fn _D7(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x10);
}

// D8 — RET C {5/2}
pub fn _D8(c: &mut Context, b: &mut Bus) {
    om_ret_if!(c, b; cpu::CARRY);
}

// D9 — RETI {4}
pub fn _D9(c: &mut Context, b: &mut Bus) {
    om_ret!(c, b);
    c.ime = 1;
}

// DA nn nn — JP C, u16 {4/3}
pub fn _DA(c: &mut Context, b: &mut Bus) {
    om_jp_if!(c, b; cpu::CARRY);
}

// DC nn nn — CALL C, u16 {6/3}
pub fn _DC(c: &mut Context, b: &mut Bus) {
    om_call_if!(c, b; cpu::CARRY);
}

// DE nn — SBC A, u8 {2}
pub fn _DE(c: &mut Context, b: &mut Bus) {
    om_sbc8_a!(c; om_read_next8!(c, b));
}

// DF — RST $18 {4}
pub fn _DF(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x18);
}

// E0 — LD ($FF00 + n), A {3}
pub fn _E0(c: &mut Context, b: &mut Bus) {
    let address = 0xFF00 + om_read_next8!(c, b) as u16;
    om_write8!(c, b; address, c.a);
}

// E1 — POP HL {3}
pub fn _E1(c: &mut Context, b: &mut Bus) {
    let r = om_pop16!(c, b);
    c.set_hl(r);
}

// E2 — LD ($FF00 + C), A {2}
pub fn _E2(c: &mut Context, b: &mut Bus) {
    om_write8!(c, b; 0xFF00 + c.c as u16, c.a);
}

// E5 — PUSH HL {4}
pub fn _E5(c: &mut Context, b: &mut Bus) {
    om_push16!(c, b; c.get_hl());
}

// E6 nn — AND A, u8 {2}
pub fn _E6(c: &mut Context, b: &mut Bus) {
    om_and8_a!(c; om_read_next8!(c, b));
}

// E7 — RST $20 {4}
pub fn _E7(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x20);
}

// E8 nn — ADD SP, i8 {4}
pub fn _E8(c: &mut Context, b: &mut Bus) {
    c.sp = om_add16_sp!(c, b; om_read_next8!(c, b));
    c.step(b);
}

// E9 — JP HL {1}
pub fn _E9(c: &mut Context, _: &mut Bus) {
    c.pc = c.get_hl();
}

// EA nn nn — LD (u16), A {4}
pub fn _EA(c: &mut Context, b: &mut Bus) {
    let address = om_read_next16!(c, b);
    om_write8!(c, b; address, c.a);
}

// EE nn — XOR A, u8 {2}
pub fn _EE(c: &mut Context, b: &mut Bus) {
    om_xor8_a!(c; om_read_next8!(c, b));
}

// EF — RST $28 {4}
pub fn _EF(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x28);
}

// F0 — LD A, ($FF00 + n) {3}
pub fn _F0(c: &mut Context, b: &mut Bus) {
    let address = 0xFF00 + om_read_next8!(c, b) as u16;
    c.a = om_read8!(c, b; address);
}

// F1 — POP AF {3}
pub fn _F1(c: &mut Context, b: &mut Bus) {
    let r = om_pop16!(c, b);
    c.set_af(r);
}

// F2 — LD A, ($FF00 + C) {2}
pub fn _F2(c: &mut Context, b: &mut Bus) {
    c.a = om_read8!(c, b; 0xFF00 + c.c as u16);
}

// F3 — DI {1}
pub fn _F3(c: &mut Context, _: &mut Bus) {
    c.ime = 0;
}

// F5 — PUSH AF {4}
pub fn _F5(c: &mut Context, b: &mut Bus) {
    om_push16!(c, b; c.get_af());
}

// F6 nn — OR A, u8 {2}
pub fn _F6(c: &mut Context, b: &mut Bus) {
    om_or8_a!(c; om_read_next8!(c, b));
}

// F7 — RST $30 {4}
pub fn _F7(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x30);
}

// F8 nn — LD HL, SP + i8 {3}
pub fn _F8(c: &mut Context, b: &mut Bus) {
    let r = om_add16_sp!(c, b; om_read_next8!(c, b));
    c.set_hl(r);
}

// F9 — LD SP, HL {2}
pub fn _F9(c: &mut Context, b: &mut Bus) {
    c.sp = c.get_hl();
    c.step(b);
}

// FA nn nn — LD A, (u16) {4}
pub fn _FA(c: &mut Context, b: &mut Bus) {
    let address = om_read_next16!(c, b);
    c.a = om_read8!(c, b; address);
}

// FB — EI {1}
pub fn _FB(c: &mut Context, _: &mut Bus) {
    // -1 - PENDING (will set to 1 just before next instruction
    //               but after the interrupt check)
    c.ime = -1;
}

// FE nn — CP u8 {2}
pub fn _FE(c: &mut Context, b: &mut Bus) {
    om_cp8_a!(c; om_read_next8!(c, b));
}

// FF — RST $38 {4}
pub fn _FF(c: &mut Context, b: &mut Bus) {
    om_rst!(c, b; 0x38);
}

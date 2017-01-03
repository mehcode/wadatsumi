#![allow(non_snake_case)]

use gb::cpu;
use gb::cpu::Context;
use gb::bus::Bus;

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

    c.set_hl(hl.wrapping_add(1));
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
        r = r.wrapping_sub(correction);
    } else {
        r = r.wrapping_add(correction);
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
    c.set_hl(hl.wrapping_add(1));
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

    c.set_hl(hl.wrapping_sub(1));
}

// 33 — INC SP {2}
pub fn _33(c: &mut Context, b: &mut Bus) {
    c.sp = c.sp.wrapping_add(1);
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
    c.set_hl(hl.wrapping_sub(1));
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
pub fn _76(c: &mut Context, b: &mut Bus) {
    // If IME is NOT enabled but IE/IF indicate there is a pending interrupt;
    // set HALT to a funny state that will cause us to 'replay' the next
    // opcode
    c.halt = if (c.ime == 0) && (b.ie & b.if_ & 0x1F) != 0 {
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

// CB 00 — RLC B {2}
pub fn _CB_00(c: &mut Context, _: &mut Bus) {
    c.b = om_rlc8!(c; c.b);
}

// CB 01 — RLC C {2}
pub fn _CB_01(c: &mut Context, _: &mut Bus) {
    c.c = om_rlc8!(c; c.c);
}

// CB 02 — RLC D {2}
pub fn _CB_02(c: &mut Context, _: &mut Bus) {
    c.d = om_rlc8!(c; c.d);
}

// CB 03 — RLC E {2}
pub fn _CB_03(c: &mut Context, _: &mut Bus) {
    c.e = om_rlc8!(c; c.e);
}

// CB 04 — RLC H {2}
pub fn _CB_04(c: &mut Context, _: &mut Bus) {
    c.h = om_rlc8!(c; c.h);
}

// CB 05 — RLC L {2}
pub fn _CB_05(c: &mut Context, _: &mut Bus) {
    c.l = om_rlc8!(c; c.l);
}

// CB 06 — RLC (HL) {3}
pub fn _CB_06(c: &mut Context, b: &mut Bus) {
    let r = om_rlc8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 07 — RLC A {2}
pub fn _CB_07(c: &mut Context, _: &mut Bus) {
    c.a = om_rlc8!(c; c.a);
}

// CB 08 — RRC B {2}
pub fn _CB_08(c: &mut Context, _: &mut Bus) {
    c.b = om_rrc8!(c; c.b);
}

// CB 09 — RRC C {2}
pub fn _CB_09(c: &mut Context, _: &mut Bus) {
    c.c = om_rrc8!(c; c.c);
}

// CB 0A — RRC D {2}
pub fn _CB_0A(c: &mut Context, _: &mut Bus) {
    c.d = om_rrc8!(c; c.d);
}

// CB 0B — RRC E {2}
pub fn _CB_0B(c: &mut Context, _: &mut Bus) {
    c.e = om_rrc8!(c; c.e);
}

// CB 0C — RRC H {2}
pub fn _CB_0C(c: &mut Context, _: &mut Bus) {
    c.h = om_rrc8!(c; c.h);
}

// CB 0D — RRC L {2}
pub fn _CB_0D(c: &mut Context, _: &mut Bus) {
    c.l = om_rrc8!(c; c.l);
}

// CB 0E — RRC (HL) {3}
pub fn _CB_0E(c: &mut Context, b: &mut Bus) {
    let r = om_rrc8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 0F — RRC A {2}
pub fn _CB_0F(c: &mut Context, _: &mut Bus) {
    c.a = om_rrc8!(c; c.a);
}

// CB 10 — RL B {2}
pub fn _CB_10(c: &mut Context, _: &mut Bus) {
    c.b = om_rl8!(c; c.b);
}

// CB 11 — RL C {2}
pub fn _CB_11(c: &mut Context, _: &mut Bus) {
    c.c = om_rl8!(c; c.c);
}

// CB 12 — RL D {2}
pub fn _CB_12(c: &mut Context, _: &mut Bus) {
    c.d = om_rl8!(c; c.d);
}

// CB 13 — RL E {2}
pub fn _CB_13(c: &mut Context, _: &mut Bus) {
    c.e = om_rl8!(c; c.e);
}

// CB 14 — RL H {2}
pub fn _CB_14(c: &mut Context, _: &mut Bus) {
    c.h = om_rl8!(c; c.h);
}

// CB 15 — RL L {2}
pub fn _CB_15(c: &mut Context, _: &mut Bus) {
    c.l = om_rl8!(c; c.l);
}

// CB 16 — RL (HL) {3}
pub fn _CB_16(c: &mut Context, b: &mut Bus) {
    let r = om_rl8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 17 — RL A {2}
pub fn _CB_17(c: &mut Context, _: &mut Bus) {
    c.a = om_rl8!(c; c.a);
}

// CB 18 — RR B {2}
pub fn _CB_18(c: &mut Context, _: &mut Bus) {
    c.b = om_rr8!(c; c.b);
}

// CB 19 — RR C {2}
pub fn _CB_19(c: &mut Context, _: &mut Bus) {
    c.c = om_rr8!(c; c.c);
}

// CB 1A — RR D {2}
pub fn _CB_1A(c: &mut Context, _: &mut Bus) {
    c.d = om_rr8!(c; c.d);
}

// CB 1B — RR E {2}
pub fn _CB_1B(c: &mut Context, _: &mut Bus) {
    c.e = om_rr8!(c; c.e);
}

// CB 1C — RR H {2}
pub fn _CB_1C(c: &mut Context, _: &mut Bus) {
    c.h = om_rr8!(c; c.h);
}

// CB 1D — RR L {2}
pub fn _CB_1D(c: &mut Context, _: &mut Bus) {
    c.l = om_rr8!(c; c.l);
}

// CB 1E — RR (HL) {3}
pub fn _CB_1E(c: &mut Context, b: &mut Bus) {
    let r = om_rr8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 1F — RR A {2}
pub fn _CB_1F(c: &mut Context, _: &mut Bus) {
    c.a = om_rr8!(c; c.a);
}

// CB 20 — SLA B {2}
pub fn _CB_20(c: &mut Context, _: &mut Bus) {
    c.b = om_sl8!(c; c.b);
}

// CB 21 — SLA C {2}
pub fn _CB_21(c: &mut Context, _: &mut Bus) {
    c.c = om_sl8!(c; c.c);
}

// CB 22 — SLA D {2}
pub fn _CB_22(c: &mut Context, _: &mut Bus) {
    c.d = om_sl8!(c; c.d);
}

// CB 23 — SLA E {2}
pub fn _CB_23(c: &mut Context, _: &mut Bus) {
    c.e = om_sl8!(c; c.e);
}

// CB 24 — SLA H {2}
pub fn _CB_24(c: &mut Context, _: &mut Bus) {
    c.h = om_sl8!(c; c.h);
}

// CB 25 — SLA L {2}
pub fn _CB_25(c: &mut Context, _: &mut Bus) {
    c.l = om_sl8!(c; c.l);
}

// CB 26 — SLA (HL) {3}
pub fn _CB_26(c: &mut Context, b: &mut Bus) {
    let r = om_sl8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 27 — SLA A {2}
pub fn _CB_27(c: &mut Context, _: &mut Bus) {
    c.a = om_sl8!(c; c.a);
}

// CB 28 — SRA B {2}
pub fn _CB_28(c: &mut Context, _: &mut Bus) {
    c.b = om_sra8!(c; c.b);
}

// CB 29 — SRA C {2}
pub fn _CB_29(c: &mut Context, _: &mut Bus) {
    c.c = om_sra8!(c; c.c);
}

// CB 2A — SRA D {2}
pub fn _CB_2A(c: &mut Context, _: &mut Bus) {
    c.d = om_sra8!(c; c.d);
}

// CB 2B — SRA E {2}
pub fn _CB_2B(c: &mut Context, _: &mut Bus) {
    c.e = om_sra8!(c; c.e);
}

// CB 2C — SRA H {2}
pub fn _CB_2C(c: &mut Context, _: &mut Bus) {
    c.h = om_sra8!(c; c.h);
}

// CB 2D — SRA L {2}
pub fn _CB_2D(c: &mut Context, _: &mut Bus) {
    c.l = om_sra8!(c; c.l);
}

// CB 2E — SRA (HL) {3}
pub fn _CB_2E(c: &mut Context, b: &mut Bus) {
    let r = om_sra8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 2F — SRA A {2}
pub fn _CB_2F(c: &mut Context, _: &mut Bus) {
    c.a = om_sra8!(c; c.a);
}

// CB 30 — SWAP B {2}
pub fn _CB_30(c: &mut Context, _: &mut Bus) {
    c.b = om_bswap8!(c; c.b);
}

// CB 31 — SWAP C {2}
pub fn _CB_31(c: &mut Context, _: &mut Bus) {
    c.c = om_bswap8!(c; c.c);
}

// CB 32 — SWAP D {2}
pub fn _CB_32(c: &mut Context, _: &mut Bus) {
    c.d = om_bswap8!(c; c.d);
}

// CB 33 — SWAP E {2}
pub fn _CB_33(c: &mut Context, _: &mut Bus) {
    c.e = om_bswap8!(c; c.e);
}

// CB 34 — SWAP H {2}
pub fn _CB_34(c: &mut Context, _: &mut Bus) {
    c.h = om_bswap8!(c; c.h);
}

// CB 35 — SWAP L {2}
pub fn _CB_35(c: &mut Context, _: &mut Bus) {
    c.l = om_bswap8!(c; c.l);
}

// CB 36 — SWAP (HL) {3}
pub fn _CB_36(c: &mut Context, b: &mut Bus) {
    let r = om_bswap8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 37 — SWAP A {2}
pub fn _CB_37(c: &mut Context, _: &mut Bus) {
    c.a = om_bswap8!(c; c.a);
}

// CB 38 — SRL B {2}
pub fn _CB_38(c: &mut Context, _: &mut Bus) {
    c.b = om_srl8!(c; c.b);
}

// CB 39 — SRL C {2}
pub fn _CB_39(c: &mut Context, _: &mut Bus) {
    c.c = om_srl8!(c; c.c);
}

// CB 3A — SRL D {2}
pub fn _CB_3A(c: &mut Context, _: &mut Bus) {
    c.d = om_srl8!(c; c.d);
}

// CB 3B — SRL E {2}
pub fn _CB_3B(c: &mut Context, _: &mut Bus) {
    c.e = om_srl8!(c; c.e);
}

// CB 3C — SRL H {2}
pub fn _CB_3C(c: &mut Context, _: &mut Bus) {
    c.h = om_srl8!(c; c.h);
}

// CB 3D — SRL L {2}
pub fn _CB_3D(c: &mut Context, _: &mut Bus) {
    c.l = om_srl8!(c; c.l);
}

// CB 3E — SRL (HL) {3}
pub fn _CB_3E(c: &mut Context, b: &mut Bus) {
    let r = om_srl8!(c; om_read8!(c, b; c.get_hl()));
    om_write8!(c, b; c.get_hl(), r);
}

// CB 3F — SRL A {2}
pub fn _CB_3F(c: &mut Context, _: &mut Bus) {
    c.a = om_srl8!(c; c.a);
}

// CB 40 — BIT 0, B {2}
pub fn _CB_40(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 0);
}

// CB 41 — BIT 0, C {2}
pub fn _CB_41(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 0);
}

// CB 42 — BIT 0, D {2}
pub fn _CB_42(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 0);
}

// CB 43 — BIT 0, E {2}
pub fn _CB_43(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 0);
}

// CB 44 — BIT 0, H {2}
pub fn _CB_44(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 0);
}

// CB 45 — BIT 0, L {2}
pub fn _CB_45(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 0);
}

// CB 46 — BIT 0, (HL) {3}
pub fn _CB_46(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 0);
}

// CB 47 — BIT 0, A {2}
pub fn _CB_47(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 0);
}

// CB 48 — BIT 1, B {2}
pub fn _CB_48(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 1);
}

// CB 49 — BIT 1, C {2}
pub fn _CB_49(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 1);
}

// CB 4A — BIT 1, D {2}
pub fn _CB_4A(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 1);
}

// CB 4B — BIT 1, E {2}
pub fn _CB_4B(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 1);
}

// CB 4C — BIT 1, H {2}
pub fn _CB_4C(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 1);
}

// CB 4D — BIT 1, L {2}
pub fn _CB_4D(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 1);
}

// CB 4E — BIT 1, (HL) {3}
pub fn _CB_4E(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 1);
}

// CB 4F — BIT 1, A {2}
pub fn _CB_4F(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 1);
}

// CB 50 — BIT 2, B {2}
pub fn _CB_50(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 2);
}

// CB 51 — BIT 2, C {2}
pub fn _CB_51(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 2);
}

// CB 52 — BIT 2, D {2}
pub fn _CB_52(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 2);
}

// CB 53 — BIT 2, E {2}
pub fn _CB_53(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 2);
}

// CB 54 — BIT 2, H {2}
pub fn _CB_54(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 2);
}

// CB 55 — BIT 2, L {2}
pub fn _CB_55(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 2);
}

// CB 56 — BIT 2, (HL) {3}
pub fn _CB_56(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 2);
}

// CB 57 — BIT 2, A {2}
pub fn _CB_57(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 2);
}

// CB 58 — BIT 3, B {2}
pub fn _CB_58(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 3);
}

// CB 59 — BIT 3, C {2}
pub fn _CB_59(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 3);
}

// CB 5A — BIT 3, D {2}
pub fn _CB_5A(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 3);
}

// CB 5B — BIT 3, E {2}
pub fn _CB_5B(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 3);
}

// CB 5C — BIT 3, H {2}
pub fn _CB_5C(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 3);
}

// CB 5D — BIT 3, L {2}
pub fn _CB_5D(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 3);
}

// CB 5E — BIT 3, (HL) {3}
pub fn _CB_5E(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 3);
}

// CB 5F — BIT 3, A {2}
pub fn _CB_5F(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 3);
}

// CB 60 — BIT 4, B {2}
pub fn _CB_60(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 4);
}

// CB 61 — BIT 4, C {2}
pub fn _CB_61(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 4);
}

// CB 62 — BIT 4, D {2}
pub fn _CB_62(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 4);
}

// CB 63 — BIT 4, E {2}
pub fn _CB_63(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 4);
}

// CB 64 — BIT 4, H {2}
pub fn _CB_64(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 4);
}

// CB 65 — BIT 4, L {2}
pub fn _CB_65(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 4);
}

// CB 66 — BIT 4, (HL) {3}
pub fn _CB_66(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 4);
}

// CB 67 — BIT 4, A {2}
pub fn _CB_67(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 4);
}

// CB 68 — BIT 5, B {2}
pub fn _CB_68(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 5);
}

// CB 69 — BIT 5, C {2}
pub fn _CB_69(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 5);
}

// CB 6A — BIT 5, D {2}
pub fn _CB_6A(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 5);
}

// CB 6B — BIT 5, E {2}
pub fn _CB_6B(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 5);
}

// CB 6C — BIT 5, H {2}
pub fn _CB_6C(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 5);
}

// CB 6D — BIT 5, L {2}
pub fn _CB_6D(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 5);
}

// CB 6E — BIT 5, (HL) {2}
pub fn _CB_6E(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 5);
}

// CB 6F — BIT 5, A {2}
pub fn _CB_6F(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 5);
}

// CB 70 — BIT 6, B {2}
pub fn _CB_70(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 6);
}

// CB 71 — BIT 6, C {2}
pub fn _CB_71(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 6);
}

// CB 72 — BIT 6, D {2}
pub fn _CB_72(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 6);
}

// CB 73 — BIT 6, E {2}
pub fn _CB_73(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 6);
}

// CB 74 — BIT 6, H {2}
pub fn _CB_74(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 6);
}

// CB 75 — BIT 6, L {2}
pub fn _CB_75(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 6);
}

// CB 76 — BIT 6, (HL) {3}
pub fn _CB_76(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 6);
}

// CB 77 — BIT 6, A {2}
pub fn _CB_77(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 6);
}

// CB 78 — BIT 7, B {2}
pub fn _CB_78(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.b, 7);
}

// CB 79 — BIT 7, C {2}
pub fn _CB_79(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.c, 7);
}

// CB 7A — BIT 7, D {2}
pub fn _CB_7A(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.d, 7);
}

// CB 7B — BIT 7, E {2}
pub fn _CB_7B(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.e, 7);
}

// CB 7C — BIT 7, H {2}
pub fn _CB_7C(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.h, 7);
}

// CB 7D — BIT 7, L {2}
pub fn _CB_7D(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.l, 7);
}

// CB 7E — BIT 7, (HL) {3}
pub fn _CB_7E(c: &mut Context, b: &mut Bus) {
    let hl = c.get_hl();
    om_bit8!(c; om_read8!(c, b; hl), 7);
}

// CB 7F — BIT 7, A {2}
pub fn _CB_7F(c: &mut Context, _: &mut Bus) {
    om_bit8!(c; c.a, 7);
}

// CB 80 — RES 0, B {2}
pub fn _CB_80(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 0);
}

// CB 81 — RES 0, C {2}
pub fn _CB_81(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 0);
}

// CB 82 — RES 0, D {2}
pub fn _CB_82(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 0);
}

// CB 83 — RES 0, E {2}
pub fn _CB_83(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 0);
}

// CB 84 — RES 0, H {2}
pub fn _CB_84(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 0);
}

// CB 85 — RES 0, L {2}
pub fn _CB_85(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 0);
}

// CB 86 — RES 0, (HL) {3}
pub fn _CB_86(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 0);
    om_write8!(c, b; c.get_hl(), r);
}

// CB 87 — RES 0, A {2}
pub fn _CB_87(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 0);
}

// CB 88 — RES 1, B {2}
pub fn _CB_88(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 1);
}

// CB 89 — RES 1, C {2}
pub fn _CB_89(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 1);
}

// CB 8A — RES 1, D {2}
pub fn _CB_8A(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 1);
}

// CB 8B — RES 1, E {2}
pub fn _CB_8B(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 1);
}

// CB 8C — RES 1, H {2}
pub fn _CB_8C(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 1);
}

// CB 8D — RES 1, L {2}
pub fn _CB_8D(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 1);
}

// CB 8E — RES 1, (HL) {3}
pub fn _CB_8E(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 1);
    om_write8!(c, b; c.get_hl(), r);
}

// CB 8F — RES 1, A {2}
pub fn _CB_8F(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 1);
}

// CB 90 — RES 2, B {2}
pub fn _CB_90(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 2);
}

// CB 91 — RES 2, C {2}
pub fn _CB_91(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 2);
}

// CB 92 — RES 2, D {2}
pub fn _CB_92(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 2);
}

// CB 93 — RES 2, E {2}
pub fn _CB_93(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 2);
}

// CB 94 — RES 2, H {2}
pub fn _CB_94(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 2);
}

// CB 95 — RES 2, L {2}
pub fn _CB_95(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 2);
}

// CB 96 — RES 2, (HL) {3}
pub fn _CB_96(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 2);
    om_write8!(c, b; c.get_hl(), r);
}

// CB 97 — RES 2, A {2}
pub fn _CB_97(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 2);
}

// CB 98 — RES 3, B {2}
pub fn _CB_98(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 3);
}

// CB 99 — RES 3, C {2}
pub fn _CB_99(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 3);
}

// CB 9A — RES 3, D {2}
pub fn _CB_9A(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 3);
}

// CB 9B — RES 3, E {2}
pub fn _CB_9B(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 3);
}

// CB 9C — RES 3, H {2}
pub fn _CB_9C(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 3);
}

// CB 9D — RES 3, L {2}
pub fn _CB_9D(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 3);
}

// CB 9E — RES 3, (HL) {3}
pub fn _CB_9E(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 3);
    om_write8!(c, b; c.get_hl(), r);
}

// CB 9F — RES 3, A {2}
pub fn _CB_9F(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 3);
}

// CB A0 — RES 4, B {2}
pub fn _CB_A0(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 4);
}

// CB A1 — RES 4, C {2}
pub fn _CB_A1(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 4);
}

// CB A2 — RES 4, D {2}
pub fn _CB_A2(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 4);
}

// CB A3 — RES 4, E {2}
pub fn _CB_A3(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 4);
}

// CB A4 — RES 4, H {2}
pub fn _CB_A4(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 4);
}

// CB A5 — RES 4, L {2}
pub fn _CB_A5(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 4);
}

// CB A6 — RES 4, (HL) {3}
pub fn _CB_A6(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 4);
    om_write8!(c, b; c.get_hl(), r);
}

// CB A7 — RES 4, A {2}
pub fn _CB_A7(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 4);
}

// CB A8 — RES 5, B {2}
pub fn _CB_A8(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 5);
}

// CB A9 — RES 5, C {2}
pub fn _CB_A9(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 5);
}

// CB AA — RES 5, D {2}
pub fn _CB_AA(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 5);
}

// CB AB — RES 5, E {2}
pub fn _CB_AB(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 5);
}

// CB AC — RES 5, H {2}
pub fn _CB_AC(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 5);
}

// CB AD — RES 5, L {2}
pub fn _CB_AD(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 5);
}

// CB AE — RES 5, (HL) {3}
pub fn _CB_AE(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 5);
    om_write8!(c, b; c.get_hl(), r);
}

// CB AF — RES 5, A {2}
pub fn _CB_AF(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 5);
}

// CB B0 — RES 6, B {2}
pub fn _CB_B0(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 6);
}

// CB B1 — RES 6, C {2}
pub fn _CB_B1(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 6);
}

// CB B2 — RES 6, D {2}
pub fn _CB_B2(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 6);
}

// CB B3 — RES 6, E {2}
pub fn _CB_B3(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 6);
}

// CB B4 — RES 6, H {2}
pub fn _CB_B4(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 6);
}

// CB B5 — RES 6, L {2}
pub fn _CB_B5(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 6);
}

// CB B6 — RES 6, (HL) {3}
pub fn _CB_B6(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 6);
    om_write8!(c, b; c.get_hl(), r);
}

// CB B7 — RES 6, A {2}
pub fn _CB_B7(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 6);
}

// CB B8 — RES 7, B {2}
pub fn _CB_B8(c: &mut Context, _: &mut Bus) {
    c.b = om_res8!(c; c.b, 7);
}

// CB B9 — RES 7, C {2}
pub fn _CB_B9(c: &mut Context, _: &mut Bus) {
    c.c = om_res8!(c; c.c, 7);
}

// CB BA — RES 7, D {2}
pub fn _CB_BA(c: &mut Context, _: &mut Bus) {
    c.d = om_res8!(c; c.d, 7);
}

// CB BB — RES 7, E {2}
pub fn _CB_BB(c: &mut Context, _: &mut Bus) {
    c.e = om_res8!(c; c.e, 7);
}

// CB BC — RES 7, H {2}
pub fn _CB_BC(c: &mut Context, _: &mut Bus) {
    c.h = om_res8!(c; c.h, 7);
}

// CB BD — RES 7, L {2}
pub fn _CB_BD(c: &mut Context, _: &mut Bus) {
    c.l = om_res8!(c; c.l, 7);
}

// CB BE — RES 7, (HL) {3}
pub fn _CB_BE(c: &mut Context, b: &mut Bus) {
    let r = om_res8!(c; om_read8!(c, b; c.get_hl()), 7);
    om_write8!(c, b; c.get_hl(), r);
}

// CB BF — RES 7, A {2}
pub fn _CB_BF(c: &mut Context, _: &mut Bus) {
    c.a = om_res8!(c; c.a, 7);
}

// CB C0 — SET 0, B {2}
pub fn _CB_C0(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 0);
}

// CB C1 — SET 0, C {2}
pub fn _CB_C1(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 0);
}

// CB C2 — SET 0, D {2}
pub fn _CB_C2(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 0);
}

// CB C3 — SET 0, E {2}
pub fn _CB_C3(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 0);
}

// CB C4 — SET 0, H {2}
pub fn _CB_C4(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 0);
}

// CB C5 — SET 0, L {2}
pub fn _CB_C5(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 0);
}

// CB C6 — SET 0, (HL) {3}
pub fn _CB_C6(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 0);
    om_write8!(c, b; c.get_hl(), r);
}

// CB C7 — SET 0, A {2}
pub fn _CB_C7(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 0);
}

// CB C8 — SET 1, B {2}
pub fn _CB_C8(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 1);
}

// CB C9 — SET 1, C {2}
pub fn _CB_C9(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 1);
}

// CB CA — SET 1, D {2}
pub fn _CB_CA(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 1);
}

// CB CB — SET 1, E {2}
pub fn _CB_CB(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 1);
}

// CB CC — SET 1, H {2}
pub fn _CB_CC(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 1);
}

// CB CD — SET 1, L {2}
pub fn _CB_CD(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 1);
}

// CB CE — SET 1, (HL) {3}
pub fn _CB_CE(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 1);
    om_write8!(c, b; c.get_hl(), r);
}

// CB CF — SET 1, A {2}
pub fn _CB_CF(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 1);
}

// CB D0 — SET 2, B {2}
pub fn _CB_D0(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 2);
}

// CB D1 — SET 2, C {2}
pub fn _CB_D1(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 2);
}

// CB D2 — SET 2, D {2}
pub fn _CB_D2(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 2);
}

// CB D3 — SET 2, E {2}
pub fn _CB_D3(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 2);
}

// CB D4 — SET 2, H {2}
pub fn _CB_D4(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 2);
}

// CB D5 — SET 2, L {2}
pub fn _CB_D5(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 2);
}

// CB D6 — SET 2, (HL) {3}
pub fn _CB_D6(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 2);
    om_write8!(c, b; c.get_hl(), r);
}

// CB D7 — SET 2, A {2}
pub fn _CB_D7(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 2);
}

// CB D8 — SET 3, B {2}
pub fn _CB_D8(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 3);
}

// CB D9 — SET 3, C {2}
pub fn _CB_D9(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 3);
}

// CB DA — SET 3, D {2}
pub fn _CB_DA(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 3);
}

// CB DB — SET 3, E {2}
pub fn _CB_DB(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 3);
}

// CB DC — SET 3, H {2}
pub fn _CB_DC(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 3);
}

// CB DD — SET 3, L {2}
pub fn _CB_DD(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 3);
}

// CB DE — SET 3, (HL) {3}
pub fn _CB_DE(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 3);
    om_write8!(c, b; c.get_hl(), r);
}

// CB DF — SET 3, A {2}
pub fn _CB_DF(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 3);
}

// CB E0 — SET 4, B {2}
pub fn _CB_E0(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 4);
}

// CB E1 — SET 4, C {2}
pub fn _CB_E1(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 4);
}

// CB E2 — SET 4, D {2}
pub fn _CB_E2(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 4);
}

// CB E3 — SET 4, E {2}
pub fn _CB_E3(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 4);
}

// CB E4 — SET 4, H {2}
pub fn _CB_E4(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 4);
}

// CB E5 — SET 4, L {2}
pub fn _CB_E5(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 4);
}

// CB E6 — SET 4, (HL) {3}
pub fn _CB_E6(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 4);
    om_write8!(c, b; c.get_hl(), r);
}

// CB E7 — SET 4, A {2}
pub fn _CB_E7(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 4);
}

// CB E8 — SET 5, B {2}
pub fn _CB_E8(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 5);
}

// CB E9 — SET 5, C {2}
pub fn _CB_E9(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 5);
}

// CB EA — SET 5, D {2}
pub fn _CB_EA(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 5);
}

// CB EB — SET 5, E {2}
pub fn _CB_EB(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 5);
}

// CB EC — SET 5, H {2}
pub fn _CB_EC(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 5);
}

// CB ED — SET 5, L {2}
pub fn _CB_ED(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 5);
}

// CB EE — SET 5, (HL) {3}
pub fn _CB_EE(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 5);
    om_write8!(c, b; c.get_hl(), r);
}

// CB EF — SET 5, A {2}
pub fn _CB_EF(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 5);
}

// CB F0 — SET 6, B {2}
pub fn _CB_F0(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 6);
}

// CB F1 — SET 6, C {2}
pub fn _CB_F1(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 6);
}

// CB F2 — SET 6, D {2}
pub fn _CB_F2(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 6);
}

// CB F3 — SET 6, E {2}
pub fn _CB_F3(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 6);
}

// CB F4 — SET 6, H {2}
pub fn _CB_F4(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 6);
}

// CB F5 — SET 6, L {2}
pub fn _CB_F5(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 6);
}

// CB F6 — SET 6, (HL) {3}
pub fn _CB_F6(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 6);
    om_write8!(c, b; c.get_hl(), r);
}

// CB F7 — SET 6, A {2}
pub fn _CB_F7(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 6);
}

// CB F8 — SET 7, B {2}
pub fn _CB_F8(c: &mut Context, _: &mut Bus) {
    c.b = om_set8!(c; c.b, 7);
}

// CB F9 — SET 7, C {2}
pub fn _CB_F9(c: &mut Context, _: &mut Bus) {
    c.c = om_set8!(c; c.c, 7);
}

// CB FA — SET 7, D {2}
pub fn _CB_FA(c: &mut Context, _: &mut Bus) {
    c.d = om_set8!(c; c.d, 7);
}

// CB FB — SET 7, E {2}
pub fn _CB_FB(c: &mut Context, _: &mut Bus) {
    c.e = om_set8!(c; c.e, 7);
}

// CB FC — SET 7, H {2}
pub fn _CB_FC(c: &mut Context, _: &mut Bus) {
    c.h = om_set8!(c; c.h, 7);
}

// CB FD — SET 7, L {2}
pub fn _CB_FD(c: &mut Context, _: &mut Bus) {
    c.l = om_set8!(c; c.l, 7);
}

// CB FE — SET 7, (HL) {3}
pub fn _CB_FE(c: &mut Context, b: &mut Bus) {
    let r = om_set8!(c; om_read8!(c, b; c.get_hl()), 7);
    om_write8!(c, b; c.get_hl(), r);
}

// CB FF — SET 7, A {2}
pub fn _CB_FF(c: &mut Context, _: &mut Bus) {
    c.a = om_set8!(c; c.a, 7);
}

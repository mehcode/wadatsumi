// Operation Macros
// Instructions are broken down into reusable macros that allow zero-cost code reuse.

// 8-bit Memory Read/Write
// -----------------------

/// 8-bit Read {+1}
macro_rules! om_read8 (($c:expr, $b:expr; $address:expr) => {
    {
        $c.step($b);
        $b.read($address)
    }
});

/// 8-bit Read Next/Immediate {+1}
macro_rules! om_read_next8 (($c:expr, $b:expr) => {
    {
        let r = om_read8!($c, $b; $c.pc);
        $c.pc = $c.pc.wrapping_add(1);

        r
    }
});

/// 8-bit Write {+1}
macro_rules! om_write8 (($c:expr, $b:expr; $address:expr, $value:expr) => {
    {
        $c.step($b);
        $b.write($address, $value)
    }
});

// 8-bit Arithmetic/Logical
// ------------------------

/// 8-bit Decrement [z1h-]
macro_rules! om_dec8 (($c:expr; $e:expr) => {
    {
        let r = $e.wrapping_sub(1);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, true);
        $c.set_flag(cpu::HALF_CARRY, r & 0x0F == 0x0F);

        r
    }
});

/// 8-bit Decrement Register [z1h-]
macro_rules! om_dec8_r (($c:expr; $reg:ident) => {
    let r = om_dec8!($c; $c.$reg);
    $c.$reg = r;
});

/// 8-bit Increment [z1h-]
macro_rules! om_inc8 (($c:expr; $e:expr) => {
    {
        let r = $e.wrapping_add(1);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, r & 0x0F == 0x00);

        r
    }
});

/// 8-bit Increment Register [z1h-]
macro_rules! om_inc8_r (($c:expr; $reg:ident) => {
    let r = om_inc8!($c; $c.$reg);
    $c.$reg = r;
});

/// 8-bit Add (to A) [z0hc]
macro_rules! om_add8_a (($c:expr; $e:expr) => {
    let a = $c.a as u16;
    let b = $e as u16;
    let r = a.wrapping_add(b);

    $c.set_flag(cpu::HALF_CARRY, ((a & 0x0F) + (b & 0x0F)) > 0x0F);
    $c.set_flag(cpu::ZERO, (r & 0xFF) == 0);
    $c.set_flag(cpu::CARRY, r > 0xFF);
    $c.set_flag(cpu::ADD_SUBTRACT, false);

    $c.a = (r & 0xFF) as u8;
});

/// 8-bit Add (to A) w/Carry [z0hc]
macro_rules! om_adc8_a (($c:expr; $e:expr) => {
    let a = $c.a as u16;
    let b = $e as u16;
    let c = if $c.f.contains(cpu::CARRY) { 1 } else { 0 };
    let r = a.wrapping_add(b).wrapping_add(c);

    $c.set_flag(cpu::HALF_CARRY, ((a & 0x0F) + (b & 0x0F) + c) > 0x0F);
    $c.set_flag(cpu::ZERO, (r & 0xFF) == 0);
    $c.set_flag(cpu::CARRY, r > 0xFF);
    $c.set_flag(cpu::ADD_SUBTRACT, false);

    $c.a = (r & 0xFF) as u8;
});

/// 8-bit Compare (from A) [z1hc]
macro_rules! om_cp8_a (($c:expr; $e:expr) => {
    {
        let a = $c.a as i16;
        let b = $e as i16;
        let r = a.wrapping_sub(b);

        $c.set_flag(cpu::CARRY, r < 0);
        $c.set_flag(cpu::ZERO, (r & 0xFF) == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, true);
        $c.set_flag(cpu::HALF_CARRY, ((((a as i16) & 0x0F) - ((b as i16) & 0x0F)) < 0));

        (r & 0xFF) as u8
    }
});

/// 8-bit Subtract (from A) [z1hc]
macro_rules! om_sub8_a (($c:expr; $e:expr) => {
    $c.a = om_cp8_a!($c; $e);
});

/// 8-bit Subtract (from A) w/Carry [z1hc]
macro_rules! om_sbc8_a (($c:expr; $e:expr) => {
    let a = $c.a as i16;
    let b = $e as i16;
    let c = if $c.f.contains(cpu::CARRY) { 1 } else { 0 };
    let r = a.wrapping_sub(b).wrapping_sub(c);

    $c.set_flag(cpu::CARRY, r < 0);
    $c.set_flag(cpu::ZERO, (r & 0xFF) == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, true);
    $c.set_flag(cpu::HALF_CARRY, ((((a as i16) & 0x0F) - ((b as i16) & 0x0F) - (c as i16)) < 0));

    $c.a = (r & 0xFF) as u8;
});

/// 8-bit Logical AND (with A) [z010]
macro_rules! om_and8_a (($c:expr; $e:expr) => {
    let a = $c.a;
    let b = $e;
    let r = a & b;

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, true);
    $c.set_flag(cpu::CARRY, false);

    $c.a = r;
});

/// 8-bit Logical OR (with A) [z010]
macro_rules! om_or8_a (($c:expr; $e:expr) => {
    let a = $c.a;
    let b = $e;
    let r = a | b;

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, false);
    $c.set_flag(cpu::CARRY, false);

    $c.a = r;
});

/// 8-bit Logical XOR (with A) [z010]
macro_rules! om_xor8_a (($c:expr; $e:expr) => {
    let a = $c.a;
    let b = $e;
    let r = a ^ b;

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, false);
    $c.set_flag(cpu::CARRY, false);

    $c.a = r;
});

// 8-bit Rotate
// ------------

/// 8-bit Rotate Left (through carry) [z00c]
macro_rules! om_rl8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = (n << 1) | ($c.f.contains(cpu::CARRY) as u8);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, ((n & 0x80) != 0));

        r
    }
});

/// 8-bit Rotate Accumulator Left (through carry) [000c]
macro_rules! om_rla8 (($c:ident) => {
    let z = $c.get_flag(cpu::ZERO);
    $c.a = om_rl8!($c; $c.a);
    $c.set_flag(cpu::ZERO, z);
});

/// 8-bit Rotate Left [z00c]
macro_rules! om_rlc8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = (n << 1) | (n >> 7);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, ((n & 0x80) != 0));

        r
    }
});

/// 8-bit Rotate Accumulator Left [000c]
macro_rules! om_rlca8 (($c:ident) => {
    let z = $c.get_flag(cpu::ZERO);
    $c.a = om_rlc8!($c; $c.a);
    $c.set_flag(cpu::ZERO, z);
});

/// 8-bit Rotate Right (through carry) [z00c]
macro_rules! om_rr8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = (n >> 1) | (($c.f.contains(cpu::CARRY) as u8) << 7);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, ((n & 0x01) != 0));

        r
    }
});

/// 8-bit Rotate Accumulator Right (through carry) [000c]
macro_rules! om_rra8 (($c:ident) => {
    let z = $c.get_flag(cpu::ZERO);
    $c.a = om_rr8!($c; $c.a);
    $c.set_flag(cpu::ZERO, z);
});

/// 8-bit Rotate Right [z00c]
macro_rules! om_rrc8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = (n >> 1) | (n << 7);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, ((n & 0x01) != 0));

        r
    }
});

/// 8-bit Rotate Accumulator Right [000c]
macro_rules! om_rrca8 (($c:ident) => {
    let z = $c.get_flag(cpu::ZERO);
    $c.a = om_rrc8!($c; $c.a);
    $c.set_flag(cpu::ZERO, z);
});

// 8-bit Shift
// -----------

/// 8-bit Shift Left [z00c]
macro_rules! om_sl8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = n << 1;

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, (n & 0x80) != 0);

        r
    }
});

/// 8-bit Shift Right Logical [z00c]
macro_rules! om_srl8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = n >> 1;

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, (n & 0x01) != 0);

        r
    }
});

/// 8-bit Shift Right Arithmetic [z00c]
macro_rules! om_sra8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = if (n & 0x80) != 0 {
            (n >> 1) | 0x80
        } else {
            (n >> 1)
        };

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, (n & 0x01) != 0);

        r
    }
});

// 8-bit Byte Swap
// ---------------

/// 8-bit Byte Swap [z000]
macro_rules! om_bswap8 (($c:expr; $e:expr) => {
    {
        let n = $e;
        let r = (n >> 4) | ((n << 4) & 0xF0);

        $c.set_flag(cpu::ZERO, r == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, false);
        $c.set_flag(cpu::CARRY, false);

        r
    }
});

// Bit
// ---

/// Bit Test [z01-]
macro_rules! om_bit8 (($c:expr; $e:expr, $n:expr) => {
    {
        let e = $e;

        $c.set_flag(cpu::ZERO, (e & (1 << $n)) == 0);
        $c.set_flag(cpu::ADD_SUBTRACT, false);
        $c.set_flag(cpu::HALF_CARRY, true);
    }
});

/// Bit Set [----]
macro_rules! om_set8 (($c:expr; $e:expr, $n:expr) => {
    {
        $e | (1 << $n)
    }
});

/// Bit Reset [----]
macro_rules! om_res8 (($c:expr; $e:expr, $n:expr) => {
    {
        $e & !(1 << $n)
    }
});

// 16-bit Memory Read/Write
// ------------------------

/// 16-bit Read {+2}
macro_rules! om_read16 (($c:expr, $b:expr; $address:expr) => {
    {
        let l = om_read8!($c, $b; $address.wrapping_add(0)) as u16;
        let h = om_read8!($c, $b; $address.wrapping_add(1)) as u16;

        l | (h << 8)
    }
});

/// 16-bit Read Next/Immediate {+1}
macro_rules! om_read_next16 (($c:expr, $b:expr) => {
    {
        let r = om_read16!($c, $b; $c.pc);
        $c.pc = $c.pc.wrapping_add(2);

        r
    }
});

/// 16-bit Write {+2}
macro_rules! om_write16 (($c:expr, $b:expr; $address:expr, $value:expr) => {
    {
      om_write8!($c, $b; $address.wrapping_add(1), ($value >> 8) as u8);
      om_write8!($c, $b; $address.wrapping_add(0), ($value & 0xFF) as u8);
    }
});

// 16-bit Push/Pop
// ---------------

/// 16-bit Push [----] {+3}
macro_rules! om_push16 (($c:expr, $b:expr; $e:expr) => {
    // Push has a 1 M-cycle delay
    $c.step($b);

    $c.sp = $c.sp.wrapping_sub(2);
    om_write16!($c, $b; $c.sp, $e);
});

/// 16-bit Pop [(..)] {+2}
macro_rules! om_pop16 (($c:expr, $b:expr) => {
    {
        let r = om_read16!($c, $b; $c.sp);
        $c.sp = $c.sp.wrapping_add(2);

        r
    }
});

// 16-bit Arithmetic/Logical
// -------------------------

/// 16-bit Increment [----] {+1}
macro_rules! om_inc16 (($c:expr, $b:expr; $get:ident, $set:ident) => {
    let r = $c.$get();

    $c.$set(r.wrapping_add(1));
    $c.step($b);
});

/// 16-bit Decrement [----] {+1}
macro_rules! om_dec16 (($c:expr, $b:expr; $get:ident, $set:ident) => {
    let r = $c.$get();

    $c.$set(r.wrapping_sub(1));
    $c.step($b);
});

/// 16-bit Add (to HL) [-0hc] {+1}
macro_rules! om_add16_hl (($c:expr, $b:expr; $e:expr) => {
    let a = $c.get_hl();
    let b = $e;
    let r = a as u32 + b as u32;

    $c.set_flag(cpu::HALF_CARRY, ((a ^ b ^ ((r & 0xFFFF) as u16)) & 0x1000) != 0);
    $c.set_flag(cpu::CARRY, r > 0xFFFF);
    $c.set_flag(cpu::ADD_SUBTRACT, false);

    $c.set_hl((r & 0xFFFF) as u16);
    $c.step($b);
});

/// 16-bit Add (to SP) without Assignment [-0hc] {+1}
macro_rules! om_add16_sp (($c:expr, $b:expr; $e:expr) => {
    {
        let a = $c.sp;
        let b = ($e as i8) as i16;
        let r = ((a as i16) + b) as u16;

        $c.set_flag(cpu::CARRY, (r & 0xFF) < (a & 0xFF));
        $c.set_flag(cpu::HALF_CARRY, (r & 0xF) < (a & 0xF));
        $c.set_flag(cpu::ZERO, false);
        $c.set_flag(cpu::ADD_SUBTRACT, false);

        $c.step($b);

        r
    }
});

// Jump
// ----

/// Jump [----] {+3}
macro_rules! om_jp (($c:expr, $b:expr) => {
    let address = om_read_next16!($c, $b);
    $c.pc = address;

    $c.step($b);
});

/// Jump; If [----] {+3;+2}
macro_rules! om_jp_if (($c:expr, $b:expr; $flag:expr) => {
    if $c.f.contains($flag) {
        om_jp!($c, $b);
    } else {
        $c.pc += 2;
        $c.step($b);
        $c.step($b);
    }
});

/// Jump; Unless [----] {+3;+2}
macro_rules! om_jp_unless (($c:expr, $b:expr; $flag:expr) => {
    if !$c.f.contains($flag) {
        om_jp!($c, $b);
    } else {
        $c.pc += 2;
        $c.step($b);
        $c.step($b);
    }
});

/// Relative Jump [----] {+2}
macro_rules! om_jr (($c:expr, $b:expr) => {
    let offset = om_read_next8!($c, $b);
    $c.pc = (($c.pc as i32) + (((offset as u8) as i8) as i32)) as u16;

    $c.step($b);
});

/// Relative Jump; If [----] {+2;+1}
macro_rules! om_jr_if (($c:expr, $b:expr; $flag:expr) => {
    if $c.f.contains($flag) {
        om_jr!($c, $b);
    } else {
        $c.pc += 1;
        $c.step($b);
    }
});

/// Relative Jump; Unless [----] {+2;+1}
macro_rules! om_jr_unless (($c:expr, $b:expr; $flag:expr) => {
    if !$c.f.contains($flag) {
        om_jr!($c, $b);
    } else {
        $c.pc += 1;
        $c.step($b);
    }
});

// Call
// ----

/// Call [----] {+5}
macro_rules! om_call (($c:expr, $b:expr) => {
    let address = om_read_next16!($c, $b);
    om_push16!($c, $b; $c.pc);

    $c.pc = address;
});

/// Call; If [----] {+5;+2}
macro_rules! om_call_if (($c:expr, $b:expr; $flag:expr) => {
    if $c.f.contains($flag) {
        om_call!($c, $b);
    } else {
        $c.pc += 2;
        $c.step($b);
        $c.step($b);
    }
});

/// Call; Unless [----] {+5;+2}
macro_rules! om_call_unless (($c:expr, $b:expr; $flag:expr) => {
    if !$c.f.contains($flag) {
        om_call!($c, $b);
    } else {
        $c.pc += 2;
        $c.step($b);
        $c.step($b);
    }
});

// Reset
// -----

/// Reset [----] {+3}
macro_rules! om_rst (($c:expr, $b:expr; $address:expr) => {
    om_push16!($c, $b; $c.pc);
    $c.pc = $address;
});

// Return
// ------

/// Return [----] {+3}
macro_rules! om_ret (($c:expr, $b:expr) => {
    $c.pc = om_pop16!($c, $b);
    $c.step($b);
});

/// Return; If [----] {+4;+1}
macro_rules! om_ret_if (($c:expr, $b:expr; $flag:expr) => {
    $c.step($b);
    if $c.f.contains($flag) {
        om_ret!($c, $b);
    }
});

/// Return; Unless [----] {+4;+1}
macro_rules! om_ret_unless (($c:expr, $b:expr; $flag:expr) => {
    $c.step($b);
    if !$c.f.contains($flag) {
        om_ret!($c, $b);
    }
});

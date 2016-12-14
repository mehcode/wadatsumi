// Operation Macros
// Instructions are broken down into reusable macros that allow zero-cost code reuse.

// 8-bit Memory Read/Write

/// 8-bit Read {+1}
macro_rules! om_read8 (($c:ident, $b:ident; $address:expr) => {
    {
        $c.step($b);
        $b.read($address)
    }
});

/// 8-bit Read Next/Immediate {+1}
macro_rules! om_read_next8 (($c:ident, $b:ident) => {
    {
        let r = om_read8!($c, $b; $c.pc);
        $c.pc += 1;

        r
    }
});

/// 8-bit Write {+1}
macro_rules! om_write8 (($c:ident, $b:ident; $address:expr, $value:expr) => {
    {
        $c.step($b);
        $b.write($address, $value)
    }
});

// 8-bit Arithmetic/Logical

/// 8-bit Decrement [z1h-]
macro_rules! om_dec8 (($c:ident; $e:expr) => {
    let r = $e - 1;

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, true);
    $c.set_flag(cpu::HALF_CARRY, r & 0x0F == 0x0F);

    $e = r;
});

/// 8-bit Increment [z1h-]
macro_rules! om_inc8 (($c:ident; $e:expr) => {
    let r = $e + 1;

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, r & 0x0F == 0x00);

    $e = r;
});

// 8-bit Rotate/Shift

/// 8-bit Rotate Left (through carry) [z00c]
macro_rules! om_rl8 (($c:ident; $e:expr) => {
    let n = $e;
    let r = (n << 1) | ($c.f.contains(cpu::CARRY) as u8);

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, false);
    $c.set_flag(cpu::CARRY, ((n & 0x80) != 0));

    $e = r;
});

/// 8-bit Rotate Accumulator Left (through carry) [000c]
macro_rules! om_rla8 (($c:ident) => {
    om_rl8!($c; $c.a);
    $c.set_flag(cpu::ZERO, false);
});

/// 8-bit Rotate Left [z00c]
macro_rules! om_rlc8 (($c:ident; $e:expr) => {
    let n = $e;
    let r = (n << 1) | (n >> 7);

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, false);
    $c.set_flag(cpu::CARRY, ((n & 0x80) != 0));

    $e = r;
});

/// 8-bit Rotate Accumulator Left [000c]
macro_rules! om_rlca8 (($c:ident) => {
    om_rlc8!($c; $c.a);
    $c.set_flag(cpu::ZERO, false);
});

/// 8-bit Rotate Right (through carry) [z00c]
macro_rules! om_rr8 (($c:ident; $e:expr) => {
    let n = $e;
    let r = (n >> 1) | (($c.f.contains(cpu::CARRY) as u8) << 7);

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, false);
    $c.set_flag(cpu::CARRY, ((n & 0x01) != 0));

    $e = r;
});

/// 8-bit Rotate Accumulator Right (through carry) [000c]
macro_rules! om_rra8 (($c:ident) => {
    om_rr8!($c; $c.a);
    $c.set_flag(cpu::ZERO, false);
});

/// 8-bit Rotate Right [z00c]
macro_rules! om_rrc8 (($c:ident; $e:expr) => {
    let n = $e;
    let r = (n >> 1) | (n << 7);

    $c.set_flag(cpu::ZERO, r == 0);
    $c.set_flag(cpu::ADD_SUBTRACT, false);
    $c.set_flag(cpu::HALF_CARRY, false);
    $c.set_flag(cpu::CARRY, ((n & 0x01) != 0));

    $e = r;
});

/// 8-bit Rotate Accumulator Right [000c]
macro_rules! om_rrca8 (($c:ident) => {
    om_rrc8!($c; $c.a);
    $c.set_flag(cpu::ZERO, false);
});

// 16-bit Memory Read/Write

/// 16-bit Read {+2}
macro_rules! om_read16 (($c:ident, $b:ident; $address:expr) => {
    {
        let l = om_read8!($c, $b; $address + 0) as u16;
        let h = om_read8!($c, $b; $address + 1) as u16;

        l | (h << 8)
    }
});

/// 16-bit Read Next/Immediate {+1}
macro_rules! om_read_next16 (($c:ident, $b:ident) => {
    {
        let r = om_read16!($c, $b; $c.pc);
        $c.pc += 2;

        r
    }
});

/// 16-bit Write {+2}
macro_rules! om_write16 (($c:ident, $b:ident; $address:expr, $value:expr) => {
    {
      om_write8!($c, $b; $address + 1, ($value >> 8) as u8);
      om_write8!($c, $b; $address + 0, ($value & 0xFF) as u8);
    }
});

// 16-bit Arithmetic/Logical

/// 16-bit Increment [----] {+1}
macro_rules! om_inc16 (($c:ident, $b:ident; $get:ident, $set:ident) => {
    let r = $c.$get();

    $c.$set(r + 1);
    $c.step($b);
});

/// 16-bit Decrement [----] {+1}
macro_rules! om_dec16 (($c:ident, $b:ident; $get:ident, $set:ident) => {
    let r = $c.$get();

    $c.$set(r - 1);
    $c.step($b);
});

/// 16-bit Add (to HL) [-0hc] {+1}
macro_rules! om_add16_hl (($c:ident, $b:ident; $get:ident) => {
    let a = $c.get_hl();
    let b = $c.$get();
    let r = a as u32 + b as u32;

    $c.set_flag(cpu::HALF_CARRY, ((a ^ b ^ ((r & 0xFFFF) as u16)) & 0x1000) != 0);
    $c.set_flag(cpu::CARRY, r > 0xFFFF);
    $c.set_flag(cpu::ADD_SUBTRACT, false);

    $c.set_hl((r & 0xFFFF) as u16);
    $c.step($b);
});

// Operation Macros
// Instructions are broken down into reusable macros that allow zero-cost code reuse.

/// 8-bit Write {+1}
macro_rules! om_write8 (($c:ident, $b:ident; $address:expr, $value:expr) => {
    {
        $c.step($b);
        $b.write($address, $value)
    }
});

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

/// 16-bit Increment [----]
macro_rules! om_inc16 (($c:ident, $b:ident; $get:ident, $set:ident) => {
    let r = $c.$get();

    $c.$set(r + 1);
    $c.step($b);
});

/// 16-bit Decrement [----]
macro_rules! om_dec16 (($c:ident, $b:ident; $get:ident, $set:ident) => {
    let r = $c.$get();

    $c.$set(r - 1);
    $c.step($b);
});

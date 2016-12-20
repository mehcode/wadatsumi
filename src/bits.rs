
/// Return an integer with the passed bit 1 and the remaining bits 0
pub fn bit(value: bool, n: u8) -> u8 {
    (if value { 1 } else { 0 }) << n
}

// Returns the boolean value of the <n>th bit from <value>
pub fn test(value: u8, n: u8) -> bool {
    (value & (1 << n)) != 0
}

// Returns a <n> bit integer with all bits set to 1
// TODO: Please think of a better name and perhaps a better algorithm
pub fn mask(n: u8) -> u8 {
    let mut r = 0;
    for i in 0..n {
        r |= bit(true, i);
    }

    r
}

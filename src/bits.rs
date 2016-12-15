
/// Return an integer with the passed bit 1 and the remaining bits 0
pub fn bit(value: bool, n: u8) -> u8 {
    (if value { 1 } else { 0 }) << n
}

// Returns the boolean value of the <n>th bit from <value>
pub fn test(value: u8, n: u8) -> bool {
    (value & (1 << n)) != 0
}

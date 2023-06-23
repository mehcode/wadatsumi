const RAM_SIZE: usize = 0x1000;

const ENTRY_POINT: usize = 0x200;

#[derive(Debug)]
pub struct State {
    /// The display is 64x32 monochrome pixels in the standard CHIP-8.
    pub display: Vec<bool>,

    /// CHIP-8 has direct access to up to 4 kB of RAM.
    pub ram: Box<[u8; RAM_SIZE]>,

    /// A program counter (PC) which points at the current instruction in memory.
    pub pc: usize,

    /// 16 8-bit (1 byte) general-purpose variable registers called
    /// `V0` through `VF`.
    pub v: [u8; 0x10],

    /// A 16-bit index (I) register. Generally used to point at locations
    /// in memory.
    pub i: u16,

    /// Stack of 16-byte addresses. Used for subroutine nesting.
    pub stack: Vec<u16>,
}

impl State {
    pub fn new() -> Self {
        Self {
            display: vec![false; 64 * 32],
            pc: ENTRY_POINT,
            // do the dance to get a heap-allocated fixed-size array
            // sure would be nice if we had a better way to write that
            ram: vec![0; RAM_SIZE].into_boxed_slice().try_into().unwrap(),
            v: [0; 0x10],
            i: 0,
            stack: Vec::with_capacity(12),
        }
    }
}

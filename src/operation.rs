use std::vec;
use ::op;
use ::cpu;

pub struct Operation {
    // Function to handle the operation
    pub handle: fn(&mut cpu::CPU) -> (),

    // String format of operation for disassembly
    pub disassembly: &'static str,

    // Number of bytes (incl. opcode); used for disassembly
    pub size: u8,
}

impl Operation {
    fn new(handle: fn(&mut cpu::CPU) -> (), disassembly: &'static str, size: u8) -> Self {
        return Operation {
            handle: handle,
            disassembly: disassembly,
            size: size,
        };
    }
}

// Operation table
pub struct Table {
    // Operation table
    //  + 0x000 - $00 - $FF
    //  + 0x100 - $CB00 - $CBFF
    operations: vec::Vec<Operation>,
}

impl Table {
    // Make a new operation table (and fill its contents)
    pub fn new() -> Self {
        return Table {
            operations: vec![Operation::new(op::_00, "NOP", 1),
                             Operation::new(op::_01, "LD BC, {a:#X}", 2)],
        };
    }

    // Return the next operation
    // TODO: Needs access to PC and the MMU
    pub fn next(&self) -> &Operation {
        return &self.operations[1];
    }
}

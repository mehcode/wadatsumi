use std::vec;
use std::ops::Index;
use std::fmt::Write;
use std::string::String;
use strfmt;

use ::op;
use ::bus::Bus;
use ::cpu::Context;

pub struct Operation {
    // Function to handle the operation
    pub handle: fn(&mut Context, &mut Bus) -> (),

    // String format of operation for disassembly
    pub disassembly: &'static str,

    // Number of bytes (incl. opcode); used for disassembly
    pub size: u8,
}

impl Operation {
    fn new(handle: fn(&mut Context, &mut Bus) -> (), disassembly: &'static str, size: u8) -> Self {
        Operation {
            handle: handle,
            disassembly: disassembly,
            size: size,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn format(&self, _: &Context, _: &mut Bus) -> Result<String, strfmt::FmtError> {
        strfmt::strfmt_map(self.disassembly,
                           &|mut fmt: strfmt::Formatter| {
            // TODO(rust): This library seems to want me to use unwrap here which smells
            if fmt.key == "0" {
                fmt.write_str("?").unwrap()
            } else {
                fmt.write_str("??").unwrap()
            }

            Ok(())
        })
    }
}

pub struct Table {
    // Operation table
    //  + 0x000 - $00 - $FF
    //  + 0x100 - $CB00 - $CBFF
    operations: vec::Vec<Operation>,
}

impl Default for Table {
    fn default() -> Self {
        Table {
            operations: vec![Operation::new(op::_00, "NOP", 1),
                             Operation::new(op::_01, "LD BC, 0x{1:X}{0:X}", 3),
                             Operation::new(op::_02, "LD (BC), A", 1),
                             Operation::new(op::_03, "INC BC", 1),
                             Operation::new(op::_04, "INC B", 1),
                             Operation::new(op::_05, "DEC B", 1),
                             Operation::new(op::_06, "LD B, 0x{0:X}", 2),
                             Operation::new(op::_07, "RLCA", 1),
                             Operation::new(op::_08, "LD (0x{1:X}{0:X}), SP", 3),
                             Operation::new(op::_09, "ADD HL, BC", 1),
                             Operation::new(op::_0A, "LD A, (BC)", 1),
                             Operation::new(op::_0B, "DEC BC", 1),
                             Operation::new(op::_0C, "INC C", 1),
                             Operation::new(op::_0D, "DEC C", 1),
                             Operation::new(op::_0E, "LD C, 0x{0:X}", 2),
                             Operation::new(op::_0F, "RRCA", 1)],
        }
    }
}

impl Index<usize> for Table {
    type Output = Operation;

    fn index(&self, index: usize) -> &Operation {
        &self.operations[index]
    }
}

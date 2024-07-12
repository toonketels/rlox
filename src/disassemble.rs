use crate::chunk::Chunk;
use crate::codes::Byte;
use crate::opcode::OpCode;

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut n = 0;
        loop {
            let Some(code) = self.read_byte(n) else {
                break;
            };
            n = self.disassemble_instruction(code, n);
        }
    }

    // Returns the next instruction location
    pub fn disassemble_instruction(&self, byte: Byte, at: usize) -> usize {
        use OpCode::*;

        let line = self.lines.at(at);

        match OpCode::try_from(byte).expect("Not an opcode") {
            Constant => {
                let c = self
                    .read_constant(at + 1)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", at + 1));

                println!("{:8} {:8} | Constant {:?}", at, line, c);

                at + 2
            }
            Add => Self::simple_instruction("Add", at, line),
            Subtract => Self::simple_instruction("Subtract", at, line),
            Multiply => Self::simple_instruction("Multiply", at, line),
            Divide => Self::simple_instruction("Divide", at, line),
            Negate => Self::simple_instruction("Negate", at, line),
            Return => Self::simple_instruction("Return", at, line),
        }
    }

    fn simple_instruction(name: &str, at: usize, line: usize) -> usize {
        println!("{:8} {:8} | {}", at, line, name);
        at + 1
    }
}

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
        let line = self.lines.at(at);

        match OpCode::try_from(byte) {
            Ok(OpCode::Constant) => {
                let c = self
                    .read_constant(at + 1)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", at + 1));

                println!("{:8} {:8} | Constant {:?}", at, line, c);

                at + 2
            }
            Ok(OpCode::Return) => {
                println!("{:8} {:8} | Return", at, line);
                at + 1
            }

            Err(_) => {
                panic!("Not an opcode")
            }
        }
    }
}

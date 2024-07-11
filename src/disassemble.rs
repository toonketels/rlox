use crate::chunk::{Chunk, OpCode};
use crate::codes::Byte;

impl Chunk {

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut n = 0;
        loop {
            let Some(code) = self.code.get(n) else {
                break;
            };
            n = self.disassemble_instruction(code, n);
        }
    }

    // Returns the next instruction location
    fn disassemble_instruction(&self, byte: Byte, at: usize) -> usize {
        let line = self.lines.at(at);

        match OpCode::try_from(byte) {
            Ok(OpCode::Constant) => {
                let i = self
                    .code
                    .get(at + 1)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", at + 1));

                let index = i as usize;

                let c = self.constants.at(index);

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
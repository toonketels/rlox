use crate::chunk::Chunk;
use crate::opcode::{Byte, OpCode};
use std::io;
use std::io::{Cursor, Write};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        let mut buffer = io::stdout();
        self.disassemble_buffer(&mut buffer, name)
    }

    pub fn disassemble_into_string(&self, name: &str) -> String {
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        self.disassemble_buffer(&mut buffer, name);

        String::from_utf8(buffer.into_inner()).unwrap()
    }

    pub fn disassemble_instruction(&self, byte: Byte, at: usize) -> usize {
        let mut buffer = io::stdout();
        self.disassemble_instruction_buffer(&mut buffer, byte, at)
    }

    fn disassemble_buffer<W: Write>(&self, buffer: &mut W, name: &str) {
        writeln!(buffer);
        writeln!(buffer, "== {} ==", name);

        let mut n = 0;
        loop {
            let Some(code) = self.read_byte(n) else {
                break;
            };
            n = self.disassemble_instruction_buffer(buffer, code, n);
        }
    }

    // Returns the next instruction location
    fn disassemble_instruction_buffer<W: Write>(
        &self,
        buffer: &mut W,
        byte: Byte,
        at: usize,
    ) -> usize {
        use OpCode::*;

        let line = self.lines.at(at);

        match OpCode::try_from(byte).expect("Not an opcode") {
            Constant => {
                let c = self
                    .read_constant(at + 1)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | Constant {:?}", at, line, c);

                at + 2
            }

            // literals
            False => Self::simple_instruction("False", buffer, at, line),
            True => Self::simple_instruction("True", buffer, at, line),
            Nil => Self::simple_instruction("Nil", buffer, at, line),
            String => {
                let c = self
                    .read_string(at + 1)
                    .unwrap_or_else(|| panic!("String at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | String {:?}", at, line, c);

                at + 2
            }

            // comparison
            Equal => Self::simple_instruction("Equal", buffer, at, line),
            Greater => Self::simple_instruction("Greater", buffer, at, line),
            Less => Self::simple_instruction("Less", buffer, at, line),

            // unary
            Not => Self::simple_instruction("Not", buffer, at, line),

            // mathematical
            Add => Self::simple_instruction("Add", buffer, at, line),
            Subtract => Self::simple_instruction("Subtract", buffer, at, line),
            Multiply => Self::simple_instruction("Multiply", buffer, at, line),
            Divide => Self::simple_instruction("Divide", buffer, at, line),
            Negate => Self::simple_instruction("Negate", buffer, at, line),

            // bindings
            DefineGlobal => {
                let c = self
                    .read_string(at + 1)
                    .unwrap_or_else(|| panic!("String at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | Global define {:?}", at, line, c);

                at + 2
            }
            GetGlobal => {
                let c = self
                    .read_string(at + 1)
                    .unwrap_or_else(|| panic!("String at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | Global get {:?}", at, line, c);

                at + 2
            }
            SetGlobal => {
                let c = self
                    .read_string(at + 1)
                    .unwrap_or_else(|| panic!("String at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | Global set {:?}", at, line, c);

                at + 2
            }

            // statements
            Print => Self::simple_instruction("Print", buffer, at, line),
            Pop => Self::simple_instruction("Pop", buffer, at, line),
            Return => Self::simple_instruction("Return", buffer, at, line),
        }
    }

    fn simple_instruction<W: Write>(name: &str, buffer: &mut W, at: usize, line: usize) -> usize {
        writeln!(buffer, "{:8} {:8} | {}", at, line, name);
        at + 1
    }
}

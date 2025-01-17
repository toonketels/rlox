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
        writeln!(buffer).unwrap();
        writeln!(buffer, "== {} ==", name).unwrap();

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

                writeln!(buffer, "{:8} {:8} | Constant {:?}", at, line, c).unwrap();

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

                writeln!(buffer, "{:8} {:8} | String {:?}", at, line, c).unwrap();

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

                writeln!(buffer, "{:8} {:8} | Global define {:?}", at, line, c).unwrap();

                at + 2
            }
            GetGlobal => {
                let c = self
                    .read_string(at + 1)
                    .unwrap_or_else(|| panic!("String at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | Global get {:?}", at, line, c).unwrap();

                at + 2
            }
            SetGlobal => {
                let c = self
                    .read_string(at + 1)
                    .unwrap_or_else(|| panic!("String at index {:?} should exist", at + 1));

                writeln!(buffer, "{:8} {:8} | Global set {:?}", at, line, c).unwrap();

                at + 2
            }
            GetLocal => {
                let index = self.read_byte(at + 1).unwrap();

                writeln!(
                    buffer,
                    "{:8} {:8} | Local var get index({:?})",
                    at, line, index
                )
                .unwrap();
                at + 2
            }
            SetLocal => {
                let index = self.read_byte(at + 1).unwrap();
                writeln!(
                    buffer,
                    "{:8} {:8} | Local var set index({:?})",
                    at, line, index
                )
                .unwrap();
                at + 2
            }

            // control flow
            JumpIfFalse => self.jump_instruction("If (false) jump", buffer, at, line),
            JumpIfTrue => self.jump_instruction("If (true) jump", buffer, at, line),
            Jump => self.jump_instruction("Jump", buffer, at, line),
            Loop => self.loop_instruction(buffer, at, line),

            // statements
            Print => Self::simple_instruction("Print", buffer, at, line),
            Pop => Self::simple_instruction("Pop", buffer, at, line),
            Return => Self::simple_instruction("Return", buffer, at, line),
        }
    }

    fn simple_instruction<W: Write>(name: &str, buffer: &mut W, at: usize, line: usize) -> usize {
        writeln!(buffer, "{:8} {:8} | {}", at, line, name)
            .expect("simple instruction write to buffer");
        at + 1
    }

    fn jump_instruction<W: Write>(
        &self,
        name: &str,
        buffer: &mut W,
        at: usize,
        line: usize,
    ) -> usize {
        let it = self
            .read_jump(at + 1)
            .unwrap_or_else(|| panic!("Jump at index {:?} should exist", at + 1));
        let adjust_for_jump_byte_width = 2;
        let adjust_for_ip_points_to_next = 1;
        writeln!(
            buffer,
            "{:8} {:8} | {} to {:?}",
            at,
            line,
            name,
            it.distance as usize + at + adjust_for_jump_byte_width + adjust_for_ip_points_to_next
        )
        .unwrap();
        at + 3
    }

    fn loop_instruction<W: Write>(&self, buffer: &mut W, at: usize, line: usize) -> usize {
        let it = self
            .read_jump(at + 1)
            .unwrap_or_else(|| panic!("Jump at index {:?} should exist", at + 1));
        let adjust_for_jump_byte_width = 2;
        let adjust_for_ip_points_to_next = 1;
        writeln!(
            buffer,
            "{:8} {:8} | Loop back to {:?}",
            at,
            line,
            at - it.distance as usize + adjust_for_jump_byte_width + adjust_for_ip_points_to_next
        )
        .unwrap();
        at + 3
    }
}

#[derive(Debug)]
pub enum OpCode {
    // @TODO consider moving the usize out and in write to OpCode | usize
    //
    Constant(usize),
    Return,
}

type Value = f64;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    // Tracks the src line the corresponding opcode refers to for error reporting
    lines: Vec<usize>,
}

impl Chunk {
    pub fn write(&mut self, op_code: OpCode, line: usize) {
        self.code.push(op_code);
        // Keeps track which src line this belongs to
        self.lines.insert(self.code.len() - 1, line);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("== {} ==", name);

        self.code
            .iter()
            .enumerate()
            .for_each(|(index, op_code)| self.disassemble_instruction(op_code, index));
    }

    fn disassemble_instruction(&self, op_code: &OpCode, at: usize) {
        let line = self
            .lines
            .get(at)
            .unwrap_or_else(|| panic!("Line at index {:?} should exist", at));
        match op_code {
            OpCode::Constant(i) => {
                let c = self
                    .constants
                    .get(*i)
                    .unwrap_or_else(|| panic!("Constant at index {:?} should exist", i));
                println!("{:8} {:8} | Constant {:?}", at, line, c)
            }
            OpCode::Return => println!("{:8} {:8} | Return", at, line),
        }
    }
}

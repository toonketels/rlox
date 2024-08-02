use crate::opcode::Byte;

/// Codes is a byte array of machine code

#[derive(Debug)]
pub struct Codes(Vec<Byte>);

impl Codes {
    pub fn new() -> Self {
        Codes(Vec::new())
    }

    pub fn get(&self, index: usize) -> Option<Byte> {
        self.0.get(index).copied()
    }

    /// Returns the index to lookup the byte again
    pub fn add(&mut self, byte: Byte) -> usize {
        self.0.push(byte);
        self.0.len() - 1
    }

    pub fn patch(&mut self, at: usize, byte: Byte) -> usize {
        self.0[at] = byte;
        at
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

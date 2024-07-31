use crate::opcode::Value;

/// Constants contain all the constants in use by the program.

#[derive(Debug)]
pub(crate) struct Constants(Vec<Value>);

impl Constants {
    pub fn new() -> Self {
        Constants(Vec::new())
    }

    /// Returns the index to lookup the constant again
    pub fn add(&mut self, value: Value) -> usize {
        self.0.push(value);
        self.0.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        // Since we are using an rc, we can no longer use copied().
        self.0.get(index).cloned()
    }
}

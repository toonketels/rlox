use crate::opcode::Value;

pub struct Stack(Vec<Value>);

impl Stack {
    pub fn new() -> Self {
        Stack(Vec::new())
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.0.pop()
    }

    pub fn peek(&self, offset: usize) -> Option<&Value> {
        // Peek from the back of the vec as values are popped from the back
        let offset = self.0.len() - 1 - offset;
        self.0.get(offset)
    }
}

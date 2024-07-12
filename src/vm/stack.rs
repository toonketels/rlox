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
}

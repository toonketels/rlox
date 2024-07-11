pub type Value = f64;
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

    pub fn at(&self, index: usize) -> Value {
        let value = self
            .0
            .get(index)
            .unwrap_or_else(|| panic!("Constant at index {:?} should exist", index));

        *value
    }
}

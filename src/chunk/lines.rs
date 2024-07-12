/// Lines keep track of the line number corresponding to the opcode

#[derive(Debug)]
pub struct Lines(Vec<usize>);

impl Lines {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, index: usize, element: usize) {
        self.0.insert(index, element)
    }

    pub fn at(&self, index: usize) -> usize {
        let line = self
            .0
            .get(index)
            .unwrap_or_else(|| panic!("Line at index {:?} should exist", index));

        *line
    }
}

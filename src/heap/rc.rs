use crate::opcode::Obj;
use std::rc::Rc;

// Safe pointer heap implementation that works with rc.
// - Pro:
//   1. uses pointers so we can deref easily
//   2. safe rust
//   3. idiomatic rust?
// - Cons:
//   1. uses more memory (RcBox)
//   2. since we create a garbage collector to manage the memory,
//      using a rc in addition might be too much?
//   3. Value can no longer implement copy and we need to clone explicitly

pub struct RcHeap {
    objects: Vec<Rc<Obj>>,
}

impl RcHeap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn alloc(&mut self, object: Obj) -> Rc<Obj> {
        let it = Rc::new(object);
        self.objects.push(Rc::clone(&it));
        Rc::clone(&it)
    }

    pub fn free_all(&mut self) {
        self.objects.clear();
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }
}

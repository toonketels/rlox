#![allow(dead_code)]

use crate::opcode::Obj;

// Heap implementation that just returns an offset into the heap (it 'address')
// - Pro:
//  1. no ownership issues as we are just passing a usize
//  2. works even after heap amortizes and objects move around in memory
// - Cons: we need access to heap to deref the pointer. Works in the vm, but makes bugging harder.

pub struct OffsetHeap {
    objects: Vec<Obj>,
}

impl OffsetHeap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn alloc(&mut self, object: Obj) -> usize {
        let at = self.objects.len();
        self.objects.push(object);
        at
    }

    pub fn free_all(&mut self) {
        self.objects.clear();
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }
}

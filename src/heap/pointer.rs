use crate::opcode::Obj;
use std::ptr::NonNull;

// Unsafe pointer heap implementation that works with raw pointers.
// - Pro:
//   1. least amount of memory used
//   2. closer to reference implementation
//   3. uses pointers so we can deref easily
// - Cons:
//   1. unsafe

pub struct PointerHeap {
    objects: Vec<Pointer>,
}

// NewType around NonNull to make dereferencing easier
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Pointer {
    pointer: NonNull<Obj>,
}

impl Pointer {
    fn new(object: Obj) -> Self {
        let it = Box::new(object);
        let pointer = unsafe { NonNull::new_unchecked(Box::into_raw(it)) };
        Self { pointer }
    }

    pub fn as_ref(&self) -> &Obj {
        unsafe { self.pointer.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut Obj {
        unsafe { self.pointer.as_mut() }
    }
}

impl PointerHeap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn alloc(&mut self, object: Obj) -> Pointer {
        let it = Pointer::new(object);
        self.objects.push(it);
        it
    }

    pub fn free_all(&mut self) {
        self.objects.clear();
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }
}

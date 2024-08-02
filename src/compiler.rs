use crate::vm::CompilationErrorReason::ScopeUnderflow;
use crate::vm::InterpretError;
use crate::vm::InterpretError::{CompileError, RuntimeErrorWithReason};

// Tracks variable name and its scope depth
#[derive(Debug)]
pub struct LocalVar {
    name: String,
    scope_depth: i32,
}

impl LocalVar {
    pub fn new(name: String, scope_depth: i32) -> Self {
        Self { name, scope_depth }
    }
}

pub enum LocalVarResolution {
    NotFound,
    FoundAt(usize),
}

// Structure to aid compile time optimizations instead of deferring computations till run time
#[derive(Debug)]
pub struct Compiler {
    locals: Vec<LocalVar>,
    scope_depth: i32,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            locals: Vec::with_capacity(u8::MAX as usize),
            scope_depth: 0,
        }
    }

    pub fn begin_scope(&mut self) -> Result<(), InterpretError> {
        self.scope_depth += 1;
        Ok(())
    }

    pub fn end_scope(&mut self) -> Result<usize, InterpretError> {
        if self.scope_depth < 1 {
            Err(CompileError(ScopeUnderflow))?
        }

        let mut count = 0;
        for v in self.locals.iter().rev() {
            if v.scope_depth == self.scope_depth {
                count += 1;
            }
        }

        let mut pop = 0;

        while pop < count {
            self.locals.pop();
            pop += 1;
        }

        self.scope_depth -= 1;

        Ok(count)
    }

    pub fn in_local_scope(&mut self) -> bool {
        self.scope_depth > 0
    }

    pub fn add_local_var(&mut self, name: String) -> Result<usize, InterpretError> {
        if self.is_in_scope_name_collision(name.as_str()) {
            Err(RuntimeErrorWithReason(
                "Already a variable with this name in this scope",
            ))?
        }
        let at = self.locals.len();
        self.locals.push(LocalVar::new(name, self.scope_depth));
        Ok(at)
    }

    fn is_in_scope_name_collision(&self, name: &str) -> bool {
        // Start looking from the current scope which is at the end
        for v in self.locals.iter().rev() {
            // if we are in a lower scope already, the variable has not been found in the current scope
            if v.scope_depth < self.scope_depth {
                return false;
            }
            // we are still in the current scope, var has been found: collision!
            if v.name == name {
                return true;
            }
        }
        false
    }

    // The trick here is that our local vars mirror the stack so the index
    // corresponds one on one the index on the stack
    //
    // Might no longer be true once we start pushing complete stack frames
    pub fn resolve_local_variable(&self, name: &str) -> LocalVarResolution {
        // Walk from the back because we allow shadowing so we need to variable from the highest scope first
        for (i, v) in self.locals.iter().enumerate().rev() {
            if v.name == name {
                return LocalVarResolution::FoundAt(i);
            }
        }
        LocalVarResolution::NotFound
    }
}

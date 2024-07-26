use std::mem;

/// OpCodes used by our vm.

// Each opcode is a byte
pub type Byte = u8;

// Constants etc.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn as_number(&self) -> f64 {
        if let Value::Number(it) = self {
            *it
        } else {
            panic!("Value is not a number")
        }
    }

    pub fn as_bool(&self) -> bool {
        if let Value::Bool(it) = self {
            *it
        } else {
            panic!("Value is not a bool")
        }
    }

    pub fn as_nil(&self) {
        if self.is_nil() {
        } else {
            panic!("Value is not a nil")
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Constant,

    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,

    Return,
}

impl TryFrom<Byte> for OpCode {
    type Error = ();

    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        match value {
            b if b <= OpCode::Return as Byte => unsafe { Ok(mem::transmute::<u8, OpCode>(value)) },
            _ => Err(()),
        }
    }
}

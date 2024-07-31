use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;

/// OpCodes used by our vm.

// Each opcode is a byte
pub type Byte = u8;

#[derive(Debug, PartialEq, Clone)]
pub enum Obj {
    // str itself is heap allocated
    String { str: String },
}

impl Obj {
    pub fn is_string(&self) -> bool {
        matches!(self, Obj::String { str })
    }

    pub fn as_string(&self) -> &str {
        if let Obj::String { str } = self {
            str.as_ref()
        } else {
            panic!("Obj is not a string")
        }
    }
}

// Constants etc.
#[derive(Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Object(Rc<Obj>),
    Nil,
}

// An owned version of value so we can clean up the heap and return the value
#[derive(Clone, PartialEq, Debug)]
pub enum Returned {
    Number(f64),
    Bool(bool),
    Object(Obj),
    Nil,
}

impl From<Value> for Returned {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(it) => Returned::Number(it),
            Value::Bool(it) => Returned::Bool(it),
            Value::Object(it) => {
                let it = it.as_ref();
                Returned::Object(it.clone())
            }
            Value::Nil => Returned::Nil,
        }
    }
}

impl From<&str> for Returned {
    fn from(it: &str) -> Self {
        Self::Object(Obj::String {
            str: it.to_string(),
        })
    }
}

impl From<f64> for Returned {
    fn from(it: f64) -> Self {
        Self::Number(it)
    }
}

impl From<bool> for Returned {
    fn from(it: bool) -> Self {
        Self::Bool(it)
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(it) => write!(f, "{:?}", it),
            Value::Bool(it) => write!(f, "{:?}", it),
            Value::Object(it) => write!(f, "Object({:?})", *it.as_ref()),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_string(&self) -> bool {
        if let Value::Object(it) = self {
            it.is_string()
        } else {
            false
        }
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    // Note, our definition is a bit different from the book
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(it) => *it,
            Value::Number(it) => *it != 0.0, // all number are truthy expect for 0
            Value::Object(it) => false,      // @TODO revisit it
        }
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

    pub fn as_string(&self) -> &str {
        if let Value::Object(it) = self {
            it.as_string()
        } else {
            panic!("Value is not a string")
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

    // literals
    Nil,
    True,
    False,

    // static strings
    // not in book, might be a bad idea
    String,

    // comparison
    Equal,
    Greater,
    Less,

    // unary
    Not,

    // mathematical
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,

    Print,

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

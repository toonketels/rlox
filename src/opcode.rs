use std::mem;

/// OpCodes used by our vm.

// Each opcode is a byte
pub type Byte = u8;

// Constants etc.
pub type Value = f64;

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

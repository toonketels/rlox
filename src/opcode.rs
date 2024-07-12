use crate::codes::Byte;
use std::mem;

/// OpCodes used by our vm.
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
            b if b <= OpCode::Return as Byte => unsafe { Ok(mem::transmute(value)) },
            _ => Err(()),
        }

        // match value {
        //     0 => Ok(Constant),
        //
        //     1 => Ok(Add),
        //     2 => Ok(Subtract),
        //     3 => Ok(Multiply),
        //     4 => Ok(Divide),
        //     5 => Ok(Negate),
        //
        //     6 => Ok(Return),
        //     _ => Err(()),
        // }
    }
}

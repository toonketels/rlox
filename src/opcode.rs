use crate::codes::Byte;

/// OpCodes used by our vm.
#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return,
}

impl TryFrom<Byte> for OpCode {
    type Error = ();

    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        use crate::opcode::OpCode::*;

        match value {
            0 => Ok(Constant),
            1 => Ok(Return),
            _ => Err(()),
        }
    }
}

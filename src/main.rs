use rlox::chunk::Chunk;
use rlox::opcode::OpCode::Return;
use rlox::vm::{interpret, InterpretError};

fn main() -> Result<(), InterpretError> {
    let mut x = Chunk::new();

    x.write_constant(54.0, 1);
    x.write_code(Return, 2);

    x.disassemble("program");

    println!();
    println!("== VM Run ==");

    interpret(&x)
}

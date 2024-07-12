use rlox::chunk::Chunk;
use rlox::opcode::OpCode;
use rlox::vm::{interpret, InterpretError};

fn main() -> Result<(), InterpretError> {
    let mut x = Chunk::new();

    use OpCode::*;

    x.write_constant(1.2, 1);
    x.write_constant(3.4, 1);
    x.write_code(Add, 1);

    x.write_constant(5.6, 1);
    x.write_code(Divide, 1);
    x.write_code(Negate, 1);

    x.write_code(Return, 2);

    x.disassemble("program");

    println!();
    println!("== VM Run ==");

    interpret(&x)
}

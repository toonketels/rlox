use rlox::chunk::OpCode::Return;
use rlox::chunk::{Chunk, OpCode};

fn main() {
    let mut x = Chunk::new();

    x.write_constant(54.0, 1);
    x.write_code(Return, 2);

    x.disassemble_chunk("my program");

    println!("Size of op_codes: {}", std::mem::size_of::<OpCode>());

    println!("{:?}", x)
}

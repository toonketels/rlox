use rlox::chunk::OpCode::{Constant, Return};
use rlox::chunk::{Chunk, OpCode};

fn main() {
    let mut x = Chunk::new();

    let v = x.add_constant(54.0);
    x.write(Constant(v), 1);
    x.write(Return, 2);

    x.disassemble_chunk("my program");

    println!("Size of op_codes: {}", std::mem::size_of::<OpCode>());

    println!("{:?}", x)
}

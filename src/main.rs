use rlox::chunk::Chunk;
use rlox::opcode::OpCode;
use rlox::opcode::OpCode::Return;

fn main() {
    let mut x = Chunk::new();

    x.write_constant(54.0, 1);
    x.write_code(Return, 2);

    x.disassemble("my program");

    println!("Size of op_codes: {}", std::mem::size_of::<OpCode>());

    println!("{:?}", x)
}

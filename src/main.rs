use rlox::reader::run_file;
use rlox::repl::repl;
use rlox::vm::InterpretError;
use std::env::args;

fn main() -> Result<(), InterpretError> {
    let arguments = args().collect::<Vec<String>>();
    match &arguments[..] {
        [_] => repl(),
        [_, path] => run_file(path),
        _ => {
            println!("Usage: rlox [path]");
            Ok(())
        }
    }

    //
    //
    // let mut x = Chunk::new();
    //
    // use OpCode::*;
    //
    // x.write_constant(1.2, 1);
    // x.write_constant(3.4, 1);
    // x.write_code(Add, 1);
    //
    // x.write_constant(5.6, 1);
    // x.write_code(Divide, 1);
    // x.write_code(Negate, 1);
    //
    // x.write_code(Return, 2);
    //
    // x.disassemble("program");
    //
    // println!();
    // println!("== VM Run ==");
    //
    // interpret(&x)
}

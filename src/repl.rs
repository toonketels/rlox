use crate::vm::InterpretError;
use std::io::{stdin, stdout, Write};

pub fn repl() -> Result<(), InterpretError> {
    let mut line = String::new();

    println!("> Rlox repl:");
    loop {
        print!("> ");
        stdout().flush();
        stdin().read_line(&mut line)?;
        let input = line.clone();
        line.clear();
        interpret(input);
    }
    Ok(())
}

fn interpret(line: String) -> Result<(), InterpretError> {
    print!("> PRINTED {}", line);
    Ok(())
}

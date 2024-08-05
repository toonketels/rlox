use crate::parser::Parser;
use crate::tokenizer::Tokenizer;
use crate::vm::{interpret, InterpretError};
use std::io::{stdin, stdout, Write};

pub fn repl() -> Result<(), InterpretError> {
    let mut line = String::new();

    println!("> Rlox repl:");
    loop {
        print!("> ");
        stdout().flush()?;
        stdin().read_line(&mut line)?;
        let input = line.clone();
        line.clear();
        interpret_line(input)?;
    }
}

// Dummy implementation that evaluates just the current line, not taking into account
// what came before it.
fn interpret_line(line: String) -> Result<(), InterpretError> {
    let chunk = Parser::parse(Tokenizer::new(&line))?;
    let result = interpret(&chunk)?;
    print!("> PRINTED {:?}", result);
    Ok(())
}

use crate::vm::InterpretError;
use std::fs;

pub fn run_file(path: &str) -> Result<(), InterpretError> {
    println!("Reading file from path {}", path);

    let buffer = fs::read_to_string(path)?;

    interpret(&buffer)
}

fn interpret(source: &str) -> Result<(), InterpretError> {
    for line in source.lines() {
        println!("{}", line);
    }

    Ok(())
}

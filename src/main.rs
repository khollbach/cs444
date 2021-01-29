use std::error::Error;
use std::io::{self, prelude::*};
use tokenizer::Tokenizer;

mod tokenizer;

type Res<T> = Result<T, Box<dyn Error>>;

// todo: This is just placeholder code for now.
// It reads stdin, and then runs the tokenizer on it.
fn main() -> Res<()> {
    let lines: Vec<String> = io::stdin()
        .lock()
        .lines()
        .collect::<Result<_, io::Error>>()?;

    let t = Tokenizer::new();
    for token in t.tokenize(lines.iter().map(|s| s.as_str())) {
        println!("{:?}", token);
    }

    Ok(())
}

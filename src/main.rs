use std::error::Error;
use std::io::{self, prelude::*};
use tokenizer::Tokenizer;

mod tokenizer;
mod types;

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    // todo:
    // This is just placeholder code for now.
    // It slurps stdin to a string, and runs the tokenizer on it.

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(&buf);
    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}

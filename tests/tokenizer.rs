use cs444::tokenizer::{Token, TokenInfo, TokenOrComment, Tokenizer};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use walkdir::WalkDir;

type Res<T> = Result<T, Box<dyn Error>>;

/// Run `tokenize_and_echo` to validate all the `.java` files in `tests/good-examples`.
#[test]
fn good_examples() -> Res<()> {
    let base_dir = format!("{}/tests/good-examples", env!("CARGO_MANIFEST_DIR"));

    let mut tests_run = 0;

    let tokenizer = Tokenizer::new();
    for entry in WalkDir::new(base_dir) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(OsStr::to_str) == Some("java")
        {
            tokenize_and_echo(&tokenizer, entry.path())?;
            tests_run += 1;
        }
    }

    // Sanity check that the above loop actually did something.
    assert!(tests_run >= 10);

    Ok(())
}

/// Tokenize the input file, then write these tokens back to a string. Compare the output string
/// against the original input: the two should be equal.
///
/// Panics or returns an error on failure.
fn tokenize_and_echo(tokenizer: &Tokenizer, input_file: impl AsRef<Path>) -> Res<()> {
    let input = BufReader::new(File::open(&input_file)?);
    let input: Vec<String> = input.lines().collect::<Result<_, _>>()?;
    let mut output = Vec::<String>::with_capacity(input.len());

    for elem in tokenizer.tokenize_keep_comments(input.iter().map(String::as_str)) {
        // todo: sanity_check(&token);

        echo_elem(&input, elem, &mut output);
    }

    // Add trailing blank lines to output.
    while output.len() < input.len() {
        let line_len = input[output.len()].len();
        output.push(String::with_capacity(line_len));
    }

    // Add trailing whitespace to output lines.
    for (line, buf) in input.iter().zip(output.iter_mut()) {
        assert!(line.len() >= buf.len(), "line: {}\nbuf: {}", line, buf);
        let num_spaces = line.len() - buf.len();
        buf.push_str(&" ".repeat(num_spaces));
    }

    assert_eq!(input.len(), output.len());
    for (i, (expected, actual)) in input.iter().zip(output.iter()).enumerate() {
        assert_eq!(
            expected,
            actual,
            "line {} of {:?}",
            i + 1,
            input_file.as_ref()
        );
    }
    Ok(())
}

/// Append an input element (token or comment) to the output buffer.
///
/// Adds whitespace preceeding the element, according to its start position in the line.
fn echo_elem(input: &[String], elem: TokenOrComment, output: &mut Vec<String>) {
    // Add newline(s) to `output` to make room for `elem`.
    while elem.start().line_num >= output.len() {
        let line_len = input[output.len()].len();
        output.push(String::with_capacity(line_len));
    }

    let buf = output.iter_mut().last().unwrap();

    // Add spaces preceeding `elem`.
    assert!(elem.start().col >= buf.len());
    let num_spaces = elem.start().col - buf.len();
    buf.push_str(&" ".repeat(num_spaces));

    match elem {
        TokenOrComment::Token(token) => {
            buf.push_str(token.lexeme);
        }
        TokenOrComment::LineComment { start } => {
            let lexeme = &start.line[start.col..];
            buf.push_str(lexeme);
        }
        TokenOrComment::StarComment {
            start,
            end_inclusive,
        } => {
            // One-line.
            if start.line_num == end_inclusive.line_num {
                let lexeme = &start.line[start.col..=end_inclusive.col];
                buf.push_str(lexeme);
            }
            // Multi-line.
            else {
                let first = &start.line[start.col..];
                buf.push_str(first);

                // (Middle lines.)
                for i in start.line_num + 1..end_inclusive.line_num {
                    output.push(String::from(&input[i]));
                }

                let last = &end_inclusive.line[..=end_inclusive.col];
                output.push(String::from(last));
            }
        }
    }
}

/// Check the token's inner value matches its lexeme.
#[allow(unused)]
fn sanity_check<'a>(token: &TokenInfo<'a>) {
    assert_eq!(token.lexeme, token_to_str(&token.val), "{:?}", token);
}

/// Convert a token's inner value to a reasonable string representation.
#[allow(unused)]
fn token_to_str<'a>(token: &Token<'a>) -> String {
    todo!()
}

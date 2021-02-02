use std::iter::Peekable;

/// Resolve the Java escape sequences in a string.
pub fn resolve_escape_seqs(literal: &str) -> String {
    let mut input = literal.chars().peekable();
    let mut buf = String::with_capacity(literal.len());

    while let Some(c) = input.next() {
        if c != '\\' {
            buf.push(c);
        } else {
            let resolved = resolve_once(&mut input);
            buf.push(resolved);
        }
    }

    buf
}

/// Resolve a single escape sequence.
///
/// NOTE: you must have already consumed the `\` from `input` before you call this.
///
/// This consumes up to and including the end of the escape sequence, if successful. No guarantees
/// about what is consumed on failure.
fn resolve_once(input: &mut Peekable<impl Iterator<Item = char>>) -> char {
    match input.next() {
        // Technically this case should never happen, because the Java lang NFA only matches string
        // literals that *don't* have a backslash before the closing quote.
        None => panic!("Empty escape sequence: backslash at end of string literal"), // todo

        Some(next_char) => match next_char {
            'b' => '\x08', // '\b': backspace
            't' => '\t',
            'n' => '\n',
            'f' => '\x0c', // '\f': form feed
            'r' => '\r',
            '"' => '"',
            '\'' => '\'',
            '\\' => '\\',
            '0'..='7' => {
                // Consume up to a total of 4 digits.
                let mut digits = Vec::with_capacity(4);
                digits.push(next_char);
                while digits.len() < 4 {
                    match input.peek() {
                        Some('0'..='7') => {
                            digits.push(input.next().unwrap());
                        }
                        _ => break,
                    }
                }
                debug_assert!(1 <= digits.len() && digits.len() <= 4);

                if digits.len() == 4 && digits[0] >= '4' {
                    panic!(
                        "Octal literal too large (largest is \\3777), found \\{}{}{}{}",
                        digits[0], digits[1], digits[2], digits[3]
                    ); // todo
                }

                octal_to_utf8(&digits)
            }
            '8' | '9' => panic!(
                "Numeric escape sequences are octal (largest digit is 7), found \\{}", // todo
                next_char
            ),
            _ => panic!("Invalid or unknown escape sequence: \\{}", next_char), // todo
        },
    }
}

/// Turn a sequence of octal digits into a unicode scalar value.
fn octal_to_utf8(digits: &[char]) -> char {
    let mut ascii_val = 0u32;
    for &c in digits {
        debug_assert!('0' <= c && c <= '7');
        let digit = c as u32 - '0' as u32;
        ascii_val <<= 3;
        ascii_val += digit;
    }
    debug_assert!(ascii_val <= 0o3777);

    assert!(ascii_val < 128); // todo error handling
    ascii_val as u8 as char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let s = resolve_escape_seqs(r"asdf\\asdf\'");
        assert_eq!(s, r"asdf\asdf'");
    }
}

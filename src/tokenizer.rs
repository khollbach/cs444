use dfa::DFA;
use states::{State, StateSet};
use std::fmt;
use token_types::TokenType;

mod dfa;
mod java_lang_nfa;
mod nfa;
mod nfa_to_dfa;
mod states;
mod token_types;

/// A token in the output stream of the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub type_: TokenType<'a>,
    pub start: Position<'a>,
    pub lexeme: &'a str,
}

impl<'a> Token<'a> {
    pub fn end_col(self) -> usize {
        // Relies on the token being ASCII-only.
        self.start.col + self.lexeme.len()
    }
}

/// Tokenizer for the Joos 1W language.
#[derive(Debug)]
pub struct Tokenizer {
    dfa: DFA<StateSet<State>>,
}

impl Tokenizer {
    /// Compile an NFA for Java's lexical grammar into a DFA.
    pub fn new() -> Self {
        let nfa = java_lang_nfa::java_lang_nfa();
        let dfa = nfa.to_dfa();

        Self { dfa }
    }

    /// Run the "max munch" scanning algorithm to tokenize the input.
    pub fn tokenize<'a>(
        &'a self,
        lines: impl Iterator<Item = &'a str> + Clone + 'a,
    ) -> impl Iterator<Item = Token> + 'a {
        self.dfa.tokenize(all_positions(lines))
    }
}

/// A position in the input stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position<'a> {
    line_num: usize,
    line: &'a str,
    /// Invariant: 0 <= col <= line.len()
    col: usize,
}

impl<'a> Position<'a> {
    /// Get the current symbol at this position.
    ///
    /// As a special case, if the current position is 1 past the end of the current line, then we
    /// return a "newline" symbol. This makes up for `line` not containing a newline character.
    fn symbol(&self) -> Symbol {
        assert!(self.col <= self.line.len());
        if self.col == self.line.len() {
            return Symbol::new(b'\n');
        }

        let b = self.line.as_bytes()[self.col];

        // todo gracefully handle encoding errors!!
        assert!(b < 128, "Not ASCII: 0x{:x}", b);

        Symbol::new(b)
    }
}

/// Turn an iterator of lines into a flattened iterator of positions.
///
/// Includes special "newline" positions after each line.
fn all_positions<'a>(
    lines: impl Iterator<Item = &'a str> + Clone + 'a,
) -> impl Iterator<Item = Position<'a>> + Clone + 'a {
    lines
        .enumerate()
        .flat_map(|(line_num, line)| line_positions(line_num, line))
}

/// Turn a single line into an iterator of all positions in that line.
///
/// Includes a special "newline" position after the end of the line.
fn line_positions<'a>(
    line_num: usize,
    line: &'a str,
) -> impl Iterator<Item = Position<'a>> + Clone + 'a {
    // Note the inclusive range. This is crucial since `line` itself has no newline.
    (0..=line.len()).map(move |col| Position {
        line_num,
        line,
        col,
    })
}

/// A symbol in the input stream.
///
/// Used to label state transitions in DFAs and NFAs.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol {
    ascii_byte: u8,
}

impl Symbol {
    fn new(ascii_byte: u8) -> Self {
        assert!(ascii_byte < 128);
        Self { ascii_byte }
    }

    fn to_char(self) -> char {
        self.ascii_byte as char
    }
}

impl fmt::Debug for Symbol {
    /// We could just derive, but this avoids newlines in {:#?} output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Symbol({:?})", self.to_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use token_types::Keyword::{Else, If, While};
    use token_types::Operator::{Assign, Le};
    use token_types::Separator::{Comma, Dot, LBrace};
    use token_types::TokenType::{Keyword, Operator, Separator};

    /// Run a few simple examples; each should be successfully tokenized.
    #[test]
    fn tokenize_simple_examples() {
        let tokenizer = Tokenizer::new();

        for (input, expected) in vec![
            (
                vec!["if while else"],
                vec![Keyword(If), Keyword(While), Keyword(Else)],
            ),
            (
                vec!["if while", "else"],
                vec![Keyword(If), Keyword(While), Keyword(Else)],
            ),
            (
                vec!["if{ ,.<=="],
                vec![
                    Keyword(If),
                    Separator(LBrace),
                    Separator(Comma),
                    Separator(Dot),
                    Operator(Le),
                    Operator(Assign),
                ],
            ),
        ] {
            let mut actual = vec![];
            for token in tokenizer.tokenize(input.into_iter()) {
                actual.push(token.type_);
            }
            assert_eq!(expected, actual);
        }
    }
}

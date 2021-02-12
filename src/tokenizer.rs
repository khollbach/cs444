//! This module contains the logic for breaking an input stream into tokens.
//!
//! The tokenizer can be used as follows:
//! ```
//! # use cs444::tokenizer::Tokenizer;
//!
//! let input = vec![
//!     "class A {",
//!     "  public static void run() {",
//!     "    1 + 1;",
//!     "  }",
//!     "}",
//! ];
//!
//! let tokenizer = Tokenizer::new();
//!
//! for token in tokenizer.tokenize(input.into_iter()) {
//!     // do something interesting ...
//!     dbg!(token);
//! }
//! ```

use dfa::DFA;
use states::{State, StateSet, Symbol};
use tokens::{TokenInfo, TokenOrComment};

mod dfa;
mod joos_1w_nfa;
mod nfa;
mod nfa_to_dfa;
mod states;
pub mod token_types;
pub mod tokens;

/// Tokenizer for the Joos 1W language.
#[derive(Debug)]
pub struct Tokenizer {
    dfa: DFA<StateSet<State>>,
}

impl Tokenizer {
    /// Compile an NFA for the lexical grammar of Joos 1W into a DFA.
    ///
    /// Be warned that this is an expensive operation. Best to avoid calling this in a loop (e.g.
    /// in test cases, etc.)
    pub fn new() -> Self {
        let nfa = joos_1w_nfa::nfa();
        let dfa = nfa.to_dfa();

        Self { dfa }
    }

    /// Tokenize the input, stripping out comments.
    pub fn tokenize<'a>(
        &'a self,
        lines: impl Iterator<Item = &'a str> + Clone + 'a,
    ) -> impl Iterator<Item = TokenInfo> + 'a {
        self.tokenize_keep_comments(lines)
            .filter_map(|elem| match elem {
                TokenOrComment::Token(t) => Some(t),
                TokenOrComment::LineComment { .. } => None,
                TokenOrComment::StarComment { .. } => None,
            })
    }

    /// Run the "max munch" scanning algorithm to tokenize the input.
    pub fn tokenize_keep_comments<'a>(
        &'a self,
        lines: impl Iterator<Item = &'a str> + Clone + 'a,
    ) -> impl Iterator<Item = TokenOrComment> + 'a {
        self.dfa.tokenize(all_positions(lines))
    }
}

/// A position in the input stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position<'a> {
    /// Does not contain a newline character.
    pub line: &'a str,
    /// Zero-indexed.
    pub line_num: usize,
    /// Invariant: 0 <= col <= line.len()
    pub col: usize,
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

#[cfg(test)]
mod tests {
    use super::*;
    use token_types::Keyword::{Else, If, While};
    use token_types::Operator::{Assign, Le};
    use token_types::Separator::{Comma, Dot, LBrace, RBrace};
    use tokens::Token::{self, Keyword, Operator, Separator};

    /// A test case for the tokenizer. Only the tokens' inner values are checked.
    pub struct TestCase<'a> {
        pub input: Vec<&'a str>,
        pub expected_output: Vec<Token<'a>>,
    }

    impl<'a> TestCase<'a> {
        /// Panics if the input doesn't tokenize as expected.
        pub fn run(self, tokenizer: &Tokenizer) {
            let mut actual = vec![];
            for token in tokenizer.tokenize(self.input.into_iter()) {
                actual.push(token.val);
            }
            assert_eq!(self.expected_output, actual);
        }
    }

    /// A test case for the tokenizer. The entire `TokenInfo` for each token is checked.
    ///
    /// This is more detailed than `TestCase`, but also more tedious to specify.
    struct DetailedTestCase<'a> {
        input: Vec<&'a str>,
        expected_output: Vec<TokenInfo<'a>>,
    }

    impl<'a> DetailedTestCase<'a> {
        /// Panics if the input doesn't tokenize as expected.
        fn run(self, tokenizer: &Tokenizer) {
            let actual: Vec<_> = tokenizer.tokenize(self.input.into_iter()).collect();
            assert_eq!(self.expected_output, actual);
        }
    }

    /// Run a few simple examples.
    #[test]
    fn simple_examples() {
        let tokenizer = Tokenizer::new();

        for (input, expected_output) in vec![
            (vec![""], vec![]),
            (
                vec!["if while else"],
                vec![Keyword(If), Keyword(While), Keyword(Else)],
            ),
            (
                vec![" \t if while", "", "  else", " ", "", "\t"],
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
            TestCase {
                input,
                expected_output,
            }
            .run(&tokenizer);
        }
    }

    /// Run a detailed example.
    #[test]
    fn detailed_example() {
        let tokenizer = Tokenizer::new();

        let input = vec!["if while else", "", "{}"];

        let if_ = TokenInfo {
            val: Keyword(If),
            start: Position {
                line: input[0],
                line_num: 0,
                col: 0,
            },
            lexeme: "if",
        };

        let while_ = TokenInfo {
            val: Keyword(While),
            start: Position {
                line: input[0],
                line_num: 0,
                col: 3,
            },
            lexeme: "while",
        };

        let else_ = TokenInfo {
            val: Keyword(Else),
            start: Position {
                line: input[0],
                line_num: 0,
                col: 9,
            },
            lexeme: "else",
        };

        let left = TokenInfo {
            val: Separator(LBrace),
            start: Position {
                line: input[2],
                line_num: 2,
                col: 0,
            },
            lexeme: "{",
        };

        let right = TokenInfo {
            val: Separator(RBrace),
            start: Position {
                line: input[2],
                line_num: 2,
                col: 1,
            },
            lexeme: "}",
        };

        let expected_output = vec![if_, while_, else_, left, right];

        DetailedTestCase {
            input,
            expected_output,
        }
        .run(&tokenizer);
    }
}

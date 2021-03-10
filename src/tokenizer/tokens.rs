use crate::tokenizer::token_types::{Keyword, Literal, Operator, Separator};
use crate::tokenizer::Position;
use std::error::Error;
use std::fmt;

/// Diffent types of tokens in the language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    Identifier(&'a str),
    Keyword(Keyword),
    Literal(Literal),
    Separator(Separator),
    Operator(Operator),
}

/// A token in the output stream of the tokenizer, together with some metadata about where it is in
/// the input stream.
///
/// The metadata helps us provide the user with better error messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenInfo<'a> {
    pub val: Token<'a>,
    pub start: Position<'a>,
    pub lexeme: &'a str,
}

impl TokenInfo<'_> {
    /// Zero-indexed, exclusive.
    pub fn end_col(&self) -> usize {
        // Relies on the token being single-line and ASCII-only.
        // (This is true of all tokens in our language, so we're good.)
        self.start.col + self.lexeme.len()
    }
}

/// The tokenizer also supports producing an output stream with comments included.
///
/// This is the element type of that alternative output stream.
#[derive(Debug, Clone)]
pub enum TokenOrComment<'a> {
    Token(TokenInfo<'a>),
    LineComment {
        start: Position<'a>,
    },
    StarComment {
        start: Position<'a>,
        /// Inclusive!
        end_inclusive: Position<'a>,
    },
}

impl<'a> TokenOrComment<'a> {
    pub fn start(&self) -> Position<'a> {
        use TokenOrComment::*;
        match self {
            Token(t) => t.start,
            LineComment { start } => *start,
            StarComment { start, .. } => *start,
        }
    }
}

/// An error encountered while tokenizing.
#[derive(Debug)]
pub struct TokenError<'a> {
    pub start: Position<'a>,
    pub type_: TokenErrorType<'a>,
}

/// Types of `TokenError`s.
#[derive(Debug)]
pub enum TokenErrorType<'a> {
    /// The input contained a non-ascii character. We currently don't support these.
    NonAsciiChar { c: char },
    /// Not a token, nor a prefix of a token.
    NotAToken {
        /// Exclusive!
        end: Position<'a>,
    },
    /// A star-comment that is never closed. (The input stream ended first.)
    UnclosedComment,
    /// An unclosed string literal. (The line ended first.)
    UnclosedStringLit,
    /// An unclosed char literal. (The line ended first.)
    UnclosedCharLit,
}

impl<'a> fmt::Display for TokenError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // todo implement half-decent error display
        write!(f, "{:?}", self)
    }
}

impl<'a> Error for TokenError<'a> {}

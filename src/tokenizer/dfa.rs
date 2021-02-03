use crate::tokenizer::states::AcceptedStateLabel;
use crate::tokenizer::tokens::Literal::{self, Char, Int, StringLit};
use crate::tokenizer::tokens::Token;
use crate::tokenizer::{Position, Symbol, TokenInfo};
use key_pair::KeyPair;
use std::collections::HashMap as Map;
use std::hash::Hash;
use std::iter;

mod key_pair;
mod string_escapes;

#[derive(Debug)]
pub struct DFA<S> {
    pub init: S,
    pub accepted: Map<S, AcceptedStateLabel>,
    pub delta: Map<(S, Symbol), S>,
}

impl<S: Eq + Hash> DFA<S> {
    /// Tokenize the input stream, stripping out comments and whitespace.
    // todo: report errors.
    pub fn tokenize<'a>(
        &'a self,
        positions: impl Iterator<Item = Position<'a>> + Clone + 'a,
    ) -> impl Iterator<Item = TokenInfo<Token>> + 'a {
        self.main_loop(positions).flat_map(|longest_match| {
            match longest_match {
                LongestMatch::Token(token) => Some(token),
                // Silently ignore comments and whitespace.
                LongestMatch::CommentOrWhitespace => None,
            }
        })
    }

    /// Run "max munch" in a loop.
    fn main_loop<'a>(
        &'a self,
        positions: impl Iterator<Item = Position<'a>> + Clone + 'a,
    ) -> impl Iterator<Item = LongestMatch> + 'a {
        let mut positions = positions.peekable();

        iter::from_fn(move || {
            match positions.peek().copied() {
                // The stream dried up; terminate.
                None => None,
                Some(pos) => {
                    let ret = self.max_munch(pos, &mut positions);

                    // todo: handle errors more gracefully.
                    let longest_match =
                        ret.unwrap_or_else(|| panic!("Failed to tokenize at {:?}", pos));

                    Some(longest_match)
                }
            }
        })
    }

    /// Return a token corresponding to the longest matching prefix of the stream.
    ///
    /// Consumes up to and including the last symbol of that token.
    fn max_munch<'a>(
        &'a self,
        start: Position<'a>,
        positions: &mut (impl Iterator<Item = Position<'a>> + Clone),
    ) -> Option<LongestMatch<'a>> {
        // Note that we'll never check if the empty string matches, since the code below
        // transitions states before it checks acceptance.
        //
        // (We don't want empty tokens anyways, and handling them would add complexity.)
        debug_assert!(!self.accepted.contains_key(&self.init));

        let mut longest_match = None;
        let mut unused_symbols = positions.clone();

        let mut state = &self.init;

        while let Some(pos) = positions.next() {
            let key = (state, &pos.symbol()); // todo handle encoding errors (somewhere)
            state = match self.delta.get(&key as &dyn KeyPair<_, _>) {
                Some(next) => next,
                // Implicit "dead" state, stop scanning.
                None => break,
            };

            if let Some(label) = self.accepted.get(state) {
                // Keep track of the longest match, and the positions after it.
                longest_match = Some(match label {
                    AcceptedStateLabel::Token(val) => {
                        LongestMatch::Token(make_token(val, start, pos))
                    }
                    AcceptedStateLabel::CommentOrWhitespace => LongestMatch::CommentOrWhitespace,
                });
                unused_symbols = positions.clone();
            }
        }

        // Reset `positions` to reflect which symbols were actually consumed by the longest match.
        *positions = unused_symbols;

        longest_match
    }
}

enum LongestMatch<'a> {
    Token(TokenInfo<'a, Token<'a>>),
    CommentOrWhitespace,
}

/// Create TokenInfo from a token type.
///
/// Note that `start` and `end` are both inclusive!!!
fn make_token<'a>(
    val: &Token<'static>,
    start: Position<'a>,
    end: Position<'a>,
) -> TokenInfo<'a, Token<'a>> {
    // Note the inclusive range.
    let lexeme = &start.line[start.col..=end.col];

    // Fill in the guts of the token, if applicable.
    let val = match val {
        Token::Identifier(_) => Token::Identifier(lexeme),
        Token::Literal(lit) => Token::Literal(match lit {
            Int(_) => {
                // Note that in Joos 1W, all int literals are `int` type, since there is no
                // `unsigned` in Java, and no `long` in Joos 1W.
                let n: Result<u32, _> = lexeme.parse();

                // todo handle errors gracefully
                let n = n.expect("Can't parse int; probably too big.");
                assert!(n <= 2u32.pow(31));

                Int(n)
            }
            Char(_) => make_char_literal(lexeme),
            StringLit(_) => {
                // Strip quotes.
                debug_assert_eq!(&lexeme[..1], "\"");
                debug_assert_eq!(&lexeme[lexeme.len() - 1..], "\"");
                let unescaped = &lexeme[1..lexeme.len() - 1];

                StringLit(string_escapes::resolve_escape_seqs(unescaped))
            }
            l => l.clone(),
        }),
        t => t.clone(),
    };

    TokenInfo { val, start, lexeme }
}

fn make_char_literal(lexeme: &str) -> Literal {
    // Strip quotes.
    debug_assert_eq!(&lexeme[..1], "'");
    debug_assert_eq!(&lexeme[lexeme.len() - 1..], "'");
    let unescaped = &lexeme[1..lexeme.len() - 1];

    // We could make the error messages better here by making `string_escapes` resolution lazy.
    // That way as soon as there's more than one char we just stop, instead of trying to resolve
    // escapes later in the line (whose errors would probably just confuse the user).
    let s = string_escapes::resolve_escape_seqs(unescaped);
    let mut chars = s.chars();
    let ch = match chars.next() {
        None => panic!("Empty char literal; must have a char between the quotes"),
        Some(ch) => ch,
    };
    match chars.next() {
        None => (),
        Some(extra) => {
            panic!(
                "Char literal too long. Expected closing quote `'` but found `{}`",
                extra
            )
        }
    };

    Char(ch)
}

#[cfg(test)]
impl<S: Eq + Hash> DFA<S> {
    /// Test helper to assert that the dfa accepts and/or rejects the given ascii strings.
    ///
    /// This helps us test NFA to DFA conversion, among other things.
    pub fn _check(&self, accepted: &[&str], rejected: &[&str]) {
        fn symbols<'a>(s: &'a str) -> impl Iterator<Item = Symbol> + 'a {
            s.chars().map(|c| Symbol::new(c as u8))
        }

        for s in accepted {
            assert!(self._accepts(symbols(s)), "{}", s);
        }

        for s in rejected {
            assert!(!self._accepts(symbols(s)), "{}", s);
        }
    }

    /// Does this DFA accept this string of symbols?
    fn _accepts(&self, symbols: impl Iterator<Item = Symbol>) -> bool {
        let mut state = &self.init;
        for sym in symbols {
            let key = (state, &sym);
            state = match self.delta.get(&key as &dyn KeyPair<_, _>) {
                Some(next) => next,
                // Implicit "dead" state.
                None => return false,
            };
        }
        self.accepted.contains_key(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer;
    use crate::tokenizer::tokens::Keyword::If;
    use crate::tokenizer::tokens::Token::Keyword;

    /// Helper struct for specifying small DFAs in unit tests.
    struct DFABuilder<'a> {
        init: &'a str,
        accepted: Vec<(&'a str, AcceptedStateLabel)>,
        delta: Vec<((&'a str, char), &'a str)>,
    }

    impl<'a> DFABuilder<'a> {
        fn build(self) -> DFA<&'a str> {
            let init = self.init;
            let accepted = self.accepted.into_iter().collect();
            let delta = self
                .delta
                .into_iter()
                .map(|((s, ch), t)| ((s, Symbol::new(ch as u8)), t))
                .collect();

            DFA {
                init,
                accepted,
                delta,
            }
        }
    }

    /// This DFA recognizes the language {"a", "ba"}.
    fn simple_dfa() -> DFA<&'static str> {
        DFABuilder {
            init: "init",
            accepted: vec![("accept", AcceptedStateLabel::Token(Keyword(If)))],
            delta: vec![
                (("init", 'a'), "accept"),
                (("init", 'b'), "b"),
                (("b", 'a'), "accept"),
            ],
        }
        .build()
    }

    /// Check that a simple DFA matches the expected strings.
    #[test]
    fn simple_dfa_accepts() {
        let dfa = simple_dfa();

        dbg!(&dfa);

        let accepted = vec!["a", "ba"];
        let rejected = vec!["", "b", "aa", "ab", "bb", "bba", "aaaaaba"];
        dfa._check(&accepted, &rejected);
    }

    /// Run the DFA on one line of ASCII text, to tokenize it.
    fn tokenize_one_line<'a>(
        dfa: &'a DFA<&'a str>,
        line: &'a str,
    ) -> Vec<TokenInfo<'a, Token<'a>>> {
        let positions = tokenizer::all_positions(iter::once(line));

        // Skip the special "newline" position at the end of `all_positions`.
        let positions = positions.take(line.len());

        dfa.tokenize(positions).collect()
    }

    /// Tokenize a short string of a's and b's.
    #[test]
    #[allow(non_snake_case)]
    fn tokenize_As_and_Bs() {
        let dfa = simple_dfa();

        let input = "abaaababa";
        let expected = vec!["a", "ba", "a", "a", "ba", "ba"];

        let mut actual = vec![];
        for token in tokenize_one_line(&dfa, input) {
            actual.push(token.lexeme);
        }
        assert_eq!(expected, actual);
    }

    /// Fail to tokenize a short string of a's and b's.
    #[test]
    #[should_panic]
    fn simple_tokenize_failure() {
        let dfa = simple_dfa();

        let input = "abaaababab";

        // todo this test will fail once we implement robust error handling; fix it then.
        tokenize_one_line(&dfa, input);
    }
}

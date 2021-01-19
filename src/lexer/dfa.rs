use crate::lexer::types::Symbol;
use crate::types::{Token, TokenType};
use std::collections::HashMap as Map;
use std::hash::Hash;

#[derive(Debug)]
pub struct DFA<S> {
    pub init: S,
    pub accepted: Map<S, TokenType>,
    pub delta: Map<(S, Symbol), S>,
}

impl<S> DFA<S>
where
    // todo: S shouldn't really need to be clone.
    S: Clone + Eq + Hash,
{
    /// Does this DFA accept this string of symbols?
    pub fn accepts(&self, symbols: impl Iterator<Item = Symbol>) -> bool {
        let mut state = &self.init;
        for sym in symbols {
            // There doesn't really have to be a clone here; see e.g.:
            // https://stackoverflow.com/questions/45786717/how-to-implement-hashmap-with-two-keys
            state = match self.delta.get(&(state.clone(), sym)) {
                Some(next) => next,
                // Implicit "dead" state.
                None => return false,
            };
        }
        self.accepted.contains_key(state)
    }

    /// Runs "max munch" in a loop.
    // todo: report errors.
    // todo: make lazy?
    pub fn tokenize<'a>(&self, file: &'a str) -> Vec<Token<'a>> {
        let mut res = vec![];

        let mut suffix = file;
        while !suffix.is_empty() {
            match self.max_munch(suffix) {
                Some(token) => {
                    // Advance the pointer into the file, consuming the token.
                    suffix = &suffix[token.lexeme.len()..];

                    res.push(token);
                }
                // todo: improve error message; include the failed prefix that
                // was checked in max_munch.
                None => panic!("Failed to lex {}", suffix),
            }
        }

        res
    }

    /// Returns the longest prefix of `file` that matches.
    fn max_munch<'a>(&self, file: &'a str) -> Option<Token<'a>> {
        // Note that we never check if the empty string matches,
        // since the code below always transitions before checking.
        //
        // We don't want empty tokens anyways, and handling them
        // would add complexity.
        debug_assert!(!self.accepted.contains_key(&self.init));

        let mut token = None;
        let mut state = &self.init;

        for (i, sym) in file.chars().map(Symbol).enumerate() {
            // todo: properly handle non-ascii error case!
            assert!(sym.0 < 128 as char, "Non-ascii character: {}", sym.0);

            // Don't really need this clone; see comment in `accepts()`.
            state = match self.delta.get(&(state.clone(), sym)) {
                Some(next) => next,
                // Implicit "dead" state, stop scanning.
                None => break,
            };

            if let Some(&typ) = self.accepted.get(state) {
                token = Some(Token {
                    typ,
                    // Note the off-by-one here.
                    lexeme: &file[..=i],
                });
            }
        }

        token
    }

    /// Test helper to assert that the dfa accepts and/or rejects the given strings.
    #[cfg(test)]
    pub fn _check(&self, accepted: &[&str], rejected: &[&str]) {
        for s in accepted {
            assert!(self.accepts(s.chars().map(Symbol)), "{}", s);
        }

        for s in rejected {
            assert!(!self.accepts(s.chars().map(Symbol)), "{}", s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TokenType::If;

    /// Helper struct for specifying small DFAs in unit tests.
    struct DFABuilder<'a> {
        init: &'a str,
        accepted: Vec<(&'a str, TokenType)>,
        delta: Vec<((&'a str, char), &'a str)>,
    }

    impl<'a> DFABuilder<'a> {
        fn build(self) -> DFA<&'a str> {
            let init = self.init;
            let accepted = self.accepted.into_iter().collect();
            let delta = self
                .delta
                .into_iter()
                .map(|((s, ch), t)| ((s, Symbol(ch)), t))
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
            accepted: vec![("accept", If)],
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

    /// Tokenize a short string of a's and b's.
    #[test]
    fn simple_tokenize() {
        let dfa = simple_dfa();

        let a = Token {
            typ: If,
            lexeme: "a",
        };
        let ba = Token {
            typ: If,
            lexeme: "ba",
        };

        let input = "abaaababa";
        let expected = Some(a);
        let actual = dfa.max_munch(input);
        assert_eq!(expected, actual);

        let input = "baaababa";
        let expected = Some(ba);
        let actual = dfa.max_munch(input);
        assert_eq!(expected, actual);

        let input = "abaaababa";
        let expected = vec![a, ba, a, a, ba, ba];
        let actual = dfa.tokenize(input);
        assert_eq!(expected, actual);
    }

    /// Fail to tokenize a short string of a's and b's.
    #[test]
    #[should_panic]
    fn simple_tokenize_failure() {
        let dfa = simple_dfa();

        let input = "abaaababab";
        dfa.tokenize(input);
    }
}

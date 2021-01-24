use crate::lexer::dfa::DFA;
use crate::lexer::nfa_to_dfa::NfaConverter;
use crate::lexer::types::{StateSet, Symbol};
use crate::types::TokenType;
use std::collections::HashMap as Map;
use std::hash::Hash;

/// We don't provide any methods to run the NFA; you must convert it to a DFA first via `to_dfa`.
#[derive(Debug)]
pub struct NFA<S> {
    pub init: S,
    pub accepted: Map<S, TokenType>,
    pub delta: Map<(S, Symbol), Vec<S>>,
    pub epsilon: Map<S, Vec<S>>,
}

impl<S> NFA<S>
where
    S: Copy + Ord + Hash,
{
    /// Convert this NFA into an equivalent DFA (they accept the same strings).
    pub fn to_dfa(&self) -> DFA<StateSet<S>> {
        NfaConverter::new(self).to_dfa()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Keyword::If;
    use crate::types::TokenType::Keyword;

    /// Helper struct for specifying small NFAs in unit tests.
    struct NFABuilder<'a> {
        init: &'a str,
        accepted: Vec<(&'a str, TokenType)>,
        delta: Vec<((&'a str, char), Vec<&'a str>)>,
        epsilon: Vec<(&'a str, Vec<&'a str>)>,
    }

    impl<'a> NFABuilder<'a> {
        fn build(self) -> NFA<&'a str> {
            let init = self.init;
            let accepted = self.accepted.into_iter().collect();
            let delta = self
                .delta
                .into_iter()
                .map(|((s, ch), t)| ((s, Symbol::new(ch)), t))
                .collect();
            let epsilon = self.epsilon.into_iter().collect();

            NFA {
                init,
                accepted,
                delta,
                epsilon,
            }
        }
    }

    /// This NFA recognizes the language {"a", "ab", "aba"}.
    fn simple_nfa() -> NFA<&'static str> {
        NFABuilder {
            init: "init",
            accepted: vec![("a1", Keyword(If)), ("aba", Keyword(If))],
            delta: vec![
                (("init", 'a'), vec!["a1", "a2"]),
                (("a2", 'b'), vec!["ab"]),
                (("ab", 'a'), vec!["aba"]),
            ],
            epsilon: vec![("ab", vec!["a1"])],
        }
        .build()
    }

    /// Convert a simple NFA to a DFA and check that it matches the expected strings.
    #[test]
    fn nfa_to_dfa() {
        let nfa = simple_nfa();
        let dfa = nfa.to_dfa();

        // It's also worth inspecting this to see that it looks right.
        dbg!(&nfa, &dfa);

        let accepted = vec!["a", "ab", "aba"];
        let rejected = vec!["", "b", "aa", "ba", "bb", "bba", "aaaaaba"];
        dfa._check(&accepted, &rejected);
    }
}

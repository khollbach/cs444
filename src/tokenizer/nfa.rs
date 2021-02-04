use crate::tokenizer::dfa::DFA;
use crate::tokenizer::nfa_to_dfa::NfaConverter;
use crate::tokenizer::states::{AcceptedStateLabel, StateSet, Symbol};
use std::collections::HashMap as Map;
use std::hash::Hash;

/// We don't provide any methods to run the NFA; you must convert it to a DFA first via `to_dfa`.
#[derive(Debug)]
pub struct NFA<S> {
    pub init: S,
    pub accepted: Map<S, AcceptedStateLabel>,
    pub delta: Map<(S, Symbol), Vec<S>>,
    pub epsilon: Map<S, Vec<S>>,
}

impl<S: Copy + Ord + Hash> NFA<S> {
    /// Convert this NFA into an equivalent DFA (they accept the same strings).
    pub fn to_dfa(&self) -> DFA<StateSet<S>> {
        NfaConverter::new(self).to_dfa()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokens::Keyword::If;
    use crate::tokenizer::tokens::Token::Keyword;

    /// Helper struct for specifying small NFAs in unit tests.
    struct NFABuilder<'a> {
        init: &'a str,
        accepted: Vec<(&'a str, AcceptedStateLabel)>,
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
                .map(|((s, ch), t)| ((s, Symbol::new(ch as u8)), t))
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
        let if_ = AcceptedStateLabel::TokenType { type_: Keyword(If) };

        NFABuilder {
            init: "init",
            accepted: vec![("a1", if_.clone()), ("aba", if_)],
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

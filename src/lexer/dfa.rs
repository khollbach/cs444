use crate::lexer::types::Symbol;
use std::collections::{HashMap as Map, HashSet as Set};
use std::hash::Hash;

#[derive(Debug)]
pub struct DFA<S> {
    pub init: S,
    pub accepted: Set<S>,
    pub delta: Map<(S, Symbol), S>,
}

impl<S> DFA<S>
where
    S: Eq + Hash + Clone,
{
    /// Does this DFA accept this string of symbols?
    fn accepts(&self, symbols: impl Iterator<Item = Symbol>) -> bool {
        let mut state = &self.init;
        for s in symbols {
            state = match self.delta.get(&(state.clone(), s)) {
                Some(next) => next,
                // Implicit "dead" state.
                None => return false,
            };
        }
        self.accepted.contains(state)
    }

    /// Test helper to assert that the dfa accepts and/or rejects the given strings.
    pub fn _check(&self, accepted: &[&str], rejected: &[&str]) {
        for s in accepted {
            assert!(self.accepts(s.chars().map(Symbol)));
        }

        for s in rejected {
            assert!(!self.accepts(s.chars().map(Symbol)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::types::State;

    /// This DFA recognizes the language {"a", "ba"}.
    fn dfa_a_ba() -> DFA<State> {
        let init = State(0);
        let b = State(1);
        let accept = State(2);

        let accepted = vec![accept].into_iter().collect();

        let delta = vec![
            ((init, Symbol('b')), b),
            ((b, Symbol('a')), accept),
            ((init, Symbol('a')), accept),
        ]
        .into_iter()
        .collect();

        DFA {
            init,
            accepted,
            delta,
        }
    }

    /// Check that a simple DFA matches the expected strings.
    #[test]
    fn simple_dfa() {
        let dfa = dfa_a_ba();

        dbg!(&dfa);

        let accepted = vec!["a", "ba"];
        let rejected = vec!["", "b", "aa", "ab", "bb", "bba", "aaaaaba"];
        dfa._check(&accepted, &rejected);
    }
}

use crate::lexer::dfa::DFA;
use crate::lexer::types::{State, StateSet, Symbol};
use converter::NfaConverter;
use std::collections::{HashMap as Map, HashSet as Set};

mod converter;

/// We don't provide any methods to run the NFA; you must convert it to a DFA first via `to_dfa`.
#[derive(Debug)]
pub struct NFA {
    init: State,
    accepted: Set<State>,
    delta: Map<(State, Symbol), Vec<State>>,
    epsilon: Map<State, Vec<State>>,
}

impl NFA {
    /// Convert this NFA into an equivalent DFA (they accept the same strings).
    fn to_dfa(&self) -> DFA<StateSet> {
        NfaConverter::new(self).to_dfa()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This NFA recognizes the language {"a", "ab", "aba"}.
    fn nfa_a_ab_aba() -> NFA {
        let init = State(0);
        let a1 = State(1);
        let a2 = State(2);
        let b = State(3);
        let a3 = State(4);

        let accepted = vec![a2, a3].into_iter().collect();

        let delta = vec![
            ((init, Symbol('a')), vec![a1, a2]),
            ((a1, Symbol('b')), vec![b]),
            ((b, Symbol('a')), vec![a3]),
        ]
        .into_iter()
        .collect();

        let epsilon = vec![(b, vec![a2])].into_iter().collect();

        NFA {
            init,
            accepted,
            delta,
            epsilon,
        }
    }

    /// Convert a simple NFA to a DFA and check that it matches the expected strings.
    #[test]
    fn nfa_to_dfa() {
        let nfa = nfa_a_ab_aba();
        let dfa = nfa.to_dfa();

        dbg!(&nfa, &dfa);

        let accepted = vec!["a", "ab", "aba"];
        let rejected = vec!["", "b", "aa", "ba", "bb", "bba", "aaaaaba"];
        dfa._check(&accepted, &rejected);
    }
}

use std::collections::{HashMap as Map, HashSet as Set};
use std::fmt;

#[derive(Debug)]
struct DFA {
    init: State,
    accepted: Set<State>,
    delta: Map<(State, Symbol), State>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct State(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Symbol(char);

impl DFA {
    fn accepts(&self, symbols: impl Iterator<Item = Symbol>) -> bool {
        let mut state = self.init;
        for s in symbols {
            state = match self.delta.get(&(state, s)) {
                Some(&new) => new,
                // Implicit "dead" state.
                None => return false,
            };
        }
        self.accepted.contains(&state)
    }
}

impl fmt::Debug for State {
    /// We could just derive, but this avoids newlines in {:#?} output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State({:?})", self.0)
    }
}

impl fmt::Debug for Symbol {
    /// We could just derive, but this avoids newlines in {:#?} output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Symbol({:?})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_dfa() {
        let init = State(0);
        let b = State(1);
        let accept = State(2);
        let mut accepted = Set::new();
        accepted.insert(accept);
        let mut delta = Map::new();
        delta.insert((init, Symbol('b')), b);
        delta.insert((b, Symbol('a')), accept);
        delta.insert((init, Symbol('a')), accept);

        // This DFA recognizes the language {"a", "ba"}.
        let dfa = DFA {
            init,
            accepted,
            delta,
        };

        let accepted = vec!["a", "ba"];
        let rejected = vec!["", "b", "aa", "ab", "bb", "bba", "aaaaaba"];
        for s in accepted {
            assert!(dfa.accepts(s.chars().map(Symbol)));
        }
        for s in rejected {
            assert!(!dfa.accepts(s.chars().map(Symbol)));
        }
    }
}

use crate::tokenizer::dfa::DFA;
use crate::tokenizer::nfa::NFA;
use crate::tokenizer::states::{AcceptedStateLabel, StateSet, Symbol};
use std::collections::{BTreeSet, HashMap as Map, HashSet as Set};
use std::hash::Hash;

/// Helper struct to convert an NFA to an equivalent DFA.
///
/// The generic type `S` will usually be the type `State`, except in unit tests.
pub struct NfaConverter<'a, S> {
    nfa: &'a NFA<S>,

    /// An alternative representation of the keys in `nfa.delta`.
    /// This lets us efficiently find the active symbols of a given state of the nfa.
    active_symbols: Map<S, Vec<Symbol>>,
}

impl<'a, S: Copy + Ord + Hash> NfaConverter<'a, S> {
    pub fn new(nfa: &'a NFA<S>) -> Self {
        let mut tmp = Self {
            nfa,
            active_symbols: Map::new(),
        };
        tmp.compute_active_symbols();
        tmp
    }

    /// See the `active_symbols` field for details.
    fn compute_active_symbols(&mut self) {
        assert!(self.active_symbols.is_empty());
        for &(s, sym) in self.nfa.delta.keys() {
            self.active_symbols.entry(s).or_default().push(sym);
        }
    }

    /// Perform the conversion. (Sometimes known as the "powerset" construction.)
    ///
    /// The "states" of the resulting DFA have type `StateSet<S>`.
    pub fn to_dfa(self) -> DFA<StateSet<S>> {
        let init = self.eps_closure_one_state(self.nfa.init);
        let mut dfa = DFA {
            init: init.copy(),
            accepted: Map::new(),
            delta: Map::new(),
        };

        // Run a DFS starting from `init`.
        let mut seen = Set::new();
        seen.insert(init.copy());
        let mut queue = vec![init];

        // `ss` is the "current" StateSet.
        while let Some(ss) = queue.pop() {
            // Should `ss` be accepted by the DFA?
            if let Some(label) = self.is_accepted(&ss) {
                dfa.accepted.insert(ss.copy(), label);
            }

            // For each relevant symbol, enqueue a new StateSet.
            for sym in self.active_symbols(&ss) {
                let new_ss = self.next_stateset(&ss, sym);

                // Add an edge from `ss` to `new_ss`.
                dfa.delta.insert((ss.copy(), sym), new_ss.copy());

                if !seen.contains(&new_ss) {
                    seen.insert(new_ss.copy());
                    queue.push(new_ss);
                }
            }
        }

        dfa
    }

    /// Is `ss` is accepted by the DFA, and if so, which token type does it yield?
    ///
    /// This is determined by whether any of the "inner" states of the NFA are accepted.
    ///
    /// Ties for token type are broken by priority, via `S`'s `Ord` implementation. This means the
    /// most "important" tokens should have the smallest accepting states; e.g. keywords before
    /// identifiers, etc.
    fn is_accepted(&self, ss: &StateSet<S>) -> Option<AcceptedStateLabel> {
        // Since `StateSet`s are sorted, we'll find the smallest accepted state.
        for s in ss.states() {
            if let Some(label) = self.nfa.accepted.get(s) {
                return Some(label.clone());
            }
        }
        None
    }

    /// Find all symbols that have transitions out of this StateSet.
    fn active_symbols(&self, ss: &StateSet<S>) -> Set<Symbol> {
        let mut symbols = Set::new();
        for s in ss.states() {
            if let Some(syms) = self.active_symbols.get(s) {
                symbols.extend(syms.iter());
            }
        }
        symbols
    }

    /// Simultaneously transistion all states in `ss`, to produce a new StateSet.
    ///
    /// Takes the epsilon closure of the result before returning.
    fn next_stateset(&self, ss: &StateSet<S>, sym: Symbol) -> StateSet<S> {
        let mut reachable = BTreeSet::new();
        for &s in ss.states() {
            if let Some(nbrs) = self.nfa.delta.get(&(s, sym)) {
                reachable.extend(nbrs.iter().copied());
            }
        }

        self.eps_closure(&mut reachable);

        StateSet::new(reachable.into_iter())
    }

    /// Find the epsilon closure of a single state of the NFA.
    fn eps_closure_one_state(&self, state: S) -> StateSet<S> {
        let mut states = BTreeSet::new();
        states.insert(state);

        self.eps_closure(&mut states);

        StateSet::new(states.into_iter())
    }

    /// Find states reachable via epsilon transistions from `reachable`.
    ///
    /// `reachable` should typically start non-empty. It will be updated with the results.
    fn eps_closure(&self, reachable: &mut BTreeSet<S>) {
        let mut queue: Vec<_> = reachable.iter().copied().collect();

        while let Some(s) = queue.pop() {
            // Enqueue all reachable neighbours.
            if let Some(nbrs) = self.nfa.epsilon.get(&s) {
                for &nbr in nbrs {
                    if !reachable.contains(&nbr) {
                        reachable.insert(nbr);
                        queue.push(nbr);
                    }
                }
            }
        }
    }
}

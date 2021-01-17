use crate::lexer::dfa::DFA;
use crate::lexer::nfa::NFA;
use crate::lexer::types::{State, StateSet, Symbol};
use std::collections::{BTreeSet, HashMap as Map, HashSet as Set};

/// Helper struct to convert an NFA to an equivalent DFA.
pub struct NfaConverter<'a> {
    nfa: &'a NFA,

    /// Each (key, value) pair of this is a single key of `nfa.delta`.
    /// This lets us efficiently find the active symbols of a given State.
    active_symbols: Map<State, Vec<Symbol>>,
}

impl<'a> NfaConverter<'a> {
    pub fn new(nfa: &'a NFA) -> Self {
        let mut tmp = Self {
            nfa,
            active_symbols: Map::new(),
        };
        tmp.compute_active_symbols();
        tmp
    }

    /// Perform the conversion.
    ///
    /// The "states" of the resulting DFA have type StateSet.
    pub fn to_dfa(self) -> DFA<StateSet> {
        let init = self.eps_closure_one_state(self.nfa.init);
        let mut dfa = DFA {
            init: init.copy(),
            accepted: Set::new(),
            delta: Map::new(),
        };

        // Run a DFS starting from `init`.
        let mut seen = Set::new();
        seen.insert(init.copy());
        let mut q = vec![init];

        // `ss` is the "current" StateSet.
        while let Some(ss) = q.pop() {
            // Should `ss` be accepted by the DFA?
            if ss.states().iter().any(|s| self.nfa.accepted.contains(s)) {
                dfa.accepted.insert(ss.copy());
            }

            // For each relevant symbol, enqueue a new StateSet.
            for sym in self.active_symbols(&ss) {
                let new_ss = self.next_stateset(&ss, sym);

                // Add an edge from `ss` to `new_ss`.
                dfa.delta.insert((ss.copy(), sym), new_ss.copy());

                seen.insert(new_ss.copy());
                q.push(new_ss);
            }
        }

        dfa
    }

    /// See the `active_symbols` field for details.
    fn compute_active_symbols(&mut self) {
        assert!(self.active_symbols.is_empty());
        for &(s, sym) in self.nfa.delta.keys() {
            self.active_symbols.entry(s).or_default().push(sym);
        }
    }

    /// Find all symbols that have transitions out of this StateSet.
    fn active_symbols(&self, ss: &StateSet) -> Set<Symbol> {
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
    fn next_stateset(&self, ss: &StateSet, sym: Symbol) -> StateSet {
        let mut reachable = BTreeSet::new();
        for &s in ss.states() {
            if let Some(nbrs) = self.nfa.delta.get(&(s, sym)) {
                reachable.extend(nbrs.iter().copied());
            }
        }

        self.eps_closure(&mut reachable);

        StateSet::new(reachable.into_iter())
    }

    /// Find the epsilon closure of a single State.
    fn eps_closure_one_state(&self, state: State) -> StateSet {
        let mut states = BTreeSet::new();
        states.insert(state);

        self.eps_closure(&mut states);

        StateSet::new(states.into_iter())
    }

    /// Find states reachable via epsilon transistions from `reachable`.
    ///
    /// `reachable` should typically start non-empty. It will be updated with the results.
    fn eps_closure(&self, reachable: &mut BTreeSet<State>) {
        let mut q: Vec<_> = reachable.iter().copied().collect();

        while let Some(s) = q.pop() {
            // Enqueue all reachable neighbours.
            if let Some(nbrs) = self.nfa.epsilon.get(&s) {
                for &nbr in nbrs {
                    if !reachable.contains(&nbr) {
                        reachable.insert(nbr);
                        q.push(nbr);
                    }
                }
            }
        }
    }
}

use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;

/// A symbol in the input stream to a DFA or NFA.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub char);

/// A state of a DFA or NFA.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State(pub u32);

/// A sorted, unique list of states.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StateSet {
    states: Rc<[State]>,
}

impl StateSet {
    /// `states` must be non-empty and strictly increasing.
    pub fn new(states: impl Iterator<Item = State>) -> Self {
        let ss = Self {
            states: Rc::from_iter(states),
        };

        // In our code, we never need to work with empty StateSets.
        debug_assert!(!ss.states.is_empty());
        debug_assert!(ss._is_sorted_unique());

        ss
    }

    /// Get the inner states.
    pub fn states(&self) -> &[State] {
        &self.states
    }

    /// Just an alias for `clone`, because `Rc` clones are cheap.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Check if the states are strictly increasing.
    #[must_use]
    fn _is_sorted_unique(&self) -> bool {
        (1..self.states.len()).all(|i| self.states[i - 1] < self.states[i])
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

impl fmt::Debug for StateSet {
    /// We could just derive, but this avoids newlines in {:#?} output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StateSet({:?})", self.states)
    }
}

use crate::tokenizer::tokens::Token;
use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;

/// Used in NFAs and DFAs to label each accepted state with the token type it accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptedStateLabel {
    /// Beware that the inner data of the `Token` is likely just filler. E.g., it will never
    /// contain an actual identifier from the input stream. So you'll have to fill it in during
    /// tokenizing.
    TokenType {
        type_: Token<'static>,
    },
    LineComment,
    StarComment,
    JavadocComment,
    Whitespace,
}

/// A state of a DFA or NFA.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State(pub u32);

impl fmt::Debug for State {
    /// We could just derive, but this avoids newlines in {:#?} output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State({:?})", self.0)
    }
}

/// A sorted, unique list of states.
///
/// We use these as the states of a DFA generated from an NFA via the "powerset" construction.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StateSet<S> {
    states: Rc<[S]>,
}

impl<S> StateSet<S>
where
    S: Copy + Ord,
{
    /// `states` must be non-empty and strictly increasing.
    pub fn new(states: impl Iterator<Item = S>) -> Self {
        let ss = Self {
            states: Rc::from_iter(states),
        };

        // In our code, we never need to work with empty StateSets.
        debug_assert!(!ss.states.is_empty());
        debug_assert!(ss._is_sorted_unique());

        ss
    }

    /// Check if the states are sorted, strictly increasing.
    #[must_use]
    fn _is_sorted_unique(&self) -> bool {
        (1..self.states.len()).all(|i| self.states[i - 1] < self.states[i])
    }

    /// Get the inner states.
    pub fn states(&self) -> &[S] {
        &self.states
    }

    /// Just an alias for `clone`, because `S` is Copy and `Rc` clones are cheap.
    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl<S: fmt::Debug> fmt::Debug for StateSet<S> {
    /// This avoids newlines in {:#?} output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StateSet({:?})", self.states)
    }
}

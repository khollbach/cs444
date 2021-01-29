//! This allows us to lookup values in a HashMap whose keys are (A, B), even if we only have a hold
//! of (&A, &B). For details on how this works, see here:
//! https://stackoverflow.com/questions/45786717/how-to-implement-hashmap-with-two-keys
//!
//! Example use:
//! ```
//! use std::collections::HashMap;
//!
//! let mut map = HashMap::new();
//! map.insert((123, 456), 777);
//!
//! let key = (&123, &456);
//! assert_eq!(map.get(&key as &dyn KeyPair<_, _>), Some(&777));
//! ```

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

pub trait KeyPair<A, B> {
    fn a(&self) -> &A;
    fn b(&self) -> &B;
}

impl<A, B> KeyPair<A, B> for (A, B) {
    fn a(&self) -> &A {
        &self.0
    }

    fn b(&self) -> &B {
        &self.1
    }
}

impl<A, B> KeyPair<A, B> for (&A, &B) {
    fn a(&self) -> &A {
        self.0
    }

    fn b(&self) -> &B {
        self.1
    }
}

/// A pair (A, B) can be borrowed as a KeyPair<A, B> trait object.
impl<'a, A: 'a, B: 'a> Borrow<dyn KeyPair<A, B> + 'a> for (A, B) {
    fn borrow(&self) -> &(dyn KeyPair<A, B> + 'a) {
        self
    }
}

impl<'a, A: Hash, B: Hash> Hash for (dyn KeyPair<A, B> + 'a) {
    /// Crucially, this is the same way a tuple (A, B) is hashed. The implementation of std HashMap
    /// relies on this; see the documentation for `HashMap::get`.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.a().hash(state);
        self.b().hash(state);
    }
}

impl<'a, A: PartialEq, B: PartialEq> PartialEq for (dyn KeyPair<A, B> + 'a) {
    fn eq(&self, other: &dyn KeyPair<A, B>) -> bool {
        self.a() == other.a() && self.b() == other.b()
    }
}

impl<'a, A: Eq, B: Eq> Eq for (dyn KeyPair<A, B> + 'a) {}

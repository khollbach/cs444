use crate::types::Token;
use dfa::DFA;
use types::{State, StateSet};

mod dfa;
mod java_lang_nfa;
mod nfa;
mod nfa_to_dfa;
mod types;

pub struct Tokenizer {
    dfa: DFA<StateSet<State>>,
}

impl Tokenizer {
    pub fn new() -> Self {
        let nfa = java_lang_nfa::java_lang_nfa();
        let dfa = nfa.to_dfa();

        Self { dfa }
    }

    pub fn tokenize<'a>(&self, file: &'a str) -> Vec<Token<'a>> {
        self.dfa.tokenize(file)
    }
}

use crate::tokenizer::nfa::NFA;
use crate::tokenizer::states::{AcceptedStateLabel, State};
use crate::tokenizer::token_types::TokenType;
use crate::tokenizer::token_types::{Keyword, Operator, Separator};
use crate::tokenizer::token_types::{KEYWORDS, OPERATORS, SEPARATORS};
use crate::tokenizer::Symbol;
use std::collections::HashMap as Map;

mod constants;

/// Generate an NFA that recognizes the lexical grammar of Java (actually Joos 1W, there are some
/// differences; e.g. no floating point.).
pub fn java_lang_nfa() -> NFA<State> {
    let mut builder = NFABuilder::new();

    for &k in &KEYWORDS {
        builder.keyword(k);
    }

    for &sep in &SEPARATORS {
        builder.separator(sep);
    }

    for &op in &OPERATORS {
        builder.operator(op);
    }

    builder.whitespace();

    //builder.comments(); todo

    builder.identifiers();

    // todo: Implement the other token types.
    // There are reference NFAs given in the spec, so we can probably just copy those.
    // Make sure to put high-priority tokens above the others,
    // since token prio is encoded in the State `Ord` implementation.

    builder.nfa
}

/// An NFA builder for Joos 1W.
///
/// We don't do anything fancy with regexes; instead we just hand-code the NFA for each token type.
/// These NFAs are based on those given in the Java spec, linked on the course webpage.
struct NFABuilder {
    nfa: NFA<State>,
    num_states: u32,
}

impl NFABuilder {
    fn new() -> Self {
        Self {
            num_states: 1,
            nfa: NFA {
                init: State(0),
                accepted: Map::new(),
                delta: Map::new(),
                epsilon: Map::new(),
            },
        }
    }

    fn new_state(&mut self) -> State {
        let st = State(self.num_states);
        self.num_states += 1;
        st
    }

    fn keyword(&mut self, k: Keyword) {
        self.exact_match(&k.to_string(), TokenType::Keyword(k));
    }

    fn separator(&mut self, sep: Separator) {
        self.exact_match(&sep.to_string(), TokenType::Separator(sep));
    }

    fn operator(&mut self, op: Operator) {
        self.exact_match(&op.to_string(), TokenType::Operator(op));
    }

    /// Add states (and transitions) to the NFA for recognizing a specific sequence of symbols.
    ///
    /// This can be used to add a keyword to the tokenizer, for example.
    ///
    /// `s` must be ascii.
    fn exact_match(&mut self, s: &str, token_type: TokenType<'static>) {
        assert!(!s.is_empty());

        // Create an "initial" state for this token type.
        // Link `init` to it via an epsilon transition.
        let first = self.new_state();
        self.nfa
            .epsilon
            .entry(self.nfa.init)
            .or_default()
            .push(first);

        let mut prev = first;
        for sym in s.as_bytes().iter().copied().map(Symbol::new) {
            let curr = self.new_state();
            self.nfa.delta.entry((prev, sym)).or_default().push(curr);
            prev = curr;
        }

        // Accept the final state for this token type.
        let label = AcceptedStateLabel::Token(token_type);
        self.nfa.accepted.insert(prev, label);
    }

    /// Add states to recognize whitespace: any nonempty sequence of ' ', '\t', '\f', '\n'.
    fn whitespace(&mut self) {
        let first = self.new_state();
        let second = self.new_state();

        // Link to first.
        self.nfa
            .epsilon
            .entry(self.nfa.init)
            .or_default()
            .push(first);

        // Accept second.
        let label = AcceptedStateLabel::CommentOrWhitespace;
        self.nfa.accepted.insert(second, label);

        // Add transitions.
        for sym in constants::whitespace() {
            // first -> second
            self.nfa.delta.insert((first, sym), vec![second]);

            // second -> second
            self.nfa.delta.insert((second, sym), vec![second]);
        }
    }

    /// Add states to the NFA for recognizing identifiers. This should be called *after*
    /// `keywords()` and `literals()`, since ties are broken by which accepting state is smallest.
    fn identifiers(&mut self) {
        let first = self.new_state();
        let second = self.new_state();

        // Link to first.
        self.nfa
            .epsilon
            .entry(self.nfa.init)
            .or_default()
            .push(first);

        // Accept second.
        let s = "-*-java-identifier-*-";
        let label = AcceptedStateLabel::Token(TokenType::Identifier(s));
        self.nfa.accepted.insert(second, label);

        for sym in constants::letters() {
            // first -> second
            self.nfa.delta.insert((first, sym), vec![second]);

            // second -> second
            self.nfa.delta.insert((second, sym), vec![second]);
        }

        for sym in constants::digits() {
            // second -> second
            self.nfa.delta.insert((second, sym), vec![second]);
        }
    }
}

use crate::lexer::nfa::NFA;
use crate::lexer::types::{State, Symbol};
use crate::types::TokenType;
use crate::types::{Keyword, Literal, Operator, Separator};
use crate::types::{KEYWORDS, OPERATORS, SEPARATORS};
use std::collections::HashMap as Map;

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

    // todo: These are just placeholders, until I properly implement whitespace.
    builder.exact_match(" ", TokenType::Literal(Literal::Bool(false)));
    builder.exact_match("\n", TokenType::Literal(Literal::Bool(true)));

    // todo: Implement the other token types.
    // There are reference NFAs given in the spec, so we can probably just copy those.
    // Make sure to put high-priority tokens above the others,
    // since token prio is encoded in the State `Ord` implementation.

    builder.nfa
}

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
    /// This can be used to add a keyword to the lexer, for example.
    ///
    /// `s` must be ascii.
    fn exact_match(&mut self, s: &str, token_type: TokenType) {
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
        for c in s.chars() {
            let curr = self.new_state();
            self.nfa
                .delta
                .entry((prev, Symbol::new(c)))
                .or_default()
                .push(curr);
            prev = curr;
        }

        // Accept the final state for this token type.
        self.nfa.accepted.insert(prev, token_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Keyword::{Else, If, While};
    use crate::types::Literal::Bool;
    use crate::types::Operator::{Assign, Le};
    use crate::types::Separator::{Comma, Dot, LBrace};
    use crate::types::TokenType::{Keyword, Literal, Operator, Separator};

    /// Run a few simple examples; each should be successfully tokenized.
    #[test]
    fn tokenize_simple_examples() {
        // todo: remove or change this to pass once you implement whitespace.
        let sp = Literal(Bool(false));
        let nl = Literal(Bool(true));

        for (input, expected) in vec![
            (
                "if while else",
                vec![Keyword(If), sp, Keyword(While), sp, Keyword(Else)],
            ),
            (
                "if while\nelse",
                vec![Keyword(If), sp, Keyword(While), nl, Keyword(Else)],
            ),
            (
                "if{ ,.<==\n",
                vec![
                    Keyword(If),
                    Separator(LBrace),
                    sp,
                    Separator(Comma),
                    Separator(Dot),
                    Operator(Le),
                    Operator(Assign),
                    nl,
                ],
            ),
        ] {
            let nfa = java_lang_nfa();
            let dfa = nfa.to_dfa();

            let mut actual = vec![];
            for token in dfa.tokenize(&input) {
                actual.push(token.typ);
            }
            assert_eq!(expected, actual);
        }
    }
}

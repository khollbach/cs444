use crate::tokenizer::nfa::NFA;
use crate::tokenizer::states::{AcceptedStateLabel, State};
use crate::tokenizer::token_types::{Keyword, Operator, Separator};
use crate::tokenizer::token_types::{Literal, TokenType};
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

    builder.literals();
    builder.whitespace();
    builder.comments();

    // This should appear last, to correctly break ties in favor of keywords, etc.
    builder.identifiers();

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
        let state = State(self.num_states);
        self.num_states += 1;
        state
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

    fn literals(&mut self) {
        self.ints();
        self.bools();
        self.chars();
        self.strings();
        self.null();
    }

    fn ints(&mut self) {
        //todo!()
    }

    /// Boolean literals `true` and `false`. The accepted states are labelled with the
    /// corresponding boolean values.
    fn bools(&mut self) {
        self.exact_match("false", TokenType::Literal(Literal::Bool(false)));
        self.exact_match("true", TokenType::Literal(Literal::Bool(true)));
    }

    /// Recognize string literals.
    fn strings(&mut self) {
        let filler = "-*-java-string-literal-*-";
        let label = AcceptedStateLabel::Token(TokenType::Literal(Literal::String(filler)));
        self.strings_or_chars(Symbol::new(b'"'), label);
    }

    /// Recognize character literals.
    fn chars(&mut self) {
        let filler = '?';
        let label = AcceptedStateLabel::Token(TokenType::Literal(Literal::Char(filler)));
        self.strings_or_chars(Symbol::new(b'\''), label);
    }

    /// Helper function for `self.strings()` and `self.chars()`, so I don't repeat myself.
    ///
    /// Add states to the NFA to recognize either string literals or char literals.
    fn strings_or_chars(&mut self, quote: Symbol, label: AcceptedStateLabel) {
        let start = self.new_state();
        let inner = self.new_state();
        let odd_backslash = self.new_state(); // "I've just seen an *odd* number of backslashes."
        let end = self.new_state();

        // Link to start; accept end.
        self.nfa
            .epsilon
            .entry(self.nfa.init)
            .or_default()
            .push(start);
        self.nfa.accepted.insert(end, label);

        // start -> inner
        // inner -> end
        self.nfa
            .delta
            .entry((start, quote))
            .or_default()
            .push(inner);
        self.nfa.delta.entry((inner, quote)).or_default().push(end);

        // inner -> odd_backslash
        let bslash = Symbol::new(b'\\');
        self.nfa
            .delta
            .entry((inner, bslash))
            .or_default()
            .push(odd_backslash);

        // odd_backslash -> inner
        // Everything except newline (which by ommission leads to the implicit "dead" state).
        for sym in constants::all_symbols() {
            if sym.to_char() != '\n' {
                self.nfa
                    .delta
                    .entry((odd_backslash, sym))
                    .or_default()
                    .push(inner);
            }
        }

        // inner -> inner
        // Everything except quote, backslash, newline.
        // By ommision, newline leads to the implicit "dead" state.
        for sym in constants::all_symbols() {
            match sym.to_char() {
                '"' | '\\' | '\n' => (),
                _ => {
                    self.nfa.delta.entry((inner, sym)).or_default().push(inner);
                }
            }
        }
    }

    /// `null` literal. Basically a keyword as far as the tokenizer is concerned.
    fn null(&mut self) {
        self.exact_match("null", TokenType::Literal(Literal::Null));
    }

    fn comments(&mut self) {
        //todo!()
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
        let filler = "-*-java-identifier-*-";
        let label = AcceptedStateLabel::Token(TokenType::Identifier(filler));
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

#[cfg(test)]
mod tests {
    use crate::tokenizer::tests::TestCase;
    use crate::tokenizer::token_types::Literal::String as StringLit;
    use crate::tokenizer::token_types::TokenType::Literal;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn simple_string_lit() {
        TestCase {
            input: vec!["\"asdf\""],
            expected_output: vec![Literal(StringLit("asdf"))],
        }
        .run(&Tokenizer::new())
    }

    #[test]
    fn string_lit_escape_quote() {
        // todo: update the values in the `expected` tokens once we actually implement string
        // escapes.

        let tokenizer = Tokenizer::new();
        for (input, expected_output) in vec![
            (
                vec!["\"asdf\\\"asdf\""],
                vec![Literal(StringLit("asdf\\\"asdf"))],
            ),
            (
                vec!["  \"abcabc\\\\abc\"  "],
                vec![Literal(StringLit("abcabc\\\\abc"))],
            ),
        ] {
            TestCase {
                input,
                expected_output,
            }
            .run(&tokenizer)
        }
    }
}

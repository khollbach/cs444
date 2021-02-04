use crate::tokenizer::nfa::NFA;
use crate::tokenizer::states::AcceptedStateLabel::{
    JavadocComment, LineComment, StarComment, TokenType as AcceptedToken, Whitespace,
};
use crate::tokenizer::states::{AcceptedStateLabel, State};
use crate::tokenizer::tokens::Literal::{Bool, Char, Int, Null, StringLit};
use crate::tokenizer::tokens::Token::{Identifier, Literal};
use crate::tokenizer::tokens::{Keyword, Operator, Separator, Token};
use crate::tokenizer::tokens::{KEYWORDS, OPERATORS, SEPARATORS};
use crate::tokenizer::Symbol;
use std::collections::HashMap as Map;
use StarCommentType::{Javadoc, OneStar};

mod constants;

/// Generate an NFA that recognizes the lexical grammar of Java (actually Joos 1W, there are some
/// differences; e.g. no floating point.).
///
/// We'll end up successfully tokenizing some operators that technically aren't in Joos 1W, e.g.
/// right-shift-assign (">>="), etc. We'll catch these later during parsing though, so that's fine
/// with me.
pub fn nfa() -> NFA<State> {
    let mut builder = NFABuilder::new();

    builder.comments();
    builder.whitespace();

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

    // This should appear after most things, to correctly break ties in favor of keywords, null
    // literals, etc.
    builder.identifiers();

    builder.nfa
}

/// Input type of the helper method `star_comments`.
enum StarCommentType {
    OneStar,
    Javadoc,
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
    /// A new NFA with just an initial state.
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

    /// Add a new state to the NFA, with increasing sequence numbers.
    ///
    /// (The state itself isn't actually stored in the NFA, but this helps us make sure states are
    /// unique.)
    fn new_state(&mut self) -> State {
        let state = State(self.num_states);
        self.num_states += 1;
        state
    }

    /// Link from `init` to `start` via an epsilon transition.
    fn eps(&mut self, start: State) {
        self.nfa
            .epsilon
            .entry(self.nfa.init)
            .or_default()
            .push(start);
    }

    /// Add a delta transition to the NFA.
    fn delta(&mut self, src: State, sym: Symbol, dest: State) {
        self.nfa.delta.entry((src, sym)).or_default().push(dest);
    }

    /// Add a delta transition to the NFA, given an ASCII char.
    fn delta_char(&mut self, src: State, ch: char, dest: State) {
        assert!(ch < 128 as char);
        let sym = Symbol::new(ch as u8);
        self.delta(src, sym, dest);
    }

    /// Add a keyword to the NFA.
    fn keyword(&mut self, k: Keyword) {
        self.exact_match(&k.to_string(), Token::Keyword(k));
    }

    /// Add a separator to the NFA.
    fn separator(&mut self, sep: Separator) {
        self.exact_match(&sep.to_string(), Token::Separator(sep));
    }

    /// Add an operator to the NFA.
    fn operator(&mut self, op: Operator) {
        self.exact_match(&op.to_string(), Token::Operator(op));
    }

    /// Add states and transitions to the NFA for recognizing a specific sequence of symbols.
    ///
    /// This can be used to add a keyword to the tokenizer, for example. `s` must be ascii.
    ///
    /// Basically, this just generates a linked list of states.
    fn exact_match(&mut self, s: &str, type_: Token<'static>) {
        assert!(!s.is_empty());

        let start = self.new_state();
        self.eps(start);

        let mut prev = start;
        for c in s.chars() {
            let curr = self.new_state();
            self.delta_char(prev, c, curr);
            prev = curr;
        }

        let label = AcceptedToken { type_ };
        self.nfa.accepted.insert(prev, label);
    }

    /// Add all types of literals to the NFA.
    fn literals(&mut self) {
        self.ints();
        self.bools();
        self.chars();
        self.strings();
        self.null();
    }

    /// Add int literals to the NFA.
    fn ints(&mut self) {
        self.zero();
        self.non_zero();
    }

    /// Add states to recognize the zero literal: `0`.
    fn zero(&mut self) {
        let start = self.new_state();
        let end = self.new_state();

        self.eps(start);

        let label = AcceptedToken {
            type_: Literal(Int(0)),
        };
        self.nfa.accepted.insert(end, label);

        // start -> end
        self.delta_char(start, '0', end);
    }

    /// Add states to recognize non-zero int literals, e.g. `10234`.
    ///
    /// Always positive, since unary negation is lexed separately.
    fn non_zero(&mut self) {
        let start = self.new_state();
        let end = self.new_state();

        self.eps(start);

        let filler = 55555;
        let label = AcceptedToken {
            type_: Literal(Int(filler)),
        };
        self.nfa.accepted.insert(end, label);

        // start -> end
        for sym in constants::digits() {
            if sym.to_char() != '0' {
                self.delta(start, sym, end);
            }
        }

        // end -> end
        for sym in constants::digits() {
            self.delta(end, sym, end);
        }
    }

    /// Boolean literals `true` and `false`. The accepted states are labelled with the
    /// corresponding boolean values.
    fn bools(&mut self) {
        self.exact_match("false", Literal(Bool(false)));
        self.exact_match("true", Literal(Bool(true)));
    }

    /// Recognize string literals.
    fn strings(&mut self) {
        let filler = String::from("-*-java-string-literal-*-");
        let label = AcceptedToken {
            type_: Literal(StringLit(filler)),
        };
        self.strings_or_chars('"', label);
    }

    /// Recognize character literals.
    fn chars(&mut self) {
        let filler = '?';
        let label = AcceptedToken {
            type_: Literal(Char(filler)),
        };
        self.strings_or_chars('\'', label);
    }

    /// Helper function for `self.strings()` and `self.chars()`, so I don't repeat myself.
    ///
    /// Add states to the NFA to recognize either string literals or char literals.
    fn strings_or_chars(&mut self, quote: char, label: AcceptedStateLabel) {
        let start = self.new_state();
        let inner = self.new_state();
        let odd_backslash = self.new_state(); // "I've just seen an *odd* number of backslashes."
        let end = self.new_state();

        // Link to start; accept end.
        self.eps(start);
        self.nfa.accepted.insert(end, label);

        // start -> inner
        self.delta_char(start, quote, inner);

        // inner -> end
        self.delta_char(inner, quote, end);

        // inner -> odd_backslash
        self.delta_char(inner, '\\', odd_backslash);

        // odd_backslash -> inner
        // Everything except newline (which by ommission leads to the implicit "dead" state).
        for sym in constants::all_symbols() {
            if sym.to_char() != '\n' {
                self.delta(odd_backslash, sym, inner);
            }
        }

        // inner -> inner
        // Everything except quote, backslash, newline.
        // By ommision, newline leads to the implicit "dead" state.
        for sym in constants::all_symbols() {
            match sym.to_char() {
                '"' | '\\' | '\n' => (),
                _ => {
                    self.delta(inner, sym, inner);
                }
            }
        }
    }

    /// `null` literal. Basically a keyword as far as the tokenizer is concerned.
    fn null(&mut self) {
        self.exact_match("null", Literal(Null));
    }

    /// Add states to the NFA for recognizing identifiers. This should be called *after*
    /// `keywords()` and `literals()`, since ties are broken by which accepting state is smallest.
    fn identifiers(&mut self) {
        let start = self.new_state();
        let end = self.new_state();

        self.eps(start);

        let filler = "-*-java-identifier-*-";
        let label = AcceptedToken {
            type_: Identifier(filler),
        };
        self.nfa.accepted.insert(end, label);

        for sym in constants::letters() {
            // start -> end
            self.delta(start, sym, end);

            // end -> end
            self.delta(end, sym, end);
        }

        for sym in constants::digits() {
            // end -> end
            self.delta(end, sym, end);
        }
    }

    /// Add comments to the NFA.
    fn comments(&mut self) {
        self.line_comments();

        // Javadoc comments should precede normal ones, to correctly break the tie.
        self.star_comments(StarCommentType::Javadoc);
        self.star_comments(StarCommentType::OneStar);
    }

    /// Add states to recognize line comments.
    fn line_comments(&mut self) {
        let start = self.new_state();
        let one_slash = self.new_state();
        let two_slash = self.new_state();
        // red slash, blue slash
        let end = self.new_state();

        self.eps(start);

        let label = LineComment;
        self.nfa.accepted.insert(end, label);

        // start -> one_slash
        // one_slash -> two_slash
        self.delta_char(start, '/', one_slash);
        self.delta_char(one_slash, '/', two_slash);

        // two_slash -> two_slash
        for sym in constants::all_symbols() {
            if sym.to_char() != '\n' {
                self.delta(two_slash, sym, two_slash);
            }
        }

        // two_slash -> end
        self.delta_char(two_slash, '\n', end);
    }

    /// Add states to recognize star comments (normal or javadoc).
    fn star_comments(&mut self, val: StarCommentType) {
        let start = self.new_state();
        let slash = self.new_state();
        let inner = self.new_state();
        let inner_star = self.new_state();
        let end = self.new_state();

        self.eps(start);

        let label = match val {
            StarCommentType::OneStar => StarComment,
            // For now, we're just silent discarding doc comments; same as regular comments.
            // Maybe at some point we'll want to do something different with these...
            StarCommentType::Javadoc => JavadocComment,
        };
        self.nfa.accepted.insert(end, label);

        // start -> slash
        self.delta_char(start, '/', slash);

        // slash -> inner
        match val {
            OneStar => {
                self.delta_char(slash, '*', inner);
            }
            Javadoc => {
                // Add one extra star before 'inner'.
                let first_star = self.new_state();
                self.delta_char(slash, '*', first_star);
                self.delta_char(first_star, '*', inner);
            }
        }

        // inner -> inner
        for sym in constants::all_symbols() {
            if sym.to_char() != '*' {
                self.delta(inner, sym, inner);
            }
        }

        // inner -> inner_star
        self.delta_char(inner, '*', inner_star);

        // inner_star -> inner_star
        self.delta_char(inner_star, '*', inner_star);

        // inner_star -> inner
        for sym in constants::all_symbols() {
            match sym.to_char() {
                '*' | '/' => (),
                _ => {
                    self.delta(inner_star, sym, inner);
                }
            }
        }

        // inner_star -> end
        self.delta_char(inner_star, '/', end);
    }

    /// Add states to recognize whitespace: any nonempty sequence of ' ', '\t', '\f', '\n'.
    fn whitespace(&mut self) {
        let start = self.new_state();
        let end = self.new_state();

        self.eps(start);

        let label = Whitespace;
        self.nfa.accepted.insert(end, label);

        for sym in constants::whitespace() {
            // start -> end
            self.delta(start, sym, end);

            // end -> end
            self.delta(end, sym, end);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tests::TestCase;
    use crate::tokenizer::tokens::Literal::StringLit;
    use crate::tokenizer::tokens::Token::Literal;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn simple_string_lit() {
        TestCase {
            input: vec!["\"asdf\""],
            expected_output: vec![Literal(StringLit(String::from("asdf")))],
        }
        .run(&Tokenizer::new())
    }

    #[test]
    fn string_lit_escape_quote() {
        // todo fix these after impl'ing esc seq's

        let tokenizer = Tokenizer::new();
        for (input, expected_output) in vec![
            (
                vec!["\"asdf\\\"asdf\""],
                vec![Literal(StringLit(String::from("asdf\"asdf")))],
            ),
            (
                vec!["  \"abcabc\\\\abc\"  "],
                vec![Literal(StringLit(String::from("abcabc\\abc")))],
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

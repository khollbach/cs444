/// Diffent types of tokens in the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    If,
    //Else,
    //While,
    // etc...
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub typ: TokenType,
    pub lexeme: &'a str,
}

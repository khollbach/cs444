// For now, so we don't get "unused variant" warnings:
#![allow(dead_code)]

use std::fmt;

/// A token in the output stream of the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub typ: TokenType,
    pub lexeme: &'a str,
}

/// Diffent types of tokens in the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Ident,
    Keyword(Keyword),
    Literal(Literal),
    Separator(Separator),
    Operator(Operator),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Abstract,
    Boolean,
    Break,
    Byte,
    Case,
    Catch,
    Char,
    Class,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Extends,
    Final,
    Finally,
    Float,
    For,
    Goto,
    If,
    Implements,
    Import,
    Instanceof,
    Int,
    Interface,
    Long,
    Native,
    New,
    Package,
    Private,
    Protected,
    Public,
    Return,
    Short,
    Static,
    Strictfp,
    Super,
    Switch,
    Synchronized,
    This,
    Throw,
    Throws,
    Transient,
    Try,
    Void,
    Volatile,
    While,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    //Int(u32),
    Bool(bool),
    //Char(char),
    //String(&str),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Dot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Assign,
    Gt,
    Lt,
    Not,
    BitNot,
    Question,
    Colon,
    Eq,
    Le,
    Ge,
    Ne,
    And,
    Or,
    Increment,
    Decrement,
    Plus,
    Minus,
    Star,
    Divide,
    BitAnd,
    BitOr,
    BirXor,
    Mod,
    LShift,
    RShift,
    URShift,
    PlusEq,
    MinusEq,
    TimesEq,
    DivideEq,
    BitAndEq,
    BitOrEq,
    BitXorEq,
    ModEq,
    LShiftEq,
    RShiftEq,
    URShiftEq,
}

pub const KEYWORDS: [Keyword; 48] = {
    use Keyword::*;

    [
        Abstract,
        Boolean,
        Break,
        Byte,
        Case,
        Catch,
        Char,
        Class,
        Const,
        Continue,
        Default,
        Do,
        Double,
        Else,
        Extends,
        Final,
        Finally,
        Float,
        For,
        Goto,
        If,
        Implements,
        Import,
        Instanceof,
        Int,
        Interface,
        Long,
        Native,
        New,
        Package,
        Private,
        Protected,
        Public,
        Return,
        Short,
        Static,
        Strictfp,
        Super,
        Switch,
        Synchronized,
        This,
        Throw,
        Throws,
        Transient,
        Try,
        Void,
        Volatile,
        While,
    ]
};

pub const SEPARATORS: [Separator; 9] = {
    use Separator::*;

    [
        LParen, RParen, LBrace, RBrace, LBracket, RBracket, Semicolon, Comma, Dot,
    ]
};

pub const OPERATORS: [Operator; 37] = {
    use Operator::*;

    [
        Assign, Gt, Lt, Not, BitNot, Question, Colon, Eq, Le, Ge, Ne, And, Or, Increment,
        Decrement, Plus, Minus, Star, Divide, BitAnd, BitOr, BirXor, Mod, LShift, RShift, URShift,
        PlusEq, MinusEq, TimesEq, DivideEq, BitAndEq, BitOrEq, BitXorEq, ModEq, LShiftEq, RShiftEq,
        URShiftEq,
    ]
};

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Keyword::*;

        let s = match self {
            Abstract => "abstract",
            Boolean => "boolean",
            Break => "break",
            Byte => "byte",
            Case => "case",
            Catch => "catch",
            Char => "char",
            Class => "class",
            Const => "const",
            Continue => "continue",
            Default => "default",
            Do => "do",
            Double => "double",
            Else => "else",
            Extends => "extends",
            Final => "final",
            Finally => "finally",
            Float => "float",
            For => "for",
            Goto => "goto",
            If => "if",
            Implements => "implements",
            Import => "import",
            Instanceof => "instanceof",
            Int => "int",
            Interface => "interface",
            Long => "long",
            Native => "native",
            New => "new",
            Package => "package",
            Private => "private",
            Protected => "protected",
            Public => "public",
            Return => "return",
            Short => "short",
            Static => "static",
            Strictfp => "strictfp",
            Super => "super",
            Switch => "switch",
            Synchronized => "synchronized",
            This => "this",
            Throw => "throw",
            Throws => "throws",
            Transient => "transient",
            Try => "try",
            Void => "void",
            Volatile => "volatile",
            While => "while",
        };

        write!(f, "{}", s)
    }
}

impl fmt::Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Separator::*;

        let s = match self {
            LParen => "(",
            RParen => ")",
            LBrace => "{",
            RBrace => "}",
            LBracket => "[",
            RBracket => "]",
            Semicolon => ";",
            Comma => ",",
            Dot => ".",
        };

        write!(f, "{}", s)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Operator::*;

        let s = match self {
            Assign => "=",
            Gt => ">",
            Lt => "<",
            Not => "!",
            BitNot => "~",
            Question => "?",
            Colon => ":",
            Eq => "==",
            Le => "<=",
            Ge => ">=",
            Ne => "!=",
            And => "&&",
            Or => "||",
            Increment => "++",
            Decrement => "--",
            Plus => "+",
            Minus => "-",
            Star => "*",
            Divide => "/",
            BitAnd => "&",
            BitOr => "|",
            BirXor => "^",
            Mod => "%",
            LShift => "<<",
            RShift => ">>",
            URShift => ">>>",
            PlusEq => "+=",
            MinusEq => "-=",
            TimesEq => "*=",
            DivideEq => "/=",
            BitAndEq => "&=",
            BitOrEq => "|=",
            BitXorEq => "^=",
            ModEq => "%=",
            LShiftEq => "<<=",
            RShiftEq => ">>=",
            URShiftEq => ">>>=",
        };

        write!(f, "{}", s)
    }
}

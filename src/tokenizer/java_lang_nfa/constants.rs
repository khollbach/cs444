use crate::tokenizer::Symbol;

pub fn whitespace() -> impl Iterator<Item = Symbol> {
    WHITE_SPACE.iter().map(|&c| Symbol::new(c as u8))
}

pub fn digits() -> impl Iterator<Item = Symbol> {
    DIGITS.iter().map(|&c| Symbol::new(c as u8))
}

pub fn letters() -> impl Iterator<Item = Symbol> {
    LETTERS.iter().map(|&c| Symbol::new(c as u8))
}

/// As defined in the Java spec.
///
/// (0x0C is ASCII form feed.)
const WHITE_SPACE: [char; 4] = [' ', '\t', '\x0C', '\n'];

/// As defined in the Java spec. (Obviously.)
const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// As defined in the Java spec.
///
/// Includes '_', and for or some reason, '$'.
const LETTERS: [char; 54] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'Y', 'X', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'y', 'x', 'z', '_', '$',
];

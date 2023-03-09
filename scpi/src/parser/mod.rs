//! SCPI Parser and response formatter
//!

pub mod expression;
pub mod parameters;
pub mod response;
pub mod suffix;
pub mod tokenizer;

pub use tokenizer::util::{mnemonic_compare, mnemonic_match};

/// Wrappers to format and discriminate SCPI types
pub mod format {

    /// Hexadecimal data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Hex<V>(pub V);

    /// Binary data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Binary<V>(pub V);

    /// Octal data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Octal<V>(pub V);

    /// Arbitrary data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Arbitrary<'a>(pub &'a [u8]);

    /// Expression data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Expression<'a>(pub &'a [u8]);

    /// Character data
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Character<'a>(pub &'a [u8]);
}

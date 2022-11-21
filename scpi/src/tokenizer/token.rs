use crate::prelude::*;
use crate::util;

/// SCPI tokens
/// Loosely based on IEEE488.2 Chapter 7
///
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Token<'a> {
    /// A header mnemonic separator `:`
    HeaderMnemonicSeparator,
    /// A header query suffix `?`
    HeaderQuerySuffix,
    /// A message unit separator `;`
    ProgramMessageUnitSeparator,
    /// A Program header separator ` `
    ProgramHeaderSeparator,
    /// A program data separator ','
    ProgramDataSeparator,
    /// A program mnemonic
    ProgramMnemonic(&'a [u8]),
    /// A <CHARACTER PROGRAM DATA> 7.7.1
    CharacterProgramData(&'a [u8]),
    /// A <DECIMAL NUMERIC PROGRAM DATA> 7.7.2
    DecimalNumericProgramData(&'a [u8]),
    /// A <DECIMAL NUMERIC PROGRAM DATA> 7.7.2 followed by a <SUFFIX PROGRAM DATA> 7.7.3
    DecimalNumericSuffixProgramData(&'a [u8], &'a [u8]),
    /// A <NONDECIMAL NUMERIC PROGRAM DATA> 7.7.4
    NonDecimalNumericProgramData(u64),
    /// A <STRING PROGRAM DATA> 7.7.5
    StringProgramData(&'a [u8]),
    /// A <ARBITRARY BLOCK PROGRAM DATA> 7.7.6
    ArbitraryBlockData(&'a [u8]),
    /// A <EXPRESSION PROGRAM DATA> 7.7.7
    ExpressionProgramData(&'a [u8]),
}

impl<'a> Token<'a> {
    pub fn is_data(&self) -> bool {
        matches!(
            self,
            Self::CharacterProgramData(_)
                | Self::DecimalNumericProgramData(_)
                | Self::DecimalNumericSuffixProgramData(_, _)
                | Self::NonDecimalNumericProgramData(_)
                | Self::StringProgramData(_)
                | Self::ArbitraryBlockData(_)
                | Self::ExpressionProgramData(_)
        )
    }

    /// Returns true if token is a ProgramMnemonic that matches provided mnemonic.
    /// Header suffix is optional if equal to 1 or not present in mnemonic.
    /// Header suffixes other than 1 must match exactly.
    ///
    /// Eg:
    /// - `head[er]` == `HEADer`
    /// - `head[er]1` == `HEADer`
    /// - `head[er]` == `HEADer1`
    /// - `head[er]<N>` == `HEADer<N>`
    /// Where `[]` marks optional, `<>` required.
    ///
    pub fn match_program_header(&self, mnemonic: &'a [u8]) -> bool {
        match self {
            Token::ProgramMnemonic(s) | Token::CharacterProgramData(s) => {
                util::mnemonic_compare(mnemonic, s)
                    || match (
                        util::mnemonic_split_index(mnemonic),
                        util::mnemonic_split_index(s),
                    ) {
                        (None, None) => false,
                        (Some((m, index)), None) => util::mnemonic_compare(m, s) && index == b"1",
                        (None, Some((x, index))) => {
                            util::mnemonic_compare(mnemonic, x) && index == b"1"
                        }
                        (Some((m, index1)), Some((x, index2))) => {
                            util::mnemonic_compare(m, x) && (index1 == index2)
                        }
                    }
            }
            _ => false,
        }
    }
}

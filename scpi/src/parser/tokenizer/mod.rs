//! The tokenizer splits a SCPI command into more managable tokens.
//!   

use crate::error::ErrorCode;

use core::slice::Iter;

pub use self::token::Token;

mod token;
pub mod util;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Tokenizer<'a> {
    pub chars: Iter<'a, u8>,
    in_header: bool,
    in_common: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Tokenizer::from_byte_iter(buf.iter())
    }

    pub fn new_params(buf: &'a [u8]) -> Self {
        let mut toks = Tokenizer::from_byte_iter(buf.iter());
        toks.in_header = false;
        toks
    }

    pub(crate) fn from_byte_iter(iter: Iter<'a, u8>) -> Self {
        Tokenizer {
            chars: iter,
            in_header: true,
            in_common: false,
        }
    }

    /// <program mnemonic>
    /// See IEEE 488.2-1992 7.6.1
    /// Must start with a alphabetic character followed by alphanumeric and '_' characters.
    ///
    /// Returned errors:
    /// * ProgramMnemonicTooLong if suffix is longer than 12 characters
    fn read_mnemonic(&mut self, mut common: bool) -> Result<Token<'a>, ErrorCode> {
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self.chars.clone().next().map_or(false, |ch| {
            ch.is_ascii_alphanumeric() || *ch == b'_' || (*ch == b'*' && common)
        }) {
            common = false;
            self.chars.next();
            len += 1;
            if len > 12 {
                return Err(ErrorCode::ProgramMnemonicTooLong);
            }
        }
        Ok(Token::ProgramMnemonic(
            &s[0..s.len() - self.chars.as_slice().len()],
        ))
    }

    /// <CHARACTER PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.1
    /// Consists of a single <program mnemonic>
    ///
    /// Returned errors:
    /// * CharacterDataTooLong if suffix is longer than 12 characters
    pub(crate) fn read_character_data(&mut self) -> Result<Token<'a>, ErrorCode> {
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self
            .chars
            .clone()
            .next()
            .map_or(false, |ch| ch.is_ascii_alphanumeric() || *ch == b'_')
        {
            self.chars.next();
            len += 1;
            if len > 12 {
                return Err(ErrorCode::CharacterDataTooLong);
            }
        }
        let ret = Ok(Token::CharacterProgramData(
            &s[0..s.len() - self.chars.as_slice().len()],
        ));

        // Skip to next separator
        self.skip_ws_to_separator(ErrorCode::InvalidCharacterData)?;
        ret
    }

    pub(crate) fn read_nrf(&mut self) -> Result<Token<'a>, ErrorCode> {
        let s = self.chars.as_slice();
        /* Read leading +/- */
        util::skip_sign(&mut self.chars);
        /* Read mantissa */
        let leading_digits = util::skip_digits(&mut self.chars);
        /* Read fraction (required if no leading digits were read) */
        if let Some(b'.') = self.chars.clone().next() {
            self.chars.next().unwrap();
            if !util::skip_digits(&mut self.chars) && !leading_digits {
                return Err(ErrorCode::NumericDataError);
            }
        } else if !leading_digits {
            return Err(ErrorCode::NumericDataError);
        }
        //TODO: Lexical-core doesn't like ws around Exponent
        //util::skip_ws(self.chars);
        /* Read exponent */
        if let Some(exponent) = self.chars.clone().next() {
            if *exponent == b'E' || *exponent == b'e' {
                self.chars.next().unwrap();
                //TODO: Lexical-core doesn't like ws around Exponent
                //util::skip_ws(self.chars);
                util::skip_sign(&mut self.chars);
                if !util::skip_digits(&mut self.chars) {
                    return Err(ErrorCode::NumericDataError);
                }
            }
        }
        Ok(Token::DecimalNumericProgramData(
            &s[0..s.len() - self.chars.as_slice().len()],
        ))
    }

    /// <DECIMAL NUMERIC PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.2
    ///
    /// TODO: lexical-core does not accept whitespace between exponent separator and exponent <mantissa>E <exponent>.
    pub(crate) fn read_numeric_data(&mut self) -> Result<Token<'a>, ErrorCode> {
        let tok = self.read_nrf()?;
        if let Token::DecimalNumericProgramData(s) = tok {
            util::skip_ws(&mut self.chars);
            if let Some(x) = self.chars.clone().next() {
                if x.is_ascii_alphabetic() || *x == b'/' {
                    return self.read_suffix_data(s);
                } else {
                    self.skip_ws_to_separator(ErrorCode::InvalidSuffix)?;
                }
            }
        }
        Ok(tok)
    }

    /// <SUFFIX PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.3
    /// Reads a suffix and returns it as a string if successful, otherwise it returns an error.
    ///
    /// Returned errors:
    /// * SuffixTooLong if suffix is longer than 12 characters
    fn read_suffix_data(&mut self, val: &'a [u8]) -> Result<Token<'a>, ErrorCode> {
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self.chars.clone().next().map_or(false, |ch| {
            ch.is_ascii_alphanumeric() || *ch == b'-' || *ch == b'/' || *ch == b'.'
        }) {
            self.chars.next();
            len += 1;
            if len > 12 {
                return Err(ErrorCode::SuffixTooLong);
            }
        }

        let ret = Ok(Token::DecimalNumericSuffixProgramData(
            val,
            &s[0..s.len() - self.chars.as_slice().len()],
        ));
        // Skip to next separator
        self.skip_ws_to_separator(ErrorCode::InvalidSuffix)?;
        ret
    }

    /// <NONDECIMAL NUMERIC PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.4
    /// Reads a non-decimal numeric
    ///
    /// Returned errors:
    fn read_nondecimal_data(&mut self, radix: u8) -> Result<Token<'a>, ErrorCode> {
        let options = lexical_core::ParseIntegerOptions::new();
        let (n, len) = match radix {
            b'H' | b'h' => {
                const FORMAT: u128 = lexical_core::NumberFormatBuilder::from_radix(16);
                lexical_core::parse_partial_with_options::<u64, FORMAT>(
                    self.chars.as_slice(),
                    &options,
                )
            }
            b'Q' | b'q' => {
                const FORMAT: u128 = lexical_core::NumberFormatBuilder::from_radix(8);
                lexical_core::parse_partial_with_options::<u64, FORMAT>(
                    self.chars.as_slice(),
                    &options,
                )
            }
            b'B' | b'b' => {
                const FORMAT: u128 = lexical_core::NumberFormatBuilder::from_radix(2);
                lexical_core::parse_partial_with_options::<u64, FORMAT>(
                    self.chars.as_slice(),
                    &options,
                )
            }
            _ => return Err(ErrorCode::NumericDataError),
        }
        .map_err(|e| match e {
            lexical_core::Error::InvalidDigit(_) => ErrorCode::InvalidCharacterInNumber,
            lexical_core::Error::Overflow(_) | lexical_core::Error::Underflow(_) => {
                ErrorCode::DataOutOfRange
            }
            _ => ErrorCode::NumericDataError,
        })?;
        if len > 0 {
            self.chars.nth(len - 1).unwrap();
            let ret = Token::NonDecimalNumericProgramData(n);
            // Skip to next separator
            self.skip_ws_to_separator(ErrorCode::SuffixNotAllowed)?;
            Ok(ret)
        } else {
            Err(ErrorCode::NumericDataError)
        }
    }

    /// <STRING PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.5
    ///
    pub(crate) fn read_string_data(&mut self, x: u8, ascii: bool) -> Result<Token<'a>, ErrorCode> {
        self.chars.next(); //Consume first
        let s = self.chars.as_slice();
        loop {
            if let Some(c) = self.chars.next() {
                // End quote
                if *c == x {
                    if let Some(c2) = self.chars.clone().next() {
                        if *c2 == x {
                            // Double quote, consume and continue
                            self.chars.next().unwrap();
                            continue;
                        } else {
                            // Single quote
                            break;
                        }
                    }
                    break;
                }

                //Only ASCII allowed
                if ascii && !c.is_ascii() {
                    return Err(ErrorCode::InvalidCharacter);
                }
            } else {
                // Unexpected terminator
                return Err(ErrorCode::InvalidStringData);
            }
        }
        let ret = Ok(Token::StringProgramData(
            &s[0..s.len() - self.chars.as_slice().len() - 1],
        ));
        // Skip to next separator
        self.skip_ws_to_separator(ErrorCode::SuffixNotAllowed)?;
        ret
    }

    /// <ARBITRARY DATA PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.6
    ///
    fn read_arbitrary_data(&mut self, format: u8) -> Result<Token<'a>, ErrorCode> {
        if let Some(len) = util::ascii_to_digit(format, 10) {
            if len == 0 {
                //Take rest of string
                let rest = self.chars.as_slice();
                if rest.is_empty() {
                    return Err(ErrorCode::InvalidBlockData);
                }
                let u8str = rest
                    .get(0..rest.len() - 1)
                    .ok_or(ErrorCode::InvalidBlockData)?;
                for _ in u8str {
                    self.chars.next();
                }
                //Indefinite block data must be terminated with NL before END
                if *self.chars.next().unwrap() != b'\n' {
                    return Err(ErrorCode::InvalidBlockData);
                }
                return Ok(Token::ArbitraryBlockData(u8str));
            }

            let payload_len = lexical_core::parse::<usize>(
                self.chars
                    .as_slice()
                    .get(..len as usize)
                    .ok_or(ErrorCode::InvalidBlockData)?,
            )
            .map_err(|_| ErrorCode::InvalidBlockData)?;
            self.chars.nth(len as usize - 1).unwrap();
            let u8str = self
                .chars
                .as_slice()
                .get(0..payload_len)
                .ok_or(ErrorCode::InvalidBlockData)?;
            for _ in 0..payload_len {
                self.chars.next();
            }

            let ret = Ok(Token::ArbitraryBlockData(u8str));
            // Skip to next separator
            self.skip_ws_to_separator(ErrorCode::SuffixNotAllowed)?;
            ret
        } else {
            Err(ErrorCode::InvalidBlockData)
        }
    }

    /// <EXPRESSION PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.7
    ///
    pub(crate) fn read_expression_data(&mut self) -> Result<Token<'a>, ErrorCode> {
        self.chars.next();
        let s = self.chars.as_slice();
        static ILLEGAL_CHARS: &[u8] = &[b'"', b'\'', b';', b'(', b')'];
        //Read until closing ')'
        while self.chars.clone().next().map_or(false, |ch| *ch != b')') {
            let c = self.chars.next().unwrap();
            //Return an error if a unexpected character is encountered
            if ILLEGAL_CHARS.contains(c) || !c.is_ascii() {
                return Err(ErrorCode::InvalidExpression);
            }
        }

        let ret = Ok(Token::ExpressionProgramData(
            &s[0..s.len() - self.chars.as_slice().len()],
        ));
        //Consume ending ')' or throw error if there's none
        if self.chars.next().is_none() {
            return Err(ErrorCode::InvalidExpression);
        }
        // Skip to next separator
        self.skip_ws_to_separator(ErrorCode::SuffixNotAllowed)?;
        ret
    }

    fn skip_ws_to_separator(&mut self, error: ErrorCode) -> Result<(), ErrorCode> {
        util::skip_ws(&mut self.chars);
        if let Some(c) = self.chars.clone().next() {
            if *c != b',' && *c != b';' && *c != b'\n' {
                return Err(error);
            }
        }
        Ok(())
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, ErrorCode>;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.chars.clone().next()?;
        let ret = match x {
            /* Common command prefix */
            b'*' => {
                self.in_common = true;
                Some(self.read_mnemonic(true))
            }
            /* Header mnemonic separator/prefix */
            b':' => {
                self.chars.next();
                //Only one separator is allowed
                if let Some(x) = self.chars.clone().next() {
                    if !x.is_ascii_alphabetic() {
                        return Some(Err(ErrorCode::InvalidSeparator));
                    }
                }
                /* Not allowed outside header and strings */
                if !self.in_header || self.in_common {
                    Some(Err(ErrorCode::InvalidSeparator))
                } else {
                    Some(Ok(Token::HeaderMnemonicSeparator))
                }
            }
            /* Header query suffix */
            b'?' => {
                self.chars.next();
                //Next character after query must be a space, unit separator or <END>
                if let Some(x) = self.chars.clone().next() {
                    if !x.is_ascii_whitespace() && *x != b';' {
                        return Some(Err(ErrorCode::SyntaxError));
                    }
                }
                if !self.in_header {
                    Some(Err(ErrorCode::SyntaxError))
                } else {
                    self.in_header = false;
                    Some(Ok(Token::HeaderQuerySuffix))
                }
            }
            /* Program unit separator */
            b';' => {
                self.chars.next();
                util::skip_ws(&mut self.chars);
                self.in_header = true;
                self.in_common = false;
                Some(Ok(Token::ProgramMessageUnitSeparator))
            }
            /* Message terminator */
            //END is implied.
            // Parser should reset itself and parse next message as a new message.
            b'\n' => {
                self.chars.next();
                if self.chars.next().is_none() {
                    None
                } else {
                    Some(Err(ErrorCode::SyntaxError))
                }
            }
            /* Data separator*/
            b',' => {
                self.chars.next();
                if self.in_header {
                    Some(Err(ErrorCode::HeaderSeparatorError))
                } else {
                    util::skip_ws(&mut self.chars);
                    if let Some(c) = self.chars.clone().next() {
                        if *c == b',' || *c == b';' || *c == b'\n' {
                            return Some(Err(ErrorCode::SyntaxError));
                        }
                    }
                    Some(Ok(Token::ProgramDataSeparator))
                }
            }
            /* Whitespace */
            // Separates the header from arguments
            x if x.is_ascii_whitespace() => {
                util::skip_ws(&mut self.chars);
                /* Header ends */
                self.in_header = false;
                Some(Ok(Token::ProgramHeaderSeparator))
            }
            /* Alphabetic */
            x if x.is_ascii_alphabetic() => {
                /* If still parsing header, it's an mnemonic, else character data */
                if self.in_header {
                    Some(self.read_mnemonic(false))
                } else {
                    Some(self.read_character_data())
                }
            }
            /* Number */
            x if x.is_ascii_digit() || *x == b'-' || *x == b'+' || *x == b'.' => {
                if self.in_header {
                    Some(Err(ErrorCode::CommandHeaderError))
                } else {
                    Some(self.read_numeric_data())
                }
            }
            /* Arb. block or non-decimal data */
            b'#' => {
                self.chars.next();
                if self.in_header {
                    Some(Err(ErrorCode::CommandHeaderError))
                } else if let Some(x) = self.chars.next() {
                    Some(match x {
                        /* Arbitrary block */
                        x if x.is_ascii_digit() => self.read_arbitrary_data(*x),
                        /*Non-decimal numeric*/
                        _ => self.read_nondecimal_data(*x),
                    })
                } else {
                    Some(Err(ErrorCode::BlockDataError))
                }
            }
            /* String */
            x if *x == b'\'' || *x == b'"' => {
                if self.in_header {
                    Some(Err(ErrorCode::CommandHeaderError))
                } else {
                    Some(self.read_string_data(*x, true))
                }
            }
            b'(' => Some(self.read_expression_data()),
            /* Unknown/unexpected */
            _ => {
                let x = self.chars.next().unwrap();
                if x.is_ascii() {
                    Some(Err(ErrorCode::SyntaxError))
                } else {
                    Some(Err(ErrorCode::InvalidCharacter))
                }
            }
        };
        //extern crate std;
        //std::dbg!(ret);
        ret
    }
}

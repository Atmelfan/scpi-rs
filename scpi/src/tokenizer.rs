use crate::error::{Error, ErrorCode};
use crate::format::{Arbitrary, Character, Expression};

use core::slice::Iter;
use core::str;

use core::convert::TryFrom;

use crate::expression::{channel_list, numeric_list};
use crate::{util, NumericValues};

/// SCPI tokens
/// Loosely based on IEEE488.2 Chapter 7
///
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    /// A header mnemonic separator `:`
    HeaderMnemonicSeparator,
    /// A common header prefix `*`
    HeaderCommonPrefix,
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

    /// Handle data as a SCPI <numeric> and convert to R datatype
    ///
    /// **Note: Decimal data is not compared to the maximum/minimum value and must be done
    /// seperately (unless equal to max/min of that datatype).
    ///
    /// # Example
    /// ```
    /// use scpi::tokenizer::Token;
    /// use scpi::NumericValues;
    /// use scpi::error::ErrorCode;
    /// let  s = Token::CharacterProgramData(b"MAXimum");
    ///
    /// let mut x = 128u8;
    /// if let Ok(v) = s.numeric(|special| match special {
    ///     NumericValues::Maximum => Ok(255u8),
    ///     NumericValues::Minimum => Ok(0u8),
    ///     NumericValues::Default => Ok(1u8),
    ///     NumericValues::Up => Ok(x+1),
    ///     NumericValues::Down => Ok(x-1),
    ///     _ => Err(ErrorCode::ParameterError.into())
    /// }) {
    ///     //v is resolved to a u8 type
    ///     x = v;
    /// }
    ///
    /// ```
    ///
    ///
    pub fn numeric<F, R: TryFrom<Token<'a>, Error = Error>>(self, special: F) -> Result<R, Error>
    where
        F: FnOnce(NumericValues) -> Result<R, Error>,
    {
        match self {
            Token::CharacterProgramData(s) => match s {
                x if util::mnemonic_compare(b"MAXimum", x) => special(NumericValues::Maximum),
                x if util::mnemonic_compare(b"MINimum", x) => special(NumericValues::Minimum),
                x if util::mnemonic_compare(b"DEFault", x) => special(NumericValues::Default),
                x if util::mnemonic_compare(b"UP", x) => special(NumericValues::Up),
                x if util::mnemonic_compare(b"DOWN", x) => special(NumericValues::Down),
                x if util::mnemonic_compare(b"AUTO", x) => special(NumericValues::Auto),
                _ => <R>::try_from(self),
            },
            _ => <R>::try_from(self),
        }
    }

    pub fn numeric_range<F, R: TryFrom<Token<'a>, Error = Error>>(
        &self,
        min: R,
        max: R,
        special: F,
    ) -> Result<R, Error>
    where
        F: FnOnce(NumericValues) -> Result<R, Error>,
        R: PartialOrd + Copy,
    {
        let value = self.numeric(|choice| match choice {
            NumericValues::Maximum => Ok(max),
            NumericValues::Minimum => Ok(min),
            x => special(x),
        })?;
        if value > max || value < min {
            Err(ErrorCode::DataOutOfRange.into())
        } else {
            Ok(value)
        }
    }
}

/// Convert string data data into a slice (&\[u8\]).
///
/// # Returns
/// * `Ok(&[u8])` - If data is a string.
/// * `Err(DataTypeError)` - If data is not a string.
/// * `Err(SyntaxError)` - If token is not data
impl<'a> TryFrom<Token<'a>> for &'a [u8] {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<&'a [u8], Self::Error> {
        match value {
            Token::StringProgramData(s) => Ok(s),
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

/// Convert string data data into a boolean.
///
/// # Returns
/// * `Ok(bool)` - If data is character data matching `ON|OFF` or numeric `1|0`.
/// * `Err(IllegalParameterValue)` - If data is character data or numeric but is not a boolean
/// * `Err(DataTypeError)` - If data is not a character data or numeric.
/// * `Err(SyntaxError)` - If token is not data.
impl<'a> TryFrom<Token<'a>> for bool {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<bool, Self::Error> {
        match value {
            Token::DecimalNumericProgramData(_) => {
                // Round numeric to integer, non-zero equals true
                Ok(<isize>::try_from(value)? != 0)
            }
            Token::CharacterProgramData(s) => {
                if s.eq_ignore_ascii_case(b"ON") {
                    Ok(true)
                } else if s.eq_ignore_ascii_case(b"OFF") {
                    Ok(false)
                } else {
                    Err(ErrorCode::IllegalParameterValue.into())
                }
            }
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

/// Convert string data data into a str.
///
/// # Returns
/// * `Ok(&str)` - If data is a string.
/// * `Err(DataTypeError)` - If data is not a string.
/// * `Err(SyntaxError)` - If token is not data
impl<'a> TryFrom<Token<'a>> for &'a str {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<&'a str, Self::Error> {
        match value {
            Token::StringProgramData(s) | Token::ArbitraryBlockData(s) => {
                str::from_utf8(s).map_err(|_| ErrorCode::StringDataError.into())
            }
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

/// Convert character data into a str.
///
/// # Returns
/// * `Ok(&str)` - If data is character data.
/// * `Err(DataTypeError)` - If data is not character string.
/// * `Err(SyntaxError)` - If token is not data
impl<'a> TryFrom<Token<'a>> for Arbitrary<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Arbitrary<'a>, Self::Error> {
        match value {
            Token::ArbitraryBlockData(s) => Ok(Arbitrary(s)),
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

/// Convert character data into a str.
///
/// # Returns
/// * `Ok(&str)` - If data is character data.
/// * `Err(DataTypeError)` - If data is not character string.
/// * `Err(SyntaxError)` - If token is not data
impl<'a> TryFrom<Token<'a>> for Character<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Character<'a>, Self::Error> {
        match value {
            Token::CharacterProgramData(s) => Ok(Character(s)),
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

impl<'a> TryFrom<Token<'a>> for numeric_list::NumericList<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<numeric_list::NumericList<'a>, Self::Error> {
        match value {
            Token::ExpressionProgramData(s) => Ok(numeric_list::NumericList::new(s)),
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

impl<'a> TryFrom<Token<'a>> for channel_list::ChannelList<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<channel_list::ChannelList<'a>, Self::Error> {
        match value {
            Token::ExpressionProgramData(s) => channel_list::ChannelList::new(s).ok_or_else(|| {
                Error::extended(ErrorCode::InvalidExpression, b"Invalid channel list")
            }),
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

impl<'a> TryFrom<Token<'a>> for Expression<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Expression<'a>, Self::Error> {
        match value {
            Token::ArbitraryBlockData(s) => Ok(Expression(s)),
            t => {
                if t.is_data() {
                    Err(ErrorCode::DataTypeError.into())
                } else {
                    Err(Error::extended(
                        ErrorCode::DeviceSpecificError,
                        b"Parser error",
                    ))
                }
            }
        }
    }
}

macro_rules! impl_tryfrom_float {
    ($from:ty) => {
        impl<'a> TryFrom<Token<'a>> for $from {
            type Error = Error;

            fn try_from(value: Token) -> Result<Self, Self::Error> {
                match value {
                    Token::DecimalNumericProgramData(value) => lexical_core::parse::<$from>(value)
                        .map_err(|e| match e.code {
                            lexical_core::Error::InvalidDigit(_) => {
                                ErrorCode::InvalidCharacterInNumber.into()
                            }
                            lexical_core::Error::Overflow(_)
                            | lexical_core::Error::Underflow(_) => {
                                ErrorCode::DataOutOfRange.into()
                            }
                            _ => ErrorCode::NumericDataError.into(),
                        }),
                    Token::CharacterProgramData(s) => match s {
                        //Check for special float values
                        ref x if util::mnemonic_compare(b"INFinity", x) => Ok(<$from>::INFINITY),
                        ref x if util::mnemonic_compare(b"NINFinity", x) => {
                            Ok(<$from>::NEG_INFINITY)
                        }
                        ref x if util::mnemonic_compare(b"NAN", x) => Ok(<$from>::NAN),
                        ref x if util::mnemonic_compare(b"MAXimum", x) => Ok(<$from>::MAX),
                        ref x if util::mnemonic_compare(b"MINimum", x) => Ok(<$from>::MIN),
                        _ => Err(ErrorCode::DataTypeError.into()),
                    },
                    Token::DecimalNumericSuffixProgramData(_, _) => {
                        Err(ErrorCode::SuffixNotAllowed.into())
                    }
                    t => {
                        if t.is_data() {
                            Err(ErrorCode::DataTypeError.into())
                        } else {
                            Err(Error::extended(
                                ErrorCode::DeviceSpecificError,
                                b"Parser error",
                            ))
                        }
                    }
                }
            }
        }
    };
}

impl_tryfrom_float!(f32);
impl_tryfrom_float!(f64);

// TODO: Shitty way of rounding integers
macro_rules! impl_tryfrom_integer {
    ($from:ty, $intermediate:tt) => {
        impl<'a> TryFrom<Token<'a>> for $from {
            type Error = Error;

            fn try_from(value: Token) -> Result<Self, Self::Error> {
                match value {
                    Token::DecimalNumericProgramData(value) => lexical_core::parse::<$from>(value)
                        .or_else(|e| {
                            if matches!(e.code, lexical_core::Error::InvalidDigit(_)) {
                                let nrf = lexical_core::parse::<$intermediate>(value)?;
                                let f = <$intermediate>::round(nrf);
                                if f > (<$from>::MAX as $intermediate) {
                                    Err(lexical_core::Error::Overflow(0).into())
                                } else if f < (<$from>::MIN as $intermediate) {
                                    Err(lexical_core::Error::Underflow(0).into())
                                } else {
                                    Ok(f as $from)
                                }
                            } else {
                                Err(e)
                            }
                        })
                        .map_err(|e| match e.code {
                            lexical_core::Error::InvalidDigit(_) => {
                                ErrorCode::InvalidCharacterInNumber.into()
                            }
                            lexical_core::Error::Overflow(_)
                            | lexical_core::Error::Underflow(_) => {
                                ErrorCode::DataOutOfRange.into()
                            }
                            _ => ErrorCode::NumericDataError.into(),
                        }),
                    Token::NonDecimalNumericProgramData(value) => {
                        <$from>::try_from(value).map_err(|_| ErrorCode::DataOutOfRange.into())
                    }
                    Token::CharacterProgramData(s) => match s {
                        //Check for special float values
                        ref x if util::mnemonic_compare(b"MAXimum", x) => Ok(<$from>::MAX),
                        ref x if util::mnemonic_compare(b"MINimum", x) => Ok(<$from>::MIN),
                        _ => Err(ErrorCode::DataTypeError.into()),
                    },
                    Token::DecimalNumericSuffixProgramData(_, _) => {
                        Err(ErrorCode::SuffixNotAllowed.into())
                    }
                    t => {
                        if t.is_data() {
                            Err(ErrorCode::DataTypeError.into())
                        } else {
                            Err(Error::extended(
                                ErrorCode::DeviceSpecificError,
                                b"Parser error",
                            ))
                        }
                    }
                }
            }
        }
    };
}

// Need to fallback to floating point if numeric is not NR1 formatted.
// Use double precision on larger types to avoid rounding errors.
impl_tryfrom_integer!(usize, f64);
impl_tryfrom_integer!(isize, f64);
impl_tryfrom_integer!(i64, f64);
impl_tryfrom_integer!(u64, f64);
impl_tryfrom_integer!(i32, f64);
impl_tryfrom_integer!(u32, f64);
impl_tryfrom_integer!(i16, f32);
impl_tryfrom_integer!(u16, f32);
impl_tryfrom_integer!(i8, f32);
impl_tryfrom_integer!(u8, f32);

#[derive(Clone)]
pub struct Tokenizer<'a> {
    pub(crate) chars: Iter<'a, u8>,
    in_header: bool,
    in_common: bool,
}

impl<'a> Tokenizer<'a> {
    /// Attempts to consume a data object.
    /// If no data is found, none if returned if optional=true else Error:MissingParam.
    ///
    /// Note! Does not skip
    pub fn next_data(&mut self, optional: bool) -> Result<Option<Token<'a>>, Error> {
        //Try to read a data object
        if let Some(item) = self.clone().next() {
            //Check if next item is a data object
            let token = item?;
            match token {
                //Data object
                t if t.is_data() => {
                    //Valid data object, consume and return
                    self.next();
                    Ok(Some(token))
                }
                //Data separator, next token must be a data object
                Token::ProgramDataSeparator => {
                    self.next();
                    self.next_data(false)
                }
                _ => {
                    //Not a data object, return nothing
                    if optional {
                        Ok(None)
                    } else {
                        Err(ErrorCode::MissingParameter.into())
                    }
                }
            }
        } else {
            //No more tokens, return nothing
            if optional {
                Ok(None)
            } else {
                Err(ErrorCode::MissingParameter.into())
            }
        }
    }
}

impl<'a> Tokenizer<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Tokenizer::from_byte_iter(buf.iter())
    }

    pub(crate) fn empty() -> Self {
        Tokenizer::from_byte_iter(b"".iter())
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
    fn read_mnemonic(&mut self) -> Result<Token<'a>, ErrorCode> {
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
        let radixi = match radix {
            b'H' | b'h' => 16u8,
            b'Q' | b'q' => 8u8,
            b'B' | b'b' => 2u8,
            _ => return Err(ErrorCode::NumericDataError),
        };
        let (n, len) = lexical_core::parse_partial_radix(self.chars.as_slice(), radixi).map_err(
            |e| match e.code {
                lexical_core::Error::InvalidDigit(_) => ErrorCode::InvalidCharacterInNumber,
                lexical_core::Error::Overflow(_) | lexical_core::Error::Underflow(_) => {
                    ErrorCode::DataOutOfRange
                }
                _ => ErrorCode::NumericDataError,
            },
        )?;
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

#[cfg(test)]
mod test_parse {
    use crate::error::ErrorCode;
    use crate::tokenizer::{Token, Tokenizer};
    use crate::util;

    extern crate std;

    #[test]
    fn test_split_mnemonic() {
        assert_eq!(
            util::mnemonic_split_index(b"TRIGger54"),
            Some((b"TRIGger".as_ref(), b"54".as_ref()))
        );
        assert_eq!(
            util::mnemonic_split_index(b"T123r54"),
            Some((b"T123r".as_ref(), b"54".as_ref()))
        );
        assert_eq!(util::mnemonic_split_index(b"TRIGger"), None);
        assert_eq!(util::mnemonic_split_index(b""), None);
    }

    #[test]
    fn test_compare_mnemonic() {
        //Should return true
        assert!(util::mnemonic_compare(b"TRIGger", b"trigger"));
        assert!(util::mnemonic_compare(b"TRIGger", b"trig"));
        assert!(util::mnemonic_compare(b"TRIGger", b"TRIGGER"));
        assert!(util::mnemonic_compare(b"TRIGger", b"TRIG"));
        //Should return false
        assert!(!util::mnemonic_compare(b"TRIGger", b"TRIGge"));
        assert!(!util::mnemonic_compare(b"TRIGger", b"triggeristoodamnlong"));
        assert!(!util::mnemonic_compare(b"TRIGger", b"tri"));
    }

    #[test]
    fn test_eq_mnemonic() {
        //
        assert!(Token::ProgramMnemonic(b"trigger").match_program_header(b"TRIGger1"));
        assert!(Token::ProgramMnemonic(b"trig").match_program_header(b"TRIGger1"));
        assert!(Token::ProgramMnemonic(b"trigger1").match_program_header(b"TRIGger1"));
        assert!(Token::ProgramMnemonic(b"trigger1").match_program_header(b"TRIGger"));
        assert!(Token::ProgramMnemonic(b"trig1").match_program_header(b"TRIGger1"));
        assert!(!Token::ProgramMnemonic(b"trigger2").match_program_header(b"TRIGger1"));
        assert!(!Token::ProgramMnemonic(b"trig2").match_program_header(b"TRIGger1"));
        assert!(!Token::ProgramMnemonic(b"trig2").match_program_header(b"TRIGger1"));

        assert!(Token::ProgramMnemonic(b"trigger2").match_program_header(b"TRIGger2"));
        assert!(Token::ProgramMnemonic(b"trig2").match_program_header(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trigger").match_program_header(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trig").match_program_header(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trigger1").match_program_header(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trig1").match_program_header(b"TRIGger2"));
    }

    #[test]
    fn test_read_character_data() {
        assert_eq!(
            Tokenizer::new(b"CHARacter4 , pperg").read_character_data(),
            Ok(Token::CharacterProgramData(b"CHARacter4"))
        );
        assert_eq!(
            Tokenizer::new(b"CHARacterIsTooLong").read_character_data(),
            Err(ErrorCode::CharacterDataTooLong)
        );
        assert_eq!(
            Tokenizer::new(b"Character Invalid").read_character_data(),
            Err(ErrorCode::InvalidCharacterData)
        );
    }

    #[test]
    fn test_read_numeric_data() {
        //TODO: FIX EXPONENTS!

        assert_eq!(
            Tokenizer::new(b"25").read_numeric_data().unwrap(),
            Token::DecimalNumericProgramData(b"25")
        );

        assert_eq!(
            Tokenizer::new(b".2").read_numeric_data().unwrap(),
            Token::DecimalNumericProgramData(b".2")
        );

        assert_eq!(
            Tokenizer::new(b"1.0E5").read_numeric_data().unwrap(),
            Token::DecimalNumericProgramData(b"1.0E5")
        );

        assert_eq!(
            Tokenizer::new(b"-25e5").read_numeric_data().unwrap(),
            Token::DecimalNumericProgramData(b"-25e5")
        );

        assert_eq!(
            Tokenizer::new(b"25E-2").read_numeric_data().unwrap(),
            Token::DecimalNumericProgramData(b"25E-2")
        );

        assert_eq!(
            Tokenizer::new(b".1E2").read_numeric_data().unwrap(),
            Token::DecimalNumericProgramData(b".1E2")
        );

        assert_eq!(
            Tokenizer::new(b".1E2  SUFFIX").read_numeric_data().unwrap(),
            Token::DecimalNumericSuffixProgramData(b".1E2", b"SUFFIX")
        );

        assert_eq!(
            Tokenizer::new(b".1E2  /S").read_numeric_data().unwrap(),
            Token::DecimalNumericSuffixProgramData(b".1E2", b"/S")
        );

        assert_eq!(
            Tokenizer::new(b".1E2  'SUFFIX'")
                .read_numeric_data()
                .unwrap_err(),
            ErrorCode::InvalidSuffix
        );
    }

    #[test]
    fn test_read_suffix_data() {}

    #[test]
    fn test_read_numeric_suffix_data() {
        let mut tokenizer = Tokenizer::new(b"header 25 KHZ , 12.7E6 KOHM.M/S-2");
        assert_eq!(
            tokenizer.next(),
            Some(Ok(Token::ProgramMnemonic(b"header")))
        );
        assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramHeaderSeparator)));
        assert_eq!(
            tokenizer.next(),
            Some(Ok(Token::DecimalNumericSuffixProgramData(b"25", b"KHZ")))
        );
        assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramDataSeparator)));
        assert_eq!(
            tokenizer.next(),
            Some(Ok(Token::DecimalNumericSuffixProgramData(
                b"12.7E6",
                b"KOHM.M/S-2"
            )))
        );
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_read_string_data() {
        assert_eq!(
            Tokenizer::new(b"\"MOHM\",  gui").read_string_data(b'"', true),
            Ok(Token::StringProgramData(b"MOHM"))
        );
        assert_eq!(
            Tokenizer::new(b"'MOHM',  gui").read_string_data(b'\'', true),
            Ok(Token::StringProgramData(b"MOHM"))
        );
        assert_eq!(
            Tokenizer::new(b"'MO''HM',  gui").read_string_data(b'\'', true),
            Ok(Token::StringProgramData(b"MO''HM"))
        );

        assert_eq!(
            Tokenizer::new(b"\"MOHM").read_string_data(b'"', true),
            Err(ErrorCode::InvalidStringData)
        );
        assert_eq!(
            Tokenizer::new(b"'MOHM").read_string_data(b'"', true),
            Err(ErrorCode::InvalidStringData)
        );
        assert_eq!(
            Tokenizer::new(b"'MO\xffHM").read_string_data(b'"', true),
            Err(ErrorCode::InvalidCharacter)
        );
    }

    #[test]
    fn test_read_arb_data() {
        assert_eq!(
            Tokenizer::new(b"02\x01\x02,").read_arbitrary_data(b'2'),
            Ok(Token::ArbitraryBlockData(&[1, 2]))
        );
        assert_eq!(
            Tokenizer::new(b"2\x01\x02,").read_arbitrary_data(b'1'),
            Ok(Token::ArbitraryBlockData(&[1, 2]))
        );

        // Error, too short
        assert_eq!(
            Tokenizer::new(b"02\x01").read_arbitrary_data(b'2'),
            Err(ErrorCode::InvalidBlockData)
        );

        // Error, invalid header
        assert_eq!(
            Tokenizer::new(b"a2\x01\x02,").read_arbitrary_data(b'2'),
            Err(ErrorCode::InvalidBlockData)
        );

        // Error, header too short/invalid
        assert_eq!(
            Tokenizer::new(b"2\x01\x02,").read_arbitrary_data(b'2'),
            Err(ErrorCode::InvalidBlockData)
        );

        // Indefinite length
        assert_eq!(
            Tokenizer::new(b"\x01\x02\n").read_arbitrary_data(b'0'),
            Ok(Token::ArbitraryBlockData(&[1, 2]))
        );

        // Error, indefinite not terminated by newline
        assert_eq!(
            Tokenizer::new(b"\x01\x02").read_arbitrary_data(b'0'),
            Err(ErrorCode::InvalidBlockData)
        );
    }

    #[test]
    fn test_read_expr_data() {
        assert_eq!(
            Tokenizer::new(b"(@1!2,2,3,4,5,#,POTATO)").read_expression_data(),
            Ok(Token::ExpressionProgramData(b"@1!2,2,3,4,5,#,POTATO"))
        );
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, ErrorCode>;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.chars.clone().next()?;
        match x {
            /* Common command prefix */
            b'*' => {
                self.in_common = true;
                self.chars.next();
                if let Some(x) = self.chars.clone().next() {
                    if !x.is_ascii_alphabetic() {
                        return Some(Err(ErrorCode::CommandHeaderError));
                    }
                }

                /* Not allowed outside header and strings */
                Some(Ok(Token::HeaderCommonPrefix))
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
                        return Some(Err(ErrorCode::InvalidSeparator));
                    }
                }
                if !self.in_header {
                    Some(Err(ErrorCode::InvalidSeparator))
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
                    Some(self.read_mnemonic())
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
        }
    }
}

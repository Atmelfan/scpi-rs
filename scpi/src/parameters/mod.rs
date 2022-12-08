mod numeric;
pub use numeric::NumericValue;

use core::iter::Peekable;
use core::str;

use crate::prelude::*;
use crate::util;

/// Alias
pub struct Arguments<'a, 'b>(&'a mut Peekable<Tokenizer<'b>>);

impl<'a, 'b> Arguments<'a, 'b> {
    pub fn with(toka: &'a mut Peekable<Tokenizer<'b>>) -> Self {
        Self(toka)
    }
}

impl<'a, 'b> Arguments<'a, 'b> {
    /// Attempts to consume a data object.
    /// If no data is found, none if returned if optional=true else Error:MissingParam.
    ///
    /// Note! Does not skip
    pub fn next_optional_token(&mut self) -> Result<Option<Token<'a>>, Error> {
        //Try to read a data object

        if let Some(item) = self.0.peek() {
            //Check if next item is a data object
            let token = item.clone()?;
            match token {
                //Data object
                t if t.is_data() => {
                    //Valid data object, consume and return
                    self.0.next();
                    Ok(Some(token))
                }
                //Data separator, next token must be a data object
                Token::ProgramDataSeparator => {
                    self.0.next();
                    self.next_token().map(|v| Some(v))
                }
                // Something else
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, Error> {
        match self.next_optional_token() {
            Ok(Some(tok)) => Ok(tok),
            Ok(None) => Err(ErrorCode::MissingParameter.into()),
            Err(err) => Err(err),
        }
    }

    pub fn next<T>(&mut self) -> Result<T, Error>
    where
        T: TryFrom<Token<'a>, Error = Error>,
    {
        self.next_token()?.try_into()
    }

    pub fn next_optional<T>(&mut self) -> Result<Option<T>, Error>
    where
        T: TryFrom<Token<'a>, Error = Error>,
    {
        let tok = self.next_optional_token()?;
        match tok {
            Some(tok) => Ok(Some(tok.try_into()?)),
            None => Ok(None),
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

/// Convert string/block data data into a str.
///
/// # Returns
/// * `Ok(&str)` - If data is a string or block data.
/// * `Err(DataTypeError)` - If data is not a string.
/// * `Err(StringDataError)` - If string is not valid utf8
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
impl<'a> TryFrom<Token<'a>> for format::Arbitrary<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<format::Arbitrary<'a>, Self::Error> {
        match value {
            Token::ArbitraryBlockData(s) => Ok(format::Arbitrary(s)),
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
impl<'a> TryFrom<Token<'a>> for format::Character<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<format::Character<'a>, Self::Error> {
        match value {
            Token::CharacterProgramData(s) => Ok(format::Character(s)),
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

impl<'a> TryFrom<Token<'a>> for format::Expression<'a> {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<format::Expression<'a>, Self::Error> {
        match value {
            Token::ArbitraryBlockData(s) => Ok(format::Expression(s)),
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
                        .map_err(|e| match e {
                            lexical_core::Error::InvalidDigit(_) => {
                                ErrorCode::InvalidCharacterInNumber.into()
                            }
                            lexical_core::Error::Overflow(_)
                            | lexical_core::Error::Underflow(_) => ErrorCode::DataOutOfRange.into(),
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
    ($from:ty, $intermediate:ty) => {
        impl<'a> TryFrom<Token<'a>> for $from {
            type Error = Error;

            fn try_from(value: Token) -> Result<Self, Self::Error> {
                match value {
                    Token::DecimalNumericProgramData(value) => lexical_core::parse::<$from>(value)
                        .or_else(|e| {
                            if matches!(e, lexical_core::Error::InvalidDigit(_)) {
                                let value = lexical_core::parse::<$intermediate>(value)?;

                                if !value.is_normal() {
                                    Err(lexical_core::Error::Overflow(0).into())
                                } else if value > (<$from>::MAX as $intermediate) {
                                    Err(lexical_core::Error::Overflow(0).into())
                                } else if value < (<$from>::MIN as $intermediate) {
                                    Err(lexical_core::Error::Underflow(0).into())
                                } else {
                                    // <f32|f64>::round() doesn't exist in no_std...
                                    // Safe because value is checked to be normal and within bounds earlier
                                    if value.is_sign_positive() {
                                        Ok(unsafe { (value + 0.5).to_int_unchecked() })
                                    } else {
                                        Ok(unsafe { (value - 0.5).to_int_unchecked() })
                                    }
                                }
                            } else {
                                Err(e)
                            }
                        })
                        .map_err(|e| match e {
                            lexical_core::Error::InvalidDigit(_) => {
                                ErrorCode::InvalidCharacterInNumber.into()
                            }
                            lexical_core::Error::Overflow(_)
                            | lexical_core::Error::Underflow(_) => ErrorCode::DataOutOfRange.into(),
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

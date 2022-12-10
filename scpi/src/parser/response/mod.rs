use super::format::{Arbitrary, Binary, Character, Expression, Hex, Octal};
use crate::error::{Error, ErrorCode, Result};

mod arrayformatter;
#[cfg(feature = "alloc")]
mod vecformatter;

use lexical_core::FormattedSize;
use lexical_core::NumberFormatBuilder;

const RESPONSE_DATA_SEPARATOR: u8 = b',';
const RESPONSE_HEADER_SEPARATOR: u8 = b' ';
const RESPONSE_MESSAGE_UNIT_SEPARATOR: u8 = b';';
const RESPONSE_MESSAGE_TERMINATOR: u8 = b'\n';

pub trait ResponseData {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()>;
}

macro_rules! impl_non_decimal_data {
    ($prefix:literal, $name:ident, $radix:literal; $typ:ty) => {
        impl ResponseData for $name<$typ> {
            fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
                let mut buf = [b'0'; <$typ>::FORMATTED_SIZE];
                const FORMAT: u128 = NumberFormatBuilder::from_radix($radix);
                let options = lexical_core::WriteIntegerOptions::new();
                let slc = lexical_core::write_with_options::<_, FORMAT>(self.0, &mut buf, &options);
                formatter.push_str($prefix)?;
                formatter.push_str(slc)
            }
        }
    };
}

// Create decimal/non-decimal formatters for integers
macro_rules! impl_integer {
    ($typ:ty) => {
        impl ResponseData for $typ {
            fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
                let mut buf = [b'0'; <$typ>::FORMATTED_SIZE_DECIMAL];
                let slc = lexical_core::write::<$typ>(*self, &mut buf);
                formatter.push_str(slc)
            }
        }

        impl_non_decimal_data!(b"#H", Hex, 16; $typ);
        impl_non_decimal_data!(b"#Q", Octal, 8; $typ);
        impl_non_decimal_data!(b"#B", Binary, 2; $typ);
    };
}

impl_integer!(u8);
impl_integer!(i8);
impl_integer!(u16);
impl_integer!(i16);
impl_integer!(u32);
impl_integer!(i32);
impl_integer!(u64);
impl_integer!(i64);
impl_integer!(isize);
impl_integer!(usize);

// Create formatters for floating point
macro_rules! impl_real {
    ($typ:ty) => {
        impl ResponseData for $typ {
            fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
                if self.is_nan() {
                    // NaN is represented by 9.91E+37
                    formatter.push_str(b"9.91E+37")
                } else if self.is_infinite() {
                    // +/- Infinity is represented by +/-9.9E+37
                    if self.is_sign_negative() {
                        formatter.push_str(b"-9.9E+37")
                    } else {
                        formatter.push_str(b"9.9E+37")
                    }
                } else {
                    let mut buf = [b'0'; <$typ>::FORMATTED_SIZE_DECIMAL];
                    let slc = lexical_core::write::<$typ>(*self, &mut buf);
                    formatter.push_str(slc)
                }
            }
        }
    };
}

impl_real!(f32);
impl_real!(f64);

impl ResponseData for bool {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        if *self {
            formatter.push_byte(b'1')
        } else {
            formatter.push_byte(b'0')
        }
    }
}

impl<'a> ResponseData for Arbitrary<'a> {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        let mut buf = [0u8; usize::FORMATTED_SIZE_DECIMAL];
        let slc = lexical_core::write::<usize>(self.0.len(), &mut buf);
        if slc.len() > 9 {
            Err(ErrorCode::ExecutionError.into())
        } else {
            formatter.push_byte(b'#')?;
            slc.len().format_response_data(formatter)?;
            formatter.push_str(slc)?;
            formatter.push_str(self.0)
        }
    }
}

impl<'a> ResponseData for Character<'a> {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        formatter.push_ascii(self.0)
    }
}

impl<'a> ResponseData for Expression<'a> {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        formatter.push_byte(b'(')?;
        formatter.push_ascii(self.0)?;
        formatter.push_byte(b')')
    }
}

impl<'a> ResponseData for &'a str {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        Arbitrary(self.as_bytes()).format_response_data(formatter)
    }
}

impl<'a> ResponseData for &'a [u8] {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        if !self.is_ascii() {
            Err(ErrorCode::ExecutionError.into())
        } else {
            let mut first = true;
            formatter.push_byte(b'"')?;
            for ss in self.split(|x| *x == b'"') {
                if !first {
                    formatter.push_str(br#""""#)?;
                }
                formatter.push_ascii(ss)?;
                first = false;
            }
            formatter.push_byte(b'"')
        }
    }
}

impl ResponseData for Error {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        self.get_code().format_response_data(formatter)?;
        formatter.data_separator()?;

        if let Some(ext) = self.get_extended() {
            formatter.push_byte(b'"')?;
            formatter.push_str(self.get_message())?;
            formatter.push_byte(b';')?;
            formatter.push_str(ext)?;
            formatter.push_byte(b'"')
        } else {
            self.get_message().format_response_data(formatter)
        }
    }
}

impl<T> ResponseData for T
where
    T: crate::option::ScpiEnum,
{
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        let mnemonic = self.mnemonic();
        let short_form = mnemonic.split(|c| !c.is_ascii_uppercase()).next().unwrap();
        formatter.push_str(short_form)
    }
}

/// Formats a SCPI response
///
pub trait Formatter {
    /* I/O */

    /// Push raw string to output
    fn push_str(&mut self, s: &[u8]) -> Result<()>;

    /// Push ascii to output, panics if
    fn push_ascii(&mut self, s: &[u8]) -> Result<()> {
        debug_assert!(s.is_ascii());
        self.push_str(s)
    }

    ///Push single byte to output
    fn push_byte(&mut self, b: u8) -> Result<()>;

    /// Get underlying buffer as a byte slice
    fn as_slice(&self) -> &[u8];

    /// Clear buffer
    fn clear(&mut self);

    /// Returns length of buffer
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /* Control */

    /// Start a response message
    fn message_start(&mut self) -> Result<()>;

    /// End a response message
    fn message_end(&mut self) -> Result<()>;

    /* Formatters */

    /// Insert a data separator
    fn data_separator(&mut self) -> Result<()> {
        self.push_byte(RESPONSE_DATA_SEPARATOR)
    }

    /// Insert a data separator
    fn header_separator(&mut self) -> Result<()> {
        self.push_byte(RESPONSE_HEADER_SEPARATOR)
    }

    fn response_unit(&mut self) -> Result<ResponseUnit>;
}

pub struct ResponseUnit<'a> {
    fmt: &'a mut dyn Formatter,
    result: Result<()>,
    has_header: bool,
    has_data: bool,
}

impl<'a> ResponseUnit<'a> {
    pub fn header(&mut self, header: &[u8]) -> &mut Self {
        debug_assert!(!self.has_data, "Tried to put header after data");
        self.result = self.result.and_then(|_| {
            if self.has_header {
                self.fmt.push_byte(b':')?;
            }
            self.fmt.push_str(header)
        });
        self.has_header = true;
        self
    }

    pub fn data<U>(&mut self, data: U) -> &mut Self
    where
        U: ResponseData,
    {
        self.result = self.result.and_then(|_| {
            if self.has_data {
                self.fmt.data_separator()?;
            } else if self.has_header {
                self.fmt.header_separator()?;
            }
            data.format_response_data(self.fmt)
        });
        self.has_data = true;
        self
    }

    pub fn finish(&mut self) -> Result<()> {
        self.result
    }
}

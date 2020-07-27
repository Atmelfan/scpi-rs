use crate::error::{Error, ErrorCode, Result};
use crate::tokenizer::{Arbitrary, Character};
use arrayvec::{Array, ArrayVec};
use lexical_core::Number;

///
///
/// response.start_message();
/// {
///     response.start_unit()?
///     response.push_str("GPA-Robotics")
///     response.push_str("T800 Model 101")
///     response.push_str("0")
///     response.push_str("0")
/// },
/// {
///     response.start_unit()?
///     1i32.scpi_fmt(response)
///     b"string".scpi_fmt(response)
/// }
/// response.stop_message()
///
///

macro_rules! push_x {
    ($name:ident, $primitive:ty) => {
        #[doc = "Format as a \\<NR1 NUMERIC RESPONSE DATA\\>"]
        fn $name(&mut self, x: $primitive) -> Result<()> {
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE_DECIMAL];
            let slc = lexical_core::write::<$primitive>(x, &mut buf);
            self.push_str(slc)
        }
    };
}
macro_rules! push_x_radix {
    ($name:ident, $primitive:ty, 16) => {
        #[doc = "Format as a \\<HEXADECIMAL NUMERIC RESPONSE DATA\\>"]
        fn $name(&mut self, x: $primitive) -> Result<()> {
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE];
            let slc = lexical_core::write_radix::<$primitive>(x, 16, &mut buf);
            self.push_str(b"#H")?;
            self.push_str(slc)
        }
    };
    ($name:ident, $primitive:ty, 8) => {
        #[doc = "Format as a \\<OCTAL NUMERIC RESPONSE DATA\\>"]
        fn $name(&mut self, x: $primitive) -> Result<()> {
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE];
            let slc = lexical_core::write_radix::<$primitive>(x, 8, &mut buf);
            self.push_str(b"#Q")?;
            self.push_str(slc)
        }
    };
    ($name:ident, $primitive:ty, 2) => {
        #[doc = "Format as a \\<BINARY NUMERIC RESPONSE DATA\\>"]
        fn $name(&mut self, x: $primitive) -> Result<()> {
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE];
            let slc = lexical_core::write_radix::<$primitive>(x, 2, &mut buf);
            self.push_str(b"#B")?;
            self.push_str(slc)
        }
    };
}

/// Formats a SCPI response
///
///
///```
/// use scpi::response::ArrayVecFormatter;
/// let mut array = ArrayVecFormatter::<[u8; 128]>::new();
///
///
///```
///

const RESPONSE_DATA_SEPARATOR: u8 = b',';
const RESPONSE_MESSAGE_UNIT_SEPARATOR: u8 = b';';
const RESPONSE_MESSAGE_TERMINATOR: u8 = b'\n';

pub trait Formatter {
    /* I/O */

    /// Push raw string to output
    fn push_str(&mut self, s: &[u8]) -> Result<()>;

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

    /// Start a unit
    fn unit_start(&mut self) -> Result<()>;

    /// End a unit
    fn unit_end(&mut self) -> Result<()>;

    /// End a response message
    fn message_end(&mut self) -> Result<()>;

    /* Formatters */

    /// Insert a data separator
    fn separator(&mut self) -> Result<()> {
        self.push_byte(RESPONSE_DATA_SEPARATOR)
    }

    /// Format `err` as a error/event response
    /// `<error code>, "<error message>"`
    #[cfg(not(feature = "extended-error"))]
    fn error(&mut self, err: Error) -> Result<()> {
        self.i16_data(err.get_code())?;
        self.separator()?;
        self.str_data(err.get_message())
    }

    /// Format `err` as a error/event response
    /// `<error code>, "<error message>"`
    #[cfg(feature = "extended-error")]
    fn error(&mut self, err: Error) -> Result<()> {
        self.i16_data(err.get_code())?;
        self.separator()?;
        if let Some(ext) = err.get_extended() {
            self.push_byte(b'"')?;
            self.push_str(err.get_message())?;
            self.push_byte(b';')?;
            self.push_str(ext)?;
            self.push_byte(b'"')
        } else {
            self.str_data(err.get_message())
        }
    }

    /// Formats `s` as \<ARBITRARY ASCII DATA\>
    /// TODO: Verify ASCII data
    fn ascii_data(&mut self, s: &[u8]) -> Result<()> {
        //        if !s.is_ascii() {
        //            panic!("")
        //        }
        self.push_str(s)
    }

    /// Formats `s` as \<STRING RESPONSE DATA\>
    /// TODO: Escape any double quotes
    fn str_data(&mut self, s: &[u8]) -> Result<()> {
        self.push_byte(b'"')?;
        self.push_str(s)?;
        self.push_byte(b'"')
    }

    /// Formats `s` as \<HEADER RESPONSE DATA\>
    fn header_data(&mut self, s: &[u8]) -> Result<()> {
        self.push_str(s)?;
        self.push_byte(b' ')
    }

    /// Formats `arb` as \<ARBITRARY BLOCK RESPONSE DATA\>
    fn arb_data(&mut self, arb: Arbitrary) -> Result<()> {
        self.push_byte(b'#')?;
        let mut buf = [0u8; usize::FORMATTED_SIZE_DECIMAL];
        let slc = lexical_core::write::<usize>(arb.0.len(), &mut buf);
        self.usize_data(slc.len())?;
        self.push_str(slc)?;
        self.push_str(arb.0)
    }

    /// Formats `chr` as \<CHARACTER RESPONSE DATA\>
    fn character_data(&mut self, chr: Character) -> Result<()> {
        self.push_str(chr.0)
    }

    /// Format and push a f32 with as \<NR3 NUMERIC RESPONSE DATA\>
    ///
    /// Special values:
    /// * NaN - Should be formatted as "9.91E+37"
    /// * +/-Infinity - Should be formatted as "(-)9.9E+37"
    ///
    fn f32_data(&mut self, value: f32) -> Result<()> {
        if value.is_nan() {
            // NaN is represented by 9.91E+37
            self.push_str(b"9.91E+37")
        } else if value.is_infinite() {
            // +/- Infinity is represented by +/-9.9E+37
            if value.is_sign_negative() {
                self.push_str(b"-9.9E+37")
            } else {
                self.push_str(b"9.9E+37")
            }
        } else {
            let mut buf = [b'0'; f32::FORMATTED_SIZE_DECIMAL];
            let slc = lexical_core::write::<f32>(value, &mut buf);
            self.push_str(slc)
        }
    }

    // Implement the basic types

    /* i8 */
    push_x!(i8_data, i8);
    /* u8 */
    push_x!(u8_data, u8);
    push_x_radix!(u8_hex_data, u8, 16);
    push_x_radix!(u8_oct_data, u8, 8);
    push_x_radix!(u8_bin_data, u8, 2);
    /* i16 */
    push_x!(i16_data, i16);
    /* u16 */
    push_x!(u16_data, u16);
    push_x_radix!(u16_hex_data, u16, 16);
    push_x_radix!(u16_oct_data, u16, 8);
    push_x_radix!(u16_bin_data, u16, 2);
    /* i32 */
    push_x!(i32_data, i32);
    /* u32 */
    push_x!(u32_data, u32);
    push_x_radix!(u32_hex_data, u32, 16);
    push_x_radix!(u32_oct_data, u32, 8);
    push_x_radix!(u32_bin_data, u32, 2);
    /* i64 */
    push_x!(i64_data, i32);
    /* u64 */
    push_x!(u64_data, u32);
    push_x_radix!(u64_hex_data, u64, 16);
    push_x_radix!(u64_oct_data, u64, 8);
    push_x_radix!(u64_bin_data, u64, 2);
    /* i16 */
    push_x!(isize_data, isize);
    /* u16 */
    push_x!(usize_data, usize);
    push_x_radix!(usize_hex_data, usize, 16);
    push_x_radix!(usize_oct_data, usize, 8);
    push_x_radix!(usize_bin_data, usize, 2);
}

pub struct ArrayVecFormatter<T: Array<Item = u8>> {
    vec: ArrayVec<T>,
    index: u8,
}

impl<T: Array<Item = u8>> Default for ArrayVecFormatter<T> {
    fn default() -> Self {
        ArrayVecFormatter {
            vec: ArrayVec::<T>::new(),
            index: 0,
        }
    }
}

impl<T: Array<Item = u8>> ArrayVecFormatter<T> {
    pub fn new() -> Self {
        ArrayVecFormatter::default()
    }
}

impl<T: Array<Item = u8>> Formatter for ArrayVecFormatter<T> {
    /// Internal use
    fn push_str(&mut self, s: &[u8]) -> Result<()> {
        self.vec
            .try_extend_from_slice(s)
            .map_err(|_| ErrorCode::OutOfMemory.into())
    }

    fn push_byte(&mut self, b: u8) -> Result<()> {
        self.vec
            .try_push(b)
            .map_err(|_| ErrorCode::OutOfMemory.into())
    }

    fn as_slice(&self) -> &[u8] {
        self.vec.as_slice()
    }

    fn clear(&mut self) {
        self.vec.clear();
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn message_start(&mut self) -> Result<()> {
        self.index = 0;
        Ok(())
    }

    fn unit_start(&mut self) -> Result<()> {
        self.index += 1;
        //Add a unit separator if not first unit
        if self.index > 1 {
            self.push_byte(RESPONSE_MESSAGE_UNIT_SEPARATOR)?;
        }
        Ok(())
    }

    fn unit_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn message_end(&mut self) -> Result<()> {
        self.push_byte(RESPONSE_MESSAGE_TERMINATOR)
    }
}

#[test]
fn test_vecarray() {
    let mut array = ArrayVecFormatter::<[u8; 16]>::new();
    array.unit_start().unwrap();
    array.str_data(b"potato").unwrap();
    array.separator().unwrap();
    array.u8_data(0).unwrap();
    array.unit_end().unwrap();
    assert_eq!(array.as_slice(), b"\"potato\",0");
}

#[test]
fn test_outamemory() {
    let mut array = ArrayVecFormatter::<[u8; 1]>::new();
    array.push_byte(b'x').unwrap();
    assert_eq!(
        array.push_byte(b'x'),
        Err(Error::from(ErrorCode::OutOfMemory))
    );
    assert_eq!(
        array.push_str(b"x"),
        Err(Error::from(ErrorCode::OutOfMemory))
    );
}

#[test]
fn test_f32() {
    let mut array = ArrayVecFormatter::<[u8; 32]>::new();
    array.f32_data(f32::INFINITY).unwrap();
    array.separator().unwrap();
    array.f32_data(f32::NEG_INFINITY).unwrap();
    array.separator().unwrap();
    array.f32_data(f32::NAN).unwrap();
    // See SCPI-99 7.2.1.4 and 7.2.1.5
    assert_eq!(array.as_slice(), b"9.9E+37,-9.9E+37,9.91E+37");
}

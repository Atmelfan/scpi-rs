use arrayvec::{ArrayVec, Array};
use lexical_core;
use crate::error::Error;
use lexical_core::{FromLexical, Number, ToLexical};


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
        #[doc="Format as a \\<NR1 NUMERIC RESPONSE DATA\\>"]
        pub fn $name (&mut self, x: $primitive) -> Result<&mut Self, Error>{
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE_DECIMAL];
            let slc = lexical_core::write::<$primitive>(x, &mut buf);
            self.push_str(slc)
        }
    };
}
macro_rules! push_x_radix {
    ($name:ident, $primitive:ty, 16) => {
        #[doc="Format as a \\<HEXADECIMAL NUMERIC RESPONSE DATA\\>"]
        pub fn $name (&mut self, x: $primitive) -> Result<&mut Self, Error>{
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE];
            let slc = lexical_core::write_radix::<$primitive>(x, 16, &mut buf);
            self.push_str(b"#H")?;
            self.push_str(slc)
        }
    };
    ($name:ident, $primitive:ty, 8) => {
        #[doc="Format as a \\<OCTAL NUMERIC RESPONSE DATA\\>"]
        pub fn $name (&mut self, x: $primitive) -> Result<&mut Self, Error>{
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE];
            let slc = lexical_core::write_radix::<$primitive>(x, 8, &mut buf);
            self.push_str(b"#Q")?;
            self.push_str(slc)
        }
    };
    ($name:ident, $primitive:ty, 2) => {
        #[doc="Format as a \\<BINARY NUMERIC RESPONSE DATA\\>"]
        pub fn $name (&mut self, x: $primitive) -> Result<&mut Self, Error>{
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
const RESPONSE_HEADER_SEPARATOR: u8 = b' ';
const RESPONSE_MESSAGE_UNIT_SEPARATOR: u8 = b';';
const RESPONSE_MESSAGE_TERMINATOR: u8 = b'\n';



pub struct ArrayVecFormatter<T: Array<Item=u8>> {
    vec: ArrayVec<T>,
    index: u8
}


impl<T: Array<Item=u8>> ArrayVecFormatter<T> {
    pub fn new() -> Self {
        ArrayVecFormatter {
            vec: ArrayVec::<T>::new(),
            index: 0
        }
    }



    /// Internal use
    fn push_str(&mut self, s: &[u8]) -> Result<&mut Self, Error> {
        self.vec.try_extend_from_slice(s).map_err(|_| Error::OutOfMemory).map(|_| self)
    }

    fn push_byte(&mut self, b: u8) -> Result<&mut Self, Error> {
        self.vec.try_push(b).map_err(|_| Error::OutOfMemory).map(|_| self)
    }



    /// Start a response message
    pub(crate) fn message_start(&mut self) -> Result<&mut Self, Error>{
        self.index = 0;
        Ok(self)
    }

    /// Start a unit
    ///
    /// This will insert a \<RESPONSE MESSAGE UNIT SEPARATOR\> between units
    pub(crate) fn unit_start(&mut self) -> Result<&mut Self, Error>{
        self.index += 1;
        //Add a unit separator if not first unit
        if self.index > 1 {
            self.push_byte(RESPONSE_MESSAGE_UNIT_SEPARATOR)?;
        }
        Ok(self)
    }

    /// Insert a data separator
    pub fn separator(&mut self) -> Result<&mut Self, Error>{
        self.push_byte(RESPONSE_DATA_SEPARATOR)
    }

    /// End a unit
    pub(crate) fn unit_end(&mut self) -> Result<&mut Self, Error>{
        Ok(self)
    }

    /// End a response message
    pub(crate) fn message_end(&mut self) -> Result<&mut Self, Error>{
        self.push_byte(RESPONSE_MESSAGE_TERMINATOR)
    }

    /// Formats `s` as \<STRING RESPONSE DATA\>
    /// TODO: Escape any double quotes
    pub fn str_data(&mut self, s: &[u8]) -> Result<&mut Self, Error>{
        self.push_byte(b'"')?.push_str(s)?.push_byte(b'"')
    }

    /// Format and push a f32 with as \<NR3 NUMERIC RESPONSE DATA\>
    ///
    /// Special values:
    /// * NaN - Should be formatted as "9.91E+37"
    /// * +/-Infinity - Should be formatted as "(-)9.9E+37"
    ///
    pub fn f32_data(&mut self, value: f32) -> Result<&mut Self, Error>{
        if value.is_nan() {
            // NaN is represented by 9.91E+37
            self.push_str(b"9.91E+37")
        }else if value.is_infinite() {
            // +/- Infinity is represented by +/-9.9E+37
            if value.is_sign_negative() {
                self.push_str(b"9.9E+37")
            }else {
                self.push_str(b"-9.9E+37")
            }
        }else{
            let mut buf = [b'0'; f32::FORMATTED_SIZE_DECIMAL];
            let slc = lexical_core::write::<f32>(value, &mut buf);
            self.push_str(slc)
        }
    }

    /// Get underlying buffer as a byte slice
    ///
    pub fn as_slice(&self) -> &[u8]{
        self.vec.as_slice()
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

}



#[test]
pub fn test_vecarray(){
    let mut array = ArrayVecFormatter::<[u8; 16]>::new();
    array.unit_start().unwrap().str_data(b"potato").unwrap().separator().unwrap().u8_data(0).unwrap().unit_end();
    assert_eq!(array.as_slice(), b"\"potato\",0");


}
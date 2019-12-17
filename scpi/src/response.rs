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
        #[doc="Format and push `x` to output"]
        fn $name (&mut self, x: $primitive) -> Result<(), Error>{
            let mut buf = [b'0'; <$primitive>::FORMATTED_SIZE];
            let slc = lexical_core::write::<$primitive>(x, &mut buf);
            self.push_scpi(slc)
        }
    };
}

/// Formats data according to SCPI and pushes.
///
pub trait Formatter {

    fn push_scpi(&mut self, s: &[u8]) -> Result<(), Error>;

    /// Formats `s` as a string and
    /// TODO: Escape any double quotes
    fn push_str_data(&mut self, s: &[u8]) -> Result<(), Error>{
        self.push_scpi(b"\"")?;
        self.push_scpi(s)?;
        self.push_scpi(b"\"")
    }

    fn push_f32_data(&mut self, value: f32) -> Result<(), Error>{
        if value.is_nan() {
            // NaN is represented by 9.91E+37
            self.push_scpi(b"9.91E+37")
        }else if value.is_infinite() {
            // +/- Infinity is represented by +/-9.9E+37
            if value.is_sign_negative() {
                self.push_scpi(b"9.9E+37")
            }else {
                self.push_scpi(b"-9.9E+37")
            }
        }else{
            let mut buf = [b'0'; f32::FORMATTED_SIZE];
            let slc = lexical_core::write::<f32>(value, &mut buf);
            self.push_scpi(slc)
        }
    }

    push_x!(push_i8_data, i8);
    push_x!(push_u8_data, u8);
    push_x!(push_i16_data, i16);
    push_x!(push_u16_data, u16);
    push_x!(push_i32_data, i32);
    push_x!(push_u32_data, u32);
}

impl<T: Array<Item=u8>> Formatter for ArrayVec<T> {
    fn push_scpi(&mut self, s: &[u8]) -> Result<(), Error> {
        self.try_extend_from_slice(s).map_err(|_| Error::OutOfMemory)
    }
}

#[test]
pub fn test_vecarray(){
    use crate::response::Formatter;
    let mut array = ArrayVec::<[u8; 16]>::new();
    array.push_scpi(b"potato").unwrap();
    array.push_u8_data(0).unwrap();
    assert_eq!(array.as_slice(), b"potato0");


}
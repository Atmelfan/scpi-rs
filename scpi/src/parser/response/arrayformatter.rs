use arrayvec::ArrayVec;

use crate::error::{ErrorCode, Result};

use super::{
    Formatter, ResponseUnit, RESPONSE_MESSAGE_TERMINATOR, RESPONSE_MESSAGE_UNIT_SEPARATOR,
};

impl<const CAP: usize> Formatter for ArrayVec<u8, CAP> {
    /// Internal use
    fn push_str(&mut self, s: &[u8]) -> Result<()> {
        self.try_extend_from_slice(s)
            .map_err(|_| ErrorCode::OutOfMemory.into())
    }

    fn push_byte(&mut self, b: u8) -> Result<()> {
        self.try_push(b).map_err(|_| ErrorCode::OutOfMemory.into())
    }

    fn as_slice(&self) -> &[u8] {
        self.as_slice()
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn message_start(&mut self) -> Result<()> {
        Ok(())
    }

    fn message_end(&mut self) -> Result<()> {
        self.push_byte(RESPONSE_MESSAGE_TERMINATOR)
    }

    fn response_unit(&mut self) -> Result<ResponseUnit> {
        if !self.is_empty() {
            self.push_byte(RESPONSE_MESSAGE_UNIT_SEPARATOR)?;
        }
        Ok(ResponseUnit {
            fmt: self,
            result: Ok(()),
            has_header: false,
            has_data: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Error, tree::prelude::ResponseData};

    #[test]
    fn test_vecarray() {
        let mut array = ArrayVec::<u8, 16>::new();
        array.message_start().unwrap();
        // First unit
        array
            .response_unit()
            .unwrap()
            .data(&b"potato"[..])
            .data(0u8)
            .finish()
            .unwrap();
        // Second unit
        array.response_unit().unwrap().data(42i16).finish().unwrap();
        array.message_end().unwrap();
        assert_eq!(array.as_slice(), b"\"potato\",0;42\n");
    }

    #[test]
    fn test_outamemory() {
        let mut array = ArrayVec::<u8, 1>::new();
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
        let mut array = ArrayVec::<u8, 32>::new();
        f32::INFINITY.format_response_data(&mut array).unwrap();
        array.data_separator().unwrap();
        f32::NEG_INFINITY.format_response_data(&mut array).unwrap();
        array.data_separator().unwrap();
        f32::NAN.format_response_data(&mut array).unwrap();
        // See SCPI-99 7.2.1.4 and 7.2.1.5
        assert_eq!(array.as_slice(), b"9.9E+37,-9.9E+37,9.91E+37");
    }
}

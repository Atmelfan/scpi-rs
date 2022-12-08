use crate::error::Result;

use super::{
    Formatter, ResponseUnit, RESPONSE_MESSAGE_TERMINATOR, RESPONSE_MESSAGE_UNIT_SEPARATOR,
};

impl Formatter for alloc::vec::Vec<u8> {
    /// Internal use
    fn push_str(&mut self, s: &[u8]) -> Result<()> {
        self.extend_from_slice(s);
        Ok(())
    }

    fn push_byte(&mut self, b: u8) -> Result<()> {
        self.push(b);
        Ok(())
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

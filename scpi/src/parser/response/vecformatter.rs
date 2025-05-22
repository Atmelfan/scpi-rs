use crate::error::Result;

use super::Formatter;

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
}

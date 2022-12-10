//! # 8.3.2 Channel Lists
//! Channel lists are used to specify electrical ports on an instrument. They are typically used for
//! signal routing either in a standalone switch box or in an instrument with multiple input
//! channels. An instrument with multiple channels may or may not do any signal switching as
//! the result of a channel list. Completely separate sensing channels are allowed, but may
//! appear to the language the same as channels which are switched.
//!
//! A channel list may appear in measurement, configuration, and other such commands. For
//! example, MEAS:VOLT? (@1,3,4:6) says measure the voltage on channels 1, 3, and 4
//! through 6. Whether the measurements are performed simultaneously or in the order in the
//! list is unspecified. Channel lists are also used by the ROUTe subsystem, “Command
//! Reference,” 15.1.

use crate::error::{Error, ErrorCode};
use core::convert::TryFrom;
use core::slice::Iter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChannelSpec<'a>(&'a [u8], usize);

impl<'a> IntoIterator for ChannelSpec<'a> {
    type Item = Result<isize, ErrorCode>;
    type IntoIter = ChannelSpecIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChannelSpecIterator {
            chars: self.0.iter(),
        }
    }
}

impl<'a> ChannelSpec<'a> {
    /// Returns the dimension of this channel spec
    pub fn dimension(&self) -> usize {
        self.1
    }

    /// Returns the length of this channel spec. Identical to `dimension()`
    pub fn len(&self) -> usize {
        self.dimension()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Channel list token
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    /// A channel spec consisting of at least one numeric.
    /// Example: `1!2!3` is a three-dimensional spec
    ChannelSpec(ChannelSpec<'a>),
    /// A range consisting of two channel-specs separated by a colon.
    /// Example: `1!1:2!3` is a range from '1,1' to '2,3' (row-major)
    ChannelRange(ChannelSpec<'a>, ChannelSpec<'a>),
    /// A module-channel, contains a module (numeric or character data) and a sub-channel-list.
    ModuleChannel(&'a [u8], &'a [u8]),
    /// A character pathname (can be a file, resource etc...)
    PathName(&'a [u8]),
}

/// Iterates over a channel spec, returning a result for each dimension.
/// If the iterator encounters a badly formatted value, an error will be returned.
/// Example: `"1!2!3"` would iterate as `Ok(1),Ok(2),Ok(3)`.
///
pub struct ChannelSpecIterator<'a> {
    chars: Iter<'a, u8>,
}

impl<'a> Iterator for ChannelSpecIterator<'a> {
    type Item = Result<isize, ErrorCode>;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.chars.clone().next()?;
        Some({
            if *x == b'!' {
                self.chars.next();
            }
            lexical_core::parse_partial(self.chars.as_slice())
                .map(|(n, len)| {
                    self.chars.nth(len - 1).unwrap();
                    n
                })
                .map_err(|_| ErrorCode::ExpressionError)
        })
    }
}

impl<'a> TryFrom<ChannelSpec<'a>> for isize {
    type Error = Error;

    fn try_from(value: ChannelSpec) -> Result<Self, Self::Error> {
        if value.dimension() == 1 {
            let i: isize = value
                .into_iter()
                .next()
                .unwrap_or(Err(ErrorCode::ExpressionError))?;
            Ok(i)
        } else {
            Err(Error::extended(
                ErrorCode::InvalidExpression,
                b"Unexpected channel dimension",
            ))
        }
    }
}

impl<'a> TryFrom<ChannelSpec<'a>> for usize {
    type Error = Error;

    fn try_from(value: ChannelSpec) -> Result<Self, Self::Error> {
        let i: isize = value.try_into()?;
        i.try_into()
            .map_err(|_| ErrorCode::IllegalParameterValue.into())
    }
}

impl<'a> TryFrom<ChannelSpec<'a>> for (isize, isize) {
    type Error = Error;

    fn try_from(value: ChannelSpec) -> Result<Self, Self::Error> {
        if value.dimension() == 2 {
            let i1: isize = value
                .into_iter()
                .next()
                .unwrap_or(Err(ErrorCode::ExpressionError))?;
            let i2: isize = value
                .into_iter()
                .next()
                .unwrap_or(Err(ErrorCode::ExpressionError))?;
            Ok((i1, i2))
        } else {
            Err(Error::extended(
                ErrorCode::ExpressionError,
                b"Unexpected channel dimension",
            ))
        }
    }
}

impl<'a> TryFrom<ChannelSpec<'a>> for (usize, usize) {
    type Error = Error;

    fn try_from(value: ChannelSpec) -> Result<Self, Self::Error> {
        let (i1, i2): (isize, isize) = value.try_into()?;
        Ok((
            i1.try_into()
                .map_err(|_| Error::new(ErrorCode::IllegalParameterValue))?,
            i2.try_into()
                .map_err(|_| Error::new(ErrorCode::IllegalParameterValue))?,
        ))
    }
}

impl<'a> TryFrom<ChannelSpec<'a>> for (isize, isize, isize) {
    type Error = Error;

    fn try_from(value: ChannelSpec) -> Result<Self, Self::Error> {
        if value.dimension() == 3 {
            let i1: isize = value
                .into_iter()
                .next()
                .unwrap_or(Err(ErrorCode::ExpressionError))?;
            let i2: isize = value
                .into_iter()
                .next()
                .unwrap_or(Err(ErrorCode::ExpressionError))?;
            let i3: isize = value
                .into_iter()
                .next()
                .unwrap_or(Err(ErrorCode::ExpressionError))?;
            Ok((i1, i2, i3))
        } else {
            Err(Error::extended(
                ErrorCode::ExpressionError,
                b"Unexpected channel dimension",
            ))
        }
    }
}

impl<'a> TryFrom<ChannelSpec<'a>> for (usize, usize, usize) {
    type Error = Error;

    fn try_from(value: ChannelSpec) -> Result<Self, Self::Error> {
        let (i1, i2, i3): (isize, isize, isize) = value.try_into()?;
        Ok((
            i1.try_into()
                .map_err(|_| Error::new(ErrorCode::IllegalParameterValue))?,
            i2.try_into()
                .map_err(|_| Error::new(ErrorCode::IllegalParameterValue))?,
            i3.try_into()
                .map_err(|_| Error::new(ErrorCode::IllegalParameterValue))?,
        ))
    }
}

/// Channel list expression tokenizer
#[derive(Clone)]
pub struct ChannelList<'a> {
    pub chars: Iter<'a, u8>,
    pub first: bool,
}

impl<'a> ChannelList<'a> {
    /// Create a new channel-list tokenizer
    ///
    /// # Returns
    /// `Some(Tokenizer)` - Expression is a channel-list (starts with '@')
    /// `None` - Expression is not a channel-list
    pub fn new(expr: &'a [u8]) -> Option<Self> {
        let mut iter = expr.iter();
        if let Some(x) = iter.next() {
            if *x == b'@' {
                Some(ChannelList {
                    chars: iter.clone(),
                    first: true,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn read_channel_spec(&mut self) -> Result<(&'a [u8], usize), ErrorCode> {
        let mut dim = 1usize;
        // Read full spec
        let s = self.chars.as_slice();
        while self.chars.clone().next().map_or(false, |ch| {
            ch.is_ascii_digit() || *ch == b'-' || *ch == b'+' || *ch == b'!'
        }) {
            if let Some(x) = self.chars.next() {
                if *x == b'!' {
                    dim += 1;
                }
            }
        }

        let s = &s[0..s.len() - self.chars.as_slice().len()];

        if s.is_empty() {
            Err(ErrorCode::InvalidExpression)
        } else {
            Ok((s, dim))
        }
    }

    fn read_channel_range(&mut self) -> Result<Token<'a>, ErrorCode> {
        // Read beginning spec
        let (begin, dim1) = self.read_channel_spec()?;

        // Try to read the ending spec
        if let Some(x) = self.chars.clone().next() {
            if *x == b':' {
                self.chars.next();
                let (end, dim2) = self.read_channel_spec()?;

                if dim1 != dim2 {
                    return Err(ErrorCode::InvalidExpression);
                }

                // Return range
                return Ok(Token::ChannelRange(
                    ChannelSpec(begin, dim1),
                    ChannelSpec(end, dim2),
                ));
            }
        }

        // Return spec
        Ok(Token::ChannelSpec(ChannelSpec(begin, dim1)))
    }

    fn read_channel_path(&mut self, x: u8) -> Result<Token<'a>, ErrorCode> {
        // Read pathname
        let s = self.chars.as_slice();

        if let crate::parser::tokenizer::Token::StringProgramData(s) =
            crate::parser::tokenizer::Tokenizer::new(s).read_string_data(x, true)?
        {
            self.chars.nth(s.len() + 1); //Forward iterator characters
            Ok(Token::PathName(s))
        } else {
            Err(ErrorCode::InvalidExpression)
        }
    }

    //TODO: Implement channel modules
    //fn read_channel_module(&mut self, _name: &'a [u8]) -> Result<Token<'a>, Error> {
    //    unimplemented!()
    //}
}

impl<'a> Iterator for ChannelList<'a> {
    type Item = Result<Token<'a>, ErrorCode>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut x = self.chars.clone().next()?;
        //Ignore non-leading separator
        if *x == b',' {
            if self.first {
                return Some(Err(ErrorCode::InvalidExpression));
            }
            self.chars.next().unwrap();
            x = self.chars.clone().next()?;
        }
        self.first = false;
        //Read token
        Some(match x {
            x if x.is_ascii_digit() || *x == b'+' || *x == b'-' => self.read_channel_range(),
            x if *x == b'"' || *x == b'\'' => self.read_channel_path(*x),
            _ => Err(ErrorCode::InvalidExpression),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;

    #[test]
    fn test_channel_list() {
        let mut expr = ChannelList::new(b"@1!12,3!4:5!6,'POTATO'").unwrap();

        // Destructure a spec
        let spec = expr.next().unwrap().unwrap();
        assert_eq!(spec, Token::ChannelSpec(ChannelSpec(b"1!12", 2)));
        if let Token::ChannelSpec(spec) = spec {
            let mut spec_iter = spec.into_iter();
            assert_eq!(Some(Ok(1)), spec_iter.next());
            assert_eq!(Some(Ok(12)), spec_iter.next());
            assert_eq!(None, spec_iter.next());
        } else {
            panic!("Not a channel spec")
        }

        // Destructure a range
        let range = expr.next().unwrap().unwrap();
        assert_eq!(
            range,
            Token::ChannelRange(ChannelSpec(b"3!4", 2), ChannelSpec(b"5!6", 2))
        );
        if let Token::ChannelRange(begin, end) = range {
            let mut begin_iter = begin.into_iter();
            assert_eq!(Some(Ok(3)), begin_iter.next());
            assert_eq!(Some(Ok(4)), begin_iter.next());
            assert_eq!(None, begin_iter.next());
            let mut end_iter = end.into_iter();
            assert_eq!(Some(Ok(5)), end_iter.next());
            assert_eq!(Some(Ok(6)), end_iter.next());
            assert_eq!(None, end_iter.next());
        } else {
            panic!("Not a channel range")
        }
        assert_eq!(expr.next(), Some(Ok(Token::PathName(b"POTATO"))));
        assert_eq!(expr.next(), None);
    }
}

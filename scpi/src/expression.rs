/// Contains numeric-list tokenizer
///
pub mod numeric_list {
    use crate::error::ErrorCode;
    use core::slice::Iter;

    #[derive(Clone, PartialEq)]
    pub enum Token {
        Numeric(isize),
        NumericRange(isize, isize),
        Separator,
    }

    /// Numeric list expression tokenizer
    #[derive(Clone)]
    pub struct Tokenizer<'a> {
        pub chars: Iter<'a, u8>,
        pub expect_num: bool,
    }

    impl<'a> Tokenizer<'a> {
        fn read_numeric_data(&mut self) -> Result<Token, ErrorCode> {
            /* Read mantissa */
            let (begin, len) = lexical_core::parse_partial::<isize>(self.chars.as_slice())
                .map_err(|_| ErrorCode::NumericDataError)?;
            self.chars.nth(len - 1);

            if let Some(c) = self.chars.clone().next() {
                //&& *c == b':' {
                if *c == b':' {
                    self.chars.next();
                    let (end, len) = lexical_core::parse_partial::<isize>(self.chars.as_slice())
                        .map_err(|_| ErrorCode::NumericDataError)?;
                    return if len == 0 {
                        Err(ErrorCode::InvalidExpression)
                    } else {
                        self.chars.nth(len - 1);
                        Ok(Token::NumericRange(begin, end))
                    };
                }
            }

            Ok(Token::Numeric(begin))
        }
    }

    impl<'a> Iterator for Tokenizer<'a> {
        type Item = Result<Token, ErrorCode>;

        fn next(&mut self) -> Option<Self::Item> {
            let x = self.chars.clone().next()?;
            Some(match x {
                b',' => {
                    if self.expect_num {
                        Err(ErrorCode::InvalidExpression)
                    } else {
                        self.chars.next().unwrap();
                        self.expect_num = true;
                        Ok(Token::Separator)
                    }
                }
                x if x.is_ascii_digit() || *x == b'-' || *x == b'+' => {
                    self.expect_num = false;
                    self.read_numeric_data()
                }
                _ => Err(ErrorCode::InvalidExpression),
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::error::ErrorCode;
        use crate::expression::numeric_list::{Token, Tokenizer};
        use core::fmt;

        extern crate std;

        impl fmt::Debug for Token {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Token::Separator => write!(f, ","),
                    Token::Numeric(x) => write!(f, "{}", x),
                    Token::NumericRange(x, y) => write!(f, "{}:{}", x, y),
                }
            }
        }

        #[test]
        fn test_numeric_data() {
            let spec = Tokenizer {
                chars: b"2".iter(),
                expect_num: true,
            }
            .read_numeric_data();
            assert_eq!(spec, Ok(Token::Numeric(2)));
            let range = Tokenizer {
                chars: b"2:5".iter(),
                expect_num: true,
            }
            .read_numeric_data();
            assert_eq!(range, Ok(Token::NumericRange(2, 5)));
            let specfail = Tokenizer {
                chars: b"2::5".iter(),
                expect_num: true,
            }
            .read_numeric_data();
            assert_eq!(specfail, Err(ErrorCode::InvalidExpression));
        }

        #[test]
        fn test_numeric_list() {
            let mut expr = Tokenizer {
                chars: b"1,2:5".iter(),
                expect_num: true,
            };
            assert_eq!(expr.next(), Some(Ok(Token::Numeric(1))));
            assert_eq!(expr.next(), Some(Ok(Token::Separator)));
            assert_eq!(expr.next(), Some(Ok(Token::NumericRange(2, 5))));
            assert_eq!(expr.next(), None);
        }

        #[test]
        fn test_numeric_leading() {
            let mut expr = Tokenizer {
                chars: b",1,2:5".iter(),
                expect_num: true,
            };
            assert_eq!(expr.next(), Some(Err(ErrorCode::InvalidExpression)));
        }

        #[test]
        fn test_numeric_repeated() {
            let mut expr = Tokenizer {
                chars: b"1,,2:5".iter(),
                expect_num: true,
            };
            assert_eq!(expr.next(), Some(Ok(Token::Numeric(1))));
            assert_eq!(expr.next(), Some(Ok(Token::Separator)));
            assert_eq!(expr.next(), Some(Err(ErrorCode::InvalidExpression)));
        }
    }
}

/// Contains channel-list tokenizer
///
///
pub mod channel_list {
    use crate::error::ErrorCode;
    use crate::expression::channel_list::Token::ChannelRange;
    use core::slice::Iter;

    #[derive(Clone, Copy, PartialEq)]
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
    #[derive(Clone, PartialEq)]
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
        /// A channel separator i.e. comma
        Separator,
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

    impl<'a> Token<'a> {}

    /// Channel list expression tokenizer
    #[derive(Clone)]
    pub struct Tokenizer<'a> {
        pub chars: Iter<'a, u8>,
    }

    impl<'a> Tokenizer<'a> {
        /// Create a new channel-list tokenizer
        ///
        /// # Returns
        /// `Some(Tokenizer)` - Expression is a channel-list (starts with '@')
        /// `None` - Expression is not a channel-list
        pub fn new(expr: &'a [u8]) -> Option<Self> {
            let mut iter = expr.iter();
            if let Some(x) = iter.next() {
                if *x == b'@' {
                    Some(Tokenizer {
                        chars: iter.clone(),
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
                    return Ok(ChannelRange(
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

            if let crate::Token::StringProgramData(s) =
                crate::tokenizer::Tokenizer::new(s).read_string_data(x, true)?
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

    impl<'a> Iterator for Tokenizer<'a> {
        type Item = Result<Token<'a>, ErrorCode>;

        fn next(&mut self) -> Option<Self::Item> {
            let x = self.chars.clone().next()?;
            Some(match x {
                b',' => {
                    self.chars.next().unwrap();
                    Ok(Token::Separator)
                }
                x if x.is_ascii_digit() || *x == b'+' || *x == b'-' => self.read_channel_range(),
                x if *x == b'"' || *x == b'\'' => self.read_channel_path(*x),
                _ => Err(ErrorCode::InvalidExpression),
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::expression::channel_list::{ChannelSpec, Token, Tokenizer};
        use core::fmt;

        extern crate std;

        impl fmt::Debug for Token<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Token::Separator => write!(f, ","),
                    Token::ChannelSpec(x) => write!(f, "{:?}", x.0),
                    Token::ChannelRange(x, y) => write!(f, "{:?}:{:?}", x.0, y.0),
                    Token::PathName(x) => write!(f, "{:?}", x),
                    _ => write!(f, "(UNKNOWN)"),
                }
            }
        }

        #[test]
        fn test_channel_list() {
            let mut expr = Tokenizer::new(b"@1!12,3!4:5!6,'POTATO'").unwrap();

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

            assert_eq!(expr.next(), Some(Ok(Token::Separator)));

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

            assert_eq!(expr.next(), Some(Ok(Token::Separator)));
            assert_eq!(expr.next(), Some(Ok(Token::PathName(b"POTATO"))));
            assert_eq!(expr.next(), None);
        }
    }
}

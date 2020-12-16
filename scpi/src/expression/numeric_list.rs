//! # 8.3.3 Numeric Lists
//!
//! A numeric list is a an expression format for compactly expressing numbers and ranges of
//! numbers in a single parameter.

use crate::error::{Error, ErrorCode};
use crate::tokenizer;

type Number<'a> = tokenizer::Token<'a>;

#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
    Numeric(Number<'a>),
    NumericRange(Number<'a>, Number<'a>),
}

/// Numeric list expression tokenizer
#[derive(Clone)]
pub struct NumericList<'a> {
    pub tokenizer: tokenizer::Tokenizer<'a>,
    pub first: bool,
}

impl<'a> NumericList<'a> {
    pub fn new(s: &'a [u8]) -> NumericList<'a> {
        NumericList {
            tokenizer: tokenizer::Tokenizer::new(s),
            first: true,
        }
    }

    fn read_numeric_data(&mut self) -> Result<Token<'a>, ErrorCode> {
        let begin: Number = self.tokenizer.read_nrf()?;
        if let Some(c) = self.tokenizer.chars.clone().next() {
            //&& *c == b':' {
            if *c == b':' {
                self.tokenizer.chars.next();
                let end = self.tokenizer.read_nrf()?;
                return Ok(Token::NumericRange(begin, end));
            }
        }

        Ok(Token::Numeric(begin))
    }
}

impl<'a> Iterator for NumericList<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        //TODO: This has to be tokenizer abuse or something...
        let char = self.tokenizer.chars.clone().next()?;

        Some(match char {
            b',' if !self.first => {
                self.tokenizer.chars.next().unwrap();
                self.read_numeric_data()
                    .map_err(|err| Error::extended(ErrorCode::InvalidExpression, err.get_message()))
            }
            x if x.is_ascii_digit() || *x == b'-' || *x == b'+' && self.first => {
                self.first = false;
                self.read_numeric_data()
                    .map_err(|err| Error::extended(ErrorCode::InvalidExpression, err.get_message()))
            }
            _ => Err(Error::extended(
                ErrorCode::InvalidExpression,
                b"Invalid character",
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, ErrorCode};
    use crate::expression::numeric_list::{Number, NumericList, Token};

    extern crate std;

    #[test]
    fn test_numeric_data() {
        let spec = NumericList::new(b"2").read_numeric_data();
        assert_eq!(
            spec,
            Ok(Token::Numeric(Number::DecimalNumericProgramData(b"2")))
        );
        let range = NumericList::new(b"2:5").read_numeric_data();
        assert_eq!(
            range,
            Ok(Token::NumericRange(
                Number::DecimalNumericProgramData(b"2"),
                Number::DecimalNumericProgramData(b"5")
            ))
        );
        let specfail = NumericList::new(b"2::5").read_numeric_data();
        assert_eq!(specfail, Err(ErrorCode::NumericDataError.into()));
    }

    #[test]
    fn test_numeric_list() {
        let mut expr = NumericList::new(b"3.1415,1.1:3.9e6");
        assert_eq!(
            expr.next().unwrap(),
            Ok(Token::Numeric(Number::DecimalNumericProgramData(b"3.1415")))
        );
        assert_eq!(
            expr.next().unwrap(),
            Ok(Token::NumericRange(
                Number::DecimalNumericProgramData(b"1.1"),
                Number::DecimalNumericProgramData(b"3.9e6")
            ))
        );
        assert_eq!(expr.next(), None);
    }

    #[test]
    fn test_numeric_leading() {
        let mut expr = NumericList::new(b",1,2:5");
        assert_eq!(
            expr.next().unwrap(),
            Err(Error::extended(
                ErrorCode::InvalidExpression,
                b"Invalid character"
            ))
        );
    }

    #[test]
    fn test_numeric_repeated() {
        let mut expr = NumericList::new(b"1,,2:5");
        assert_eq!(
            expr.next().unwrap(),
            Ok(Token::Numeric(Number::DecimalNumericProgramData(b"1")))
        );
        assert_eq!(
            expr.next().unwrap(),
            Err(Error::extended(
                ErrorCode::InvalidExpression,
                ErrorCode::NumericDataError.get_message()
            ))
        );
    }
}

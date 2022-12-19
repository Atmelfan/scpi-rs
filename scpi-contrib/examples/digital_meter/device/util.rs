use scpi::tree::prelude::{ResponseData, Formatter, Token, Error};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Auto<T> {
    Auto,
    Numeric(T),
}

impl<T> ResponseData for Auto<T>
where
    T: ResponseData,
{
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> scpi::error::Result<()> {
        match self {
            Auto::Auto => formatter.push_ascii(b"AUTO"),
            Auto::Numeric(t) => t.format_response_data(formatter),
        }
    }
}

impl<'a, T> TryFrom<Token<'a>> for Auto<T>
where
    T: TryFrom<Token<'a>, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Self, Self::Error> {
        match value {
            Token::CharacterProgramData(s) => match s {
                x if scpi::parser::mnemonic_compare(b"AUTO", x) => Ok(Self::Auto),
                _ => Ok(Self::Numeric(T::try_from(value)?)),
            },
            t => Ok(Self::Numeric(T::try_from(t)?)),
        }
    }
}
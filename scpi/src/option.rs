use crate::error::{ErrorCode, Result};
use crate::response::{Data, Formatter};
use crate::tokenizer::Token;

///
pub trait ScpiEnum
where
    Self: Sized,
{
    ///
    ///
    fn from_mnemonic(s: &[u8]) -> Option<Self>;

    fn to_mnemonic(&self) -> &'static [u8];

    fn from_token(value: Token) -> Result<Self> {
        if let Token::CharacterProgramData(s) = value {
            Self::from_mnemonic(s).ok_or_else(|| ErrorCode::IllegalParameterValue.into())
        } else {
            Err(ErrorCode::DataTypeError.into())
        }
    }
}

impl<T> Data for T
where
    T: ScpiEnum,
{
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        formatter.push_str(self.to_mnemonic())
    }
}

#[cfg(test)]
mod tests {
    extern crate self as scpi;
    use super::ScpiEnum;
    use crate::error::ErrorCode;
    use crate::tokenizer::Token;
    use core::convert::TryFrom;

    #[derive(Copy, Clone, PartialEq, Debug, ScpiEnum)]
    enum MyEnum {
        #[scpi(mnemonic = b"BINary")]
        Binary,
        #[scpi(mnemonic = b"REAL")]
        Real,
        #[scpi(mnemonic = b"ASCii")]
        Ascii,
    }

    #[test]
    fn test_enum() {
        assert_eq!(MyEnum::from_mnemonic(b"real"), Some(MyEnum::Real));
        assert_eq!(MyEnum::from_mnemonic(b"bin"), Some(MyEnum::Binary));
        assert_eq!(MyEnum::from_mnemonic(b"AsCiI"), Some(MyEnum::Ascii));
        assert_eq!(MyEnum::from_mnemonic(b"potato"), None);
    }

    #[test]
    fn test_enum_types() {
        assert_eq!(
            MyEnum::from_token(Token::CharacterProgramData(b"real")),
            Ok(MyEnum::Real)
        );
        assert_eq!(
            MyEnum::from_token(Token::CharacterProgramData(b"bin")),
            Ok(MyEnum::Binary)
        );
        assert_eq!(
            MyEnum::from_token(Token::CharacterProgramData(b"potato")),
            Err(ErrorCode::IllegalParameterValue.into())
        );
        assert_eq!(
            MyEnum::from_token(Token::DecimalNumericProgramData(b"3.5")),
            Err(ErrorCode::DataTypeError.into())
        );
    }
}

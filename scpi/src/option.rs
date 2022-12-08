///
pub trait ScpiEnum
where
    Self: Sized,
{
    ///
    fn from_mnemonic(s: &[u8]) -> Option<Self>;

    fn mnemonic(&self) -> &'static [u8];

    /// Get the mnemonic short form
    ///
    /// Example: 'MNEMonic' would return 'MNEM'
    fn short_form(&self) -> &'static [u8] {
        let mnemonic = self.mnemonic();
        let len = mnemonic
            .iter()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            .count();
        &mnemonic[..len]
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
        #[scpi(mnemonic = b"L125")]
        L125,
    }

    #[test]
    fn test_enum() {
        assert_eq!(MyEnum::from_mnemonic(b"real"), Some(MyEnum::Real));
        assert_eq!(MyEnum::from_mnemonic(b"bin"), Some(MyEnum::Binary));
        assert_eq!(MyEnum::from_mnemonic(b"AsCiI"), Some(MyEnum::Ascii));
        assert_eq!(MyEnum::from_mnemonic(b"L125"), Some(MyEnum::L125));
        assert_eq!(MyEnum::from_mnemonic(b"potato"), None);
    }

    #[test]
    fn test_short_form() {
        extern crate std;
        std::println!(
            "{}",
            std::str::from_utf8(MyEnum::Binary.short_form()).unwrap()
        );
        assert_eq!(MyEnum::Binary.short_form(), b"BIN");
        assert_eq!(MyEnum::Real.short_form(), b"REAL");
        assert_eq!(MyEnum::Ascii.short_form(), b"ASC");
        assert_eq!(MyEnum::L125.short_form(), b"L125");
    }

    #[test]
    fn test_enum_types() {
        assert_eq!(
            MyEnum::try_from(Token::CharacterProgramData(b"real")),
            Ok(MyEnum::Real)
        );
        assert_eq!(
            MyEnum::try_from(Token::CharacterProgramData(b"bin")),
            Ok(MyEnum::Binary)
        );
        assert_eq!(
            MyEnum::try_from(Token::CharacterProgramData(b"potato")),
            Err(ErrorCode::IllegalParameterValue.into())
        );
        assert_eq!(
            MyEnum::try_from(Token::DecimalNumericProgramData(b"3.5")),
            Err(ErrorCode::DataTypeError.into())
        );
    }
}

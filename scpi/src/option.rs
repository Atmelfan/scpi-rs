///
trait ScpiEnum
where
    Self: Sized,
{
    ///
    ///
    fn from_mnemonic(s: &[u8]) -> Option<Self>;

    //fn to_mnemonic(&self) -> &'static [u8];
}

#[cfg(test)]
mod tests {
    extern crate self as scpi;
    use super::ScpiEnum;

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
}

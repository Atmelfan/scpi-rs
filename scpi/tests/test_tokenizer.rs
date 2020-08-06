use scpi::error::ErrorCode;
use scpi::tokenizer::{Token, Tokenizer};

extern crate std;

macro_rules! match_tokens {
    ($s:literal => $($tok:expr),*) => {
        let expected = [
            $($tok),*
        ];
        let tokens: Vec<Result<Token, ErrorCode>> = Tokenizer::new($s).into_iter().collect();
        for (a, b) in tokens.iter().zip(expected.iter()) {
            assert_eq!(a, b);
        }
        assert_eq!(tokens.len(), expected.len());
    };

}

#[test]
fn test_parse_common() {
    match_tokens![b"*IDN?" =>
        Ok(Token::HeaderCommonPrefix),
        Ok(Token::ProgramMnemonic(b"IDN")),
        Ok(Token::HeaderQuerySuffix)
    ];
}

#[test]
fn test_parse_suffix() {
    // Test that suffix are read correctly after decimal numeric and fault otherwise
    match_tokens![b"TST 1V;TST 1 V;TST 'STRING' V" =>
        Ok(Token::ProgramMnemonic(b"TST")),
        Ok(Token::ProgramHeaderSeparator),
        Ok(Token::DecimalNumericSuffixProgramData(b"1", b"V")),
        Ok(Token::ProgramMessageUnitSeparator),
        Ok(Token::ProgramMnemonic(b"TST")),
        Ok(Token::ProgramHeaderSeparator),
        Ok(Token::DecimalNumericSuffixProgramData(b"1", b"V")),
        Ok(Token::ProgramMessageUnitSeparator),
        Ok(Token::ProgramMnemonic(b"TST")),
        Ok(Token::ProgramHeaderSeparator),
        Err(ErrorCode::SuffixNotAllowed),
        Ok(Token::CharacterProgramData(b"V"))//Normally you wouldn't continue parsing
    ];
}

#[test]
fn test_parse_programdata() {
    match_tokens![b"*STB #HAA , 255, \"STRING\", 1 SUFFIX, #202\x01\x02, CHAR, (1,11,3:9)\n" =>
        Ok(Token::HeaderCommonPrefix),
        Ok(Token::ProgramMnemonic(b"STB")),
        Ok(Token::ProgramHeaderSeparator),
        Ok(Token::NonDecimalNumericProgramData(0xaa)),
        Ok(Token::ProgramDataSeparator),
        Ok(Token::DecimalNumericProgramData(b"255")),
        Ok(Token::ProgramDataSeparator),
        Ok(Token::StringProgramData(b"STRING")),
        Ok(Token::ProgramDataSeparator),
        Ok(Token::DecimalNumericSuffixProgramData(b"1", b"SUFFIX")),
        Ok(Token::ProgramDataSeparator),
        Ok(Token::ArbitraryBlockData(&[1u8,2u8])),
        Ok(Token::ProgramDataSeparator),
        Ok(Token::CharacterProgramData(b"CHAR")),
        Ok(Token::ProgramDataSeparator),
        Ok(Token::ExpressionProgramData(b"1,11,3:9"))
    ];
}

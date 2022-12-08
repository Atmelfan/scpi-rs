use super::util;
use crate::error::ErrorCode;
use crate::tokenizer::{Token, Tokenizer};

extern crate std;

#[test]
fn test_split_mnemonic() {
    assert_eq!(
        util::mnemonic_split_index(b"TRIGger54"),
        Some((b"TRIGger".as_ref(), b"54".as_ref()))
    );
    assert_eq!(
        util::mnemonic_split_index(b"T123r54"),
        Some((b"T123r".as_ref(), b"54".as_ref()))
    );
    assert_eq!(util::mnemonic_split_index(b"TRIGger"), None);
    assert_eq!(util::mnemonic_split_index(b""), None);
}

#[test]
fn test_compare_mnemonic() {
    //Should return true
    assert!(util::mnemonic_compare(b"TRIGger", b"trigger"));
    assert!(util::mnemonic_compare(b"TRIGger", b"trig"));
    assert!(util::mnemonic_compare(b"TRIGger", b"TRIGGER"));
    assert!(util::mnemonic_compare(b"TRIGger", b"TRIG"));
    //Should return false
    assert!(!util::mnemonic_compare(b"TRIGger", b"TRIGge"));
    assert!(!util::mnemonic_compare(b"TRIGger", b"triggeristoodamnlong"));
    assert!(!util::mnemonic_compare(b"TRIGger", b"tri"));
}

#[test]
fn test_eq_mnemonic() {
    //
    assert!(Token::ProgramMnemonic(b"trigger").match_program_header(b"TRIGger1"));
    assert!(Token::ProgramMnemonic(b"trig").match_program_header(b"TRIGger1"));
    assert!(Token::ProgramMnemonic(b"trigger1").match_program_header(b"TRIGger1"));
    assert!(Token::ProgramMnemonic(b"trigger1").match_program_header(b"TRIGger"));
    assert!(Token::ProgramMnemonic(b"trig1").match_program_header(b"TRIGger1"));
    assert!(!Token::ProgramMnemonic(b"trigger2").match_program_header(b"TRIGger1"));
    assert!(!Token::ProgramMnemonic(b"trig2").match_program_header(b"TRIGger1"));
    assert!(!Token::ProgramMnemonic(b"trig2").match_program_header(b"TRIGger1"));

    assert!(Token::ProgramMnemonic(b"trigger2").match_program_header(b"TRIGger2"));
    assert!(Token::ProgramMnemonic(b"trig2").match_program_header(b"TRIGger2"));
    assert!(!Token::ProgramMnemonic(b"trigger").match_program_header(b"TRIGger2"));
    assert!(!Token::ProgramMnemonic(b"trig").match_program_header(b"TRIGger2"));
    assert!(!Token::ProgramMnemonic(b"trigger1").match_program_header(b"TRIGger2"));
    assert!(!Token::ProgramMnemonic(b"trig1").match_program_header(b"TRIGger2"));
}

#[test]
fn test_read_character_data() {
    assert_eq!(
        Tokenizer::new(b"CHARacter4 , pperg").read_character_data(),
        Ok(Token::CharacterProgramData(b"CHARacter4"))
    );
    assert_eq!(
        Tokenizer::new(b"CHARacterIsTooLong").read_character_data(),
        Err(ErrorCode::CharacterDataTooLong)
    );
    assert_eq!(
        Tokenizer::new(b"Character Invalid").read_character_data(),
        Err(ErrorCode::InvalidCharacterData)
    );
}

#[test]
fn test_read_numeric_data() {
    //TODO: FIX EXPONENTS!

    assert_eq!(
        Tokenizer::new(b"25").read_numeric_data().unwrap(),
        Token::DecimalNumericProgramData(b"25")
    );

    assert_eq!(
        Tokenizer::new(b".2").read_numeric_data().unwrap(),
        Token::DecimalNumericProgramData(b".2")
    );

    assert_eq!(
        Tokenizer::new(b"1.0E5").read_numeric_data().unwrap(),
        Token::DecimalNumericProgramData(b"1.0E5")
    );

    assert_eq!(
        Tokenizer::new(b"-25e5").read_numeric_data().unwrap(),
        Token::DecimalNumericProgramData(b"-25e5")
    );

    assert_eq!(
        Tokenizer::new(b"25E-2").read_numeric_data().unwrap(),
        Token::DecimalNumericProgramData(b"25E-2")
    );

    assert_eq!(
        Tokenizer::new(b".1E2").read_numeric_data().unwrap(),
        Token::DecimalNumericProgramData(b".1E2")
    );

    assert_eq!(
        Tokenizer::new(b".1E2  SUFFIX").read_numeric_data().unwrap(),
        Token::DecimalNumericSuffixProgramData(b".1E2", b"SUFFIX")
    );

    assert_eq!(
        Tokenizer::new(b".1E2  /S").read_numeric_data().unwrap(),
        Token::DecimalNumericSuffixProgramData(b".1E2", b"/S")
    );

    assert_eq!(
        Tokenizer::new(b".1E2  'SUFFIX'")
            .read_numeric_data()
            .unwrap_err(),
        ErrorCode::InvalidSuffix
    );
}

#[test]
fn test_read_suffix_data() {}

#[test]
fn test_read_numeric_suffix_data() {
    let mut tokenizer = Tokenizer::new(b"header 25 KHZ , 12.7E6 KOHM.M/S-2");
    assert_eq!(
        tokenizer.next(),
        Some(Ok(Token::ProgramMnemonic(b"header")))
    );
    assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramHeaderSeparator)));
    assert_eq!(
        tokenizer.next(),
        Some(Ok(Token::DecimalNumericSuffixProgramData(b"25", b"KHZ")))
    );
    assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramDataSeparator)));
    assert_eq!(
        tokenizer.next(),
        Some(Ok(Token::DecimalNumericSuffixProgramData(
            b"12.7E6",
            b"KOHM.M/S-2"
        )))
    );
    assert_eq!(tokenizer.next(), None);
}

#[test]
fn test_read_string_data() {
    assert_eq!(
        Tokenizer::new(b"\"MOHM\",  gui").read_string_data(b'"', true),
        Ok(Token::StringProgramData(b"MOHM"))
    );
    assert_eq!(
        Tokenizer::new(b"'MOHM',  gui").read_string_data(b'\'', true),
        Ok(Token::StringProgramData(b"MOHM"))
    );
    assert_eq!(
        Tokenizer::new(b"'MO''HM',  gui").read_string_data(b'\'', true),
        Ok(Token::StringProgramData(b"MO''HM"))
    );

    assert_eq!(
        Tokenizer::new(b"\"MOHM").read_string_data(b'"', true),
        Err(ErrorCode::InvalidStringData)
    );
    assert_eq!(
        Tokenizer::new(b"'MOHM").read_string_data(b'"', true),
        Err(ErrorCode::InvalidStringData)
    );
    assert_eq!(
        Tokenizer::new(b"'MO\xffHM").read_string_data(b'"', true),
        Err(ErrorCode::InvalidCharacter)
    );
}

#[test]
fn test_read_arb_data() {
    assert_eq!(
        Tokenizer::new(b"02\x01\x02,").read_arbitrary_data(b'2'),
        Ok(Token::ArbitraryBlockData(&[1, 2]))
    );
    assert_eq!(
        Tokenizer::new(b"2\x01\x02,").read_arbitrary_data(b'1'),
        Ok(Token::ArbitraryBlockData(&[1, 2]))
    );

    // Error, too short
    assert_eq!(
        Tokenizer::new(b"02\x01").read_arbitrary_data(b'2'),
        Err(ErrorCode::InvalidBlockData)
    );

    // Error, invalid header
    assert_eq!(
        Tokenizer::new(b"a2\x01\x02,").read_arbitrary_data(b'2'),
        Err(ErrorCode::InvalidBlockData)
    );

    // Error, header too short/invalid
    assert_eq!(
        Tokenizer::new(b"2\x01\x02,").read_arbitrary_data(b'2'),
        Err(ErrorCode::InvalidBlockData)
    );

    // Indefinite length
    assert_eq!(
        Tokenizer::new(b"\x01\x02\n").read_arbitrary_data(b'0'),
        Ok(Token::ArbitraryBlockData(&[1, 2]))
    );

    // Error, indefinite not terminated by newline
    assert_eq!(
        Tokenizer::new(b"\x01\x02").read_arbitrary_data(b'0'),
        Err(ErrorCode::InvalidBlockData)
    );
}

#[test]
fn test_read_expr_data() {
    assert_eq!(
        Tokenizer::new(b"(@1!2,2,3,4,5,#,POTATO)").read_expression_data(),
        Ok(Token::ExpressionProgramData(b"@1!2,2,3,4,5,#,POTATO"))
    );
}

macro_rules! match_tokens {
    ($s:literal => $($tok:expr),*) => {
        let expected = [
            $($tok),*
        ];
        let tokens: std::vec::Vec<Result<Token, ErrorCode>> = Tokenizer::new($s).into_iter().collect();
        for (a, b) in tokens.iter().zip(expected.iter()) {
            assert_eq!(a, b);
        }
        assert_eq!(tokens.len(), expected.len());
    };

}

#[test]
fn test_parse_common() {
    match_tokens![b"*IDN?" =>
        Ok(Token::ProgramMnemonic(b"*IDN")),
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
        Ok(Token::ProgramMnemonic(b"*STB")),
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

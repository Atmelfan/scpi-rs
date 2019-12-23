use crate::error::Error;

use core::slice::Iter;
use core::str;
use crate::error::Error::DataTypeError;
use core::convert::{TryFrom, TryInto};
use crate::error;
use core::num::TryFromIntError;
use lexical_core::Float;

/// SCPI tokens
///
///
#[derive(Debug,PartialEq)]
pub enum Token<'a> {
    /// Defined as \<mnemonic separator\> consisting of a single `:` character
    HeaderMnemonicSeparator,    //:
    HeaderCommonPrefix,         //*
    HeaderQuerySuffix,          //?

    ProgramMessageUnitSeparator,//;
    ProgramMessageTerminator,   //\n+END
    ProgramHeaderSeparator,     //SP
    ProgramDataSeparator,       //,
    ProgramMnemonic(&'a [u8]),   // <program mnemonic>
    CharacterProgramData(&'a [u8]),
    DecimalNumericProgramData(f32),
    SuffixProgramData(&'a [u8]),
    NonDecimalNumericProgramData(u32),
    StringProgramData(&'a [u8]),
    ArbitraryBlockData(&'a [u8]),
    Utf8BlockData(&'a str)
}



impl<'a> Token<'a> {

    pub fn is_data_object(&self) -> bool {
        match self {
            Token::CharacterProgramData(_) | Token::DecimalNumericProgramData(_) | Token::SuffixProgramData(_) |
            Token::NonDecimalNumericProgramData(_) | Token::StringProgramData(_) | Token::ArbitraryBlockData(_) => true,
            _ => false
        }
    }

    pub fn mnemonic_split_index(mnemonic: &'a [u8]) -> Option<(&'a [u8], &'a [u8])>{
        let last = mnemonic.iter().rposition(|p| !p.is_ascii_digit());

        if let Some(index) = last{
            if index == mnemonic.len()-1 {
                None
            }else{
                Some(mnemonic.split_at(index+1))
            }
        }else {
            None
        }
    }

    pub fn mnemonic_compare(mnemonic: &[u8], s: &[u8]) -> bool {
        //LONGform == longform || LONG == long
        mnemonic.len() >= s.len() && {
            let mut s_iter = s.iter();
            mnemonic.iter().all(|m|
                s_iter.next().map_or(!(m.is_ascii_uppercase() || m.is_ascii_digit()), |x| m.eq_ignore_ascii_case(x))
            )
        }
    }

    //TOKen<digits> = [TOK|TOKEN](digits>1)
    pub fn eq_mnemonic(&self, mnemonic: &'a [u8]) -> bool{

        match self {
            Token::ProgramMnemonic(s) | Token::CharacterProgramData(s) => {
                Self::mnemonic_compare(mnemonic, s) || match (Self::mnemonic_split_index(mnemonic), Self::mnemonic_split_index(s)) {
                    (None, None) => false,
                    (Some((m, index)), None) => Self::mnemonic_compare(m, s) && index == b"1",
                    (None, Some((x, index))) => Self::mnemonic_compare(mnemonic, x) && index == b"1",
                    (Some((m, index1)),Some((x, index2))) => Self::mnemonic_compare(m, x) && (index1 == index2)
                }
            },
            _ => false
        }
    }

    /// If token is `CharacterProgramData`, try to map into another token.
    /// If token is not a `CharacterProgramData` the token will be returned as-is
    ///
    /// # Arguments
    /// * `special` - Function which maps a slice (likely a mnemonic) to a new Token (or emit an error)
    ///
    /// # Example
    ///
    /// ```
    /// use scpi::tokenizer::Token;
    /// use scpi::error::Error;
    /// let s = Token::CharacterProgramData(b"potato");
    ///
    /// let value = s.map_special(|s| match s {
    ///     x if Token::mnemonic_compare(b"POTato", x) => Ok(Token::DecimalNumericProgramData(5.0)),
    ///     x if Token::mnemonic_compare(b"PINEapple", x) => Ok(Token::DecimalNumericProgramData(1.0)),
    ///     _ => Err(Error::IllegalParameterValue)
    /// });
    ///
    /// //assert_eq!(value, Ok(Token::DecimalNumericProgramData(5.0)));
    ///
    /// ```
    pub fn map_special<F>(self, special: F) -> Result<Token<'a>, Error>
        where F: FnOnce(&[u8]) -> Result<Token<'a>, Error> {
        if let Token::CharacterProgramData(s) =  self {
            special(s)
        }else{
            Ok(self)
        }
    }


}

/// Allow cast from `Token::DecimalNumericProgramData | Token::CharacterProgramData` to f32.
/// # Returns
/// * `Ok(value)` - If token is a valid float object
/// * `Err(DataTypeError)` if another type of data object,
/// * `Err(SyntaxError)` if not a data object (shouldn't happen).
///
impl<'a> TryInto<f32> for Token<'a> {
    type Error = error::Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Token::DecimalNumericProgramData(value) => Ok(value),
            Token::CharacterProgramData(s) => match s.clone() {
                //Check for special float values
                ref x if Self::mnemonic_compare(b"INFinity", x) => Ok(f32::INFINITY),
                ref x if Self::mnemonic_compare(b"NINFinity", x) => Ok(f32::NEG_INFINITY),
                ref x if Self::mnemonic_compare(b"NAN", x) => Ok(f32::NAN),
                _ => Err(Error::DataTypeError)
            }
            Token::SuffixProgramData(_) | Token::NonDecimalNumericProgramData(_) | Token::StringProgramData(_)
            | Token::ArbitraryBlockData(_) => Err(Error::DataTypeError),
            _ => Err(Error::SyntaxError)
        }
    }
}

/// Allow cast from `Token::StringProgramData` to &\[u8\].
/// # Returns
/// * `Ok(value)` - If token is a valid string object
/// * `Err(DataTypeError)` if another type of data object,
/// * `Err(SyntaxError)` if not a data object (shouldn't happen).
///
impl<'a> TryInto<&'a [u8]> for Token<'a> {
    type Error = error::Error;

    fn try_into(self) -> Result<&'a [u8], Self::Error> {
        match self {
            Token::StringProgramData(s) => Ok(s),
            Token::SuffixProgramData(_) | Token::NonDecimalNumericProgramData(_) | Token::DecimalNumericProgramData(_)
            | Token::ArbitraryBlockData(_) | Token::CharacterProgramData(_) => Err(Error::DataTypeError),
            _ => Err(Error::SyntaxError)
        }
    }
}

macro_rules! impl_tryfrom_integer {
    ($from:ty) => {
        impl<'a> TryInto<$from> for Token<'a> {
            type Error = error::Error;

            fn try_into(self) -> Result<$from, Self::Error> {
                match self {
                    Token::DecimalNumericProgramData(value) => {
                        if value.is_finite() {
                            <$from>::try_from((value + 0.5f32) as i32).map_err(|_| Error::DataOutOfRange)
                        }else{
                            // Nan/Inf -> Integer is undefined, return DataOutOfRange
                            Err(Error::DataOutOfRange)
                        }
                    },
                    Token::NonDecimalNumericProgramData(value) => <$from>::try_from(value).map_err(|_| Error::DataOutOfRange),
                    Token::SuffixProgramData(_) | Token::CharacterProgramData(_) | Token::StringProgramData(_)
                    | Token::ArbitraryBlockData(_) => Err(Error::DataTypeError),
                    _ => Err(Error::SyntaxError)
                }
            }
        }
    };
}

impl_tryfrom_integer!(i32);
impl_tryfrom_integer!(u32);
impl_tryfrom_integer!(i16);
impl_tryfrom_integer!(u16);
impl_tryfrom_integer!(i8);
impl_tryfrom_integer!(u8);


#[derive(Clone)]
pub struct Tokenizer<'a> {
    chars: Iter<'a, u8>,
    in_header: bool,
    in_common: bool,
    in_numeric: bool
}

impl<'a> Tokenizer<'a> {

    /// Attempts to consume a data separator
    /// Returns an error if next token is not a separator.
    /// If next token is a terminator or end,
    /// no error will be returned and token will not be consumed
    pub fn next_separator(&mut self) -> Result<(), Error> {
        if let Some(item) = self.clone().next() {
            //Check if next item is a data object
            let token = item?;
            match token {
                Token::ProgramDataSeparator => {
                    //Valid data object, consume and return
                    self.next();
                    Ok(())
                },
                Token::ProgramMessageUnitSeparator | Token::ProgramMessageTerminator => {
                    //Message separator or terminator is also ok but do not consume
                    Ok(())
                }
                Token::SuffixProgramData(_) => Err(Error::SuffixNotAllowed),
                _ => Err(Error::InvalidSeparator)
            }
        }else {
            //No more tokens, end of message
            Ok(())
        }

    }

    /// Attempts to consume a data object.
    /// If no data is found, none if returned if optional=true else Error:MissingParam.
    ///
    /// Note! Does not skip
    pub fn next_data(&mut self, optional: bool) -> Result<Option<Token<'a>>, Error> {
        //Try to read a data object
        if let Some(item) = self.clone().next() {
            //Check if next item is a data object
            let token = item?;
            match token {
                Token::CharacterProgramData(_) | Token::DecimalNumericProgramData(_) | Token::SuffixProgramData(_) |
                Token::NonDecimalNumericProgramData(_) | Token::StringProgramData(_) | Token::ArbitraryBlockData(_) => {
                    //Valid data object, consume and return
                    self.next();
                    Ok(Some(token))
                }
                _ => {
                    //Not a data object, return nothing
                    if optional {
                        Ok(None)
                    }else{
                        Err(Error::MissingParameter)
                    }
                }
            }
        }else {
            //No more tokens, return nothing
            if optional {
                Ok(None)
            }else{
                Err(Error::MissingParameter)
            }
        }
    }

    /// Attempt to read a decimal data object (either a DecimalProgramData or a CharacterProgramData).
    /// This function is usually OK'ed and unwrapped before calling a `to_<type>` or `.special()`
    ///
    /// # Arguments
    /// * `optional` - Object is optional, will return None if data is missing and optional, Err otherwise.
    /// * `suffix` - Function to use to parse suffix (if any). Takes the value and suffix slice.
    ///
    /// # Returns
    /// * Ok(NumericalProgramData) - If a NumericalProgramData is found. If a suffix follows, the value is modified with according to suffix.
    /// * Ok(CharacterProgramData) - If a CharacterProgramData is found. Typically a special value.
    ///
    ///
    pub fn next_decimal<F>(&mut self, optional: bool, suffix: F) -> Result<Option<Token<'a>>, Error>
        where F: FnOnce(f32, &[u8]) -> Result<f32, Error> {
        let val = self.next_data(optional)?;
        if let Some(t) = val {
            match t {

                Token::DecimalNumericProgramData(mut val) => {
                    //Check if next token is a suffix and reinterpret the value if so
                    let mut clone = self.clone();
                    if let Some(x) = clone.next() {
                        if let Token::SuffixProgramData(s) = x? {
                            //Found a suffix!
                            self.clone_from(&clone);

                            val = suffix(val, s)?;
                            return Ok(Some(Token::DecimalNumericProgramData(val)));
                        }
                    }
                    //No suffix, return as is
                    Ok(Some(t))
                },
                Token::CharacterProgramData(_) => {
                    Ok(Some(t))
                }
                _ => {
                    Err(Error::MissingParameter)
                }
            }
        }else{
            Ok(None)
        }

    }



    pub fn next_arb(&mut self) -> Result<&'a [u8], Error> {
        if let Some(tok) = self.next() {
            let val = tok?;
            match val {
                Token::ArbitraryBlockData(f) => Ok(f),
                Token::DecimalNumericProgramData(_) | Token::NonDecimalNumericProgramData(_) |
                Token::StringProgramData(_) | Token::CharacterProgramData(_) | Token::SuffixProgramData(_) => Err(Error::DataTypeError),
                _ => {
                    Err(Error::MissingParameter)
                }
            }
        }else{
            Err(Error::MissingParameter)
        }
    }


}

fn ascii_to_digit(digit: u8, radix: u8) -> Option<u32> {
    let lowercase = digit.to_ascii_lowercase();

    if digit.is_ascii_digit() && digit - b'0' < radix {
        Some((digit - b'0') as u32)
    } else if lowercase.is_ascii_alphabetic() && lowercase - b'a' < radix-10 {
        Some((lowercase - b'a' + 10) as u32)
    } else {
        None
    }
}

impl<'a> Tokenizer<'a> {
    pub fn from_str(buf: &'a [u8]) -> Self {
        Tokenizer {
            chars: buf.iter(),
            in_header: true,
            in_common: false,
            in_numeric: false
        }
    }

    pub fn empty() -> Self {
        Tokenizer {
            chars: b"".iter(),
            in_header: true,
            in_common: false,
            in_numeric: false
        }
    }

    /// <program mnemonic>
    /// See IEEE 488.2-1992 7.6.1
    /// Must start with a alphabetic character followed by alphanumeric and '_' characters.
    ///
    /// Returned errors:
    /// * ProgramMnemonicTooLong if suffix is longer than 12 characters
    fn read_mnemonic(&mut self) -> Result<Token<'a>, Error> {
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self.chars.clone().next().map_or(false, |ch| ch.is_ascii_alphanumeric() || *ch == b'_') {
            self.chars.next();
            len += 1;
            if len > 12 {
                return Err(Error::ProgramMnemonicTooLong);
            }
        }
        Ok(Token::ProgramMnemonic(&s[0..s.len() - self.chars.as_slice().len()]))
    }

    /// <CHARACTER PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.1
    /// Consists of a single <program mnemonic>
    ///
    /// Returned errors:
    /// * CharacterDataTooLong if suffix is longer than 12 characters
    fn read_character_data(&mut self) -> Result<Token<'a>, Error> {
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self.chars.clone().next().map_or(false, |ch| ch.is_ascii_alphanumeric() || *ch == b'_') {
            self.chars.next();
            len += 1;
            if len > 12 {
                return Err(Error::CharacterDataTooLong);
            }
        }
        let ret = Ok(Token::CharacterProgramData(&s[0..s.len() - self.chars.as_slice().len()]));

        // Skip to next separator
        self.skip_ws_to_separator(Error::InvalidCharacterData)?;
        ret
    }

    /// <DECIMAL NUMERIC PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.2
    ///
    /// TODO: lexical-core does not accept whitespace between exponent separator and exponent <mantissa>E <exponent>.
    fn read_numeric_data(&mut self) -> Result<Token<'a>, Error> {
        /* Read mantissa */
        let (f, len) = lexical_core::parse_partial::<f32>(self.chars.as_slice()).map_err(|_|Error::NumericDataError)?;
        for _ in 0..len { self.chars.next(); };
        self.skip_ws();

        Ok(Token::DecimalNumericProgramData(f))
    }

    /// <SUFFIX PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.3
    /// Reads a suffix and returns it as a string if successful, otherwise it returns an error.
    /// TODO: Syntax check suffix
    ///
    /// Returned errors:
    /// * SuffixTooLong if suffix is longer than 12 characters
    fn read_suffix_data(&mut self) -> Result<Token<'a>, Error> {
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self.chars.clone().next().map_or(false, |ch| ch.is_ascii_alphanumeric() || *ch == b'-' || *ch == b'/' || *ch == b'.') {
            self.chars.next();
            len += 1;
            if len > 12 {
                return Err(Error::SuffixTooLong);
            }
        }

        let ret = Ok(Token::SuffixProgramData(&s[0..s.len() - self.chars.as_slice().len()]));
        // Skip to next separator
        self.skip_ws_to_separator(Error::InvalidSuffix)?;
        ret
    }

    /// <NONDECIMAL NUMERIC PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.4
    /// Reads a non-decimal numeric
    ///
    /// Returned errors:
    fn read_nondecimal_data(&mut self, radix: u8) -> Result<Token<'a>, Error> {
        let radixi = match radix {
            b'H' | b'h' => 16u32,
            b'Q' | b'q' => 8u32,
            b'B' | b'b' => 2u32,
            _ => return {
                Err(Error::NumericDataError)
            }
        };

        let mut acc = 0u32;
        let mut any = false;
        while self.chars.clone().next().map_or(false, |ch| ch.is_ascii_alphanumeric()) {

            let c = self.chars.next().unwrap();
            acc = ascii_to_digit(*c, radixi as u8).ok_or(Error::InvalidCharacterInNumber)? + (acc*radixi);
            any = true;
        }
        if !any {
            return Err(Error::NumericDataError)
        }
        let ret = Ok(Token::NonDecimalNumericProgramData(acc));
        // Skip to next separator
        self.skip_ws_to_separator(Error::InvalidSeparator)?;
        ret
    }

    /// <STRING PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.5
    ///
    fn read_string_data(&mut self, x: u8, ascii: bool) -> Result<Token<'a>, Error> {
        self.chars.next();//Consume first
        let s = self.chars.as_slice();
        while self.chars.clone().next().map_or(false, |ch| *ch != x) {
            //Only ASCII allowed
            let x = self.chars.next().unwrap();
            if ascii && !x.is_ascii() {
                return Err(Error::InvalidCharacter)
            }
        }
        if self.chars.next().is_none() {
            return Err(Error::InvalidStringData)
        }
        let ret = Ok(Token::StringProgramData(&s[0..s.len() - self.chars.as_slice().len() - 1]));
        // Skip to next separator
        self.skip_ws_to_separator(Error::InvalidSeparator)?;
        ret

    }

    /// <ARBITRARY DATA PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.6
    ///
    fn read_arbitrary_data(&mut self, format: u8) -> Result<Token<'a>, Error> {
        if let Some(len) = ascii_to_digit(format, 10) {
            if len == 0 {
                //Take rest of string
                let rest = self.chars.as_slice();
                let u8str = rest.get(0..rest.len()-1).ok_or(Error::InvalidBlockData)?;
                for _ in u8str {
                    self.chars.next();
                }
                //Indefinite block data must be terminated with NL before END
                if *self.chars.next().unwrap() != b'\n'{
                    return Err(Error::InvalidBlockData);
                }
                return Ok(Token::ArbitraryBlockData(u8str));
            }

            let mut i = len;
            let mut payload_len = 0usize;

            while self.chars.clone().next().map_or(false, |ch| ch.is_ascii_digit() && i > 0) {
                let c = self.chars.next().unwrap();
                payload_len = payload_len*10 + ascii_to_digit(*c, 10).unwrap() as usize;
                i -= 1;
            }
            //Not all payload length digits were consumed (i.e. string end or not a digit)
            if i > 0 || payload_len > self.chars.as_slice().len(){
                return Err(Error::InvalidBlockData);
            }

            let u8str = self.chars.as_slice().get(0..payload_len).ok_or(Error::InvalidBlockData)?;
            for _ in 0..payload_len {
                self.chars.next();
            }


            let ret = Ok(Token::ArbitraryBlockData(u8str));
            // Skip to next separator
            self.skip_ws_to_separator(Error::InvalidSeparator)?;
            ret

        }else {
            Err(Error::InvalidBlockData)
        }
    }

    /// <ARBITRARY DATA PROGRAM DATA>
    /// Non standard custom arbitrary data block format #s"..."/#s'...'
    /// The parser will automatically check and convert the data to a UTF8 string, emitting
    /// a InvalidBlockData error if the string is not valid UTF8.
    fn read_utf8_data(&mut self, _format: u8) -> Result<Token<'a>, Error> {

        if let Some(x) = self.chars.clone().next() {
            if *x != b'"' && *x != b'\'' {
                return Err(Error::NoError);
            }
            if let Token::StringProgramData(s) = self.read_string_data(*x, false)? {
                if let Ok(u) = str::from_utf8(s) {
                    Ok(Token::Utf8BlockData(u))
                }else{
                    Err(Error::InvalidBlockData)
                }


            }else{
                Err(Error::InvalidBlockData)
            }


        }else {
            Err(Error::InvalidBlockData)
        }
    }

    /// <EXPRESSION PROGRAM DATA>
    /// See IEEE 488.2-1992 7.7.7
    ///
    fn read_expression_data(&mut self) -> Result<Token<'a>, Error> {
        unimplemented!();
    }

    fn skip_ws(&mut self){
        while self.chars.clone().next().map_or(false, |ch| ch.is_ascii_whitespace()) {
            self.chars.next();
        }
    }

    fn skip_ws_to_separator(&mut self, error: Error) -> Result<(), Error> {
        self.skip_ws();
        if let Some(c) = self.chars.clone().next() {
            if *c != b',' && *c != b';' && *c != b'\n' {
                return Err(error);
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod test_parse {
    use crate::tokenizer::{Tokenizer, Token};
    use crate::error::Error;

    extern crate std;

    use std::fmt;

    impl fmt::Debug for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({})", self.clone() as i32)
        }
    }

    #[test]
    fn test_split_mnemonic(){
        assert_eq!(Token::mnemonic_split_index(b"TRIGger54"), Some((b"TRIGger".as_ref(), b"54".as_ref())));
        assert_eq!(Token::mnemonic_split_index(b"T123r54"), Some((b"T123r".as_ref(), b"54".as_ref())));
        assert_eq!(Token::mnemonic_split_index(b"TRIGger"), None);
        assert_eq!(Token::mnemonic_split_index(b""), None);
    }

    #[test]
    fn test_compare_mnemonic(){
        //Should return true
        assert!(Token::mnemonic_compare(b"TRIGger", b"trigger"));
        assert!(Token::mnemonic_compare(b"TRIGger", b"trig"));
        assert!(Token::mnemonic_compare(b"TRIGger", b"TRIGGER"));
        assert!(Token::mnemonic_compare(b"TRIGger", b"TRIG"));
        //Should return false
        assert!(!Token::mnemonic_compare(b"TRIGger", b"TRIGe"));
        assert!(!Token::mnemonic_compare(b"TRIGger", b"triggeristoodamnlong"));
        assert!(!Token::mnemonic_compare(b"TRIGger", b"tri"));
    }

    #[test]
    fn test_eq_mnemonic(){
        //
        assert!(Token::ProgramMnemonic(b"trigger").eq_mnemonic(b"TRIGger1"));
        assert!(Token::ProgramMnemonic(b"trig").eq_mnemonic(b"TRIGger1"));
        assert!(Token::ProgramMnemonic(b"trigger1").eq_mnemonic(b"TRIGger1"));
        assert!(Token::ProgramMnemonic(b"trig1").eq_mnemonic(b"TRIGger1"));
        assert!(!Token::ProgramMnemonic(b"trigger2").eq_mnemonic(b"TRIGger1"));
        assert!(!Token::ProgramMnemonic(b"trig2").eq_mnemonic(b"TRIGger1"));

        assert!(Token::ProgramMnemonic(b"trigger2").eq_mnemonic(b"TRIGger2"));
        assert!(Token::ProgramMnemonic(b"trig2").eq_mnemonic(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trigger").eq_mnemonic(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trig").eq_mnemonic(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trigger1").eq_mnemonic(b"TRIGger2"));
        assert!(!Token::ProgramMnemonic(b"trig1").eq_mnemonic(b"TRIGger2"));
    }

    #[test]
    fn test_read_character_data(){
        assert_eq!(Tokenizer::from_str(b"CHARacter4 , pperg").read_character_data(),
                   Ok(Token::CharacterProgramData(b"CHARacter4")));
        assert_eq!(Tokenizer::from_str(b"CHARacterIsTooLong").read_character_data(),
                   Err(Error::CharacterDataTooLong));
        assert_eq!(Tokenizer::from_str(b"Character Invalid").read_character_data(),
                   Err(Error::InvalidCharacterData));
    }

    #[test]
    fn test_read_numeric_data(){

        //TODO: FIX EXPONENTS!

        assert_eq!(Tokenizer::from_str(b"25").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(25f32));

        assert_eq!(Tokenizer::from_str(b"-10.").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(-10f32));

        assert_eq!(Tokenizer::from_str(b".2").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(0.2f32));

        assert_eq!(Tokenizer::from_str(b"1.E5").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(1e5f32));

        assert_eq!(Tokenizer::from_str(b"-25e5").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(-25e5f32));

        assert_eq!(Tokenizer::from_str(b"25E-2").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(0.25f32));

        assert_eq!(Tokenizer::from_str(b".1E2").read_numeric_data().unwrap(),
                   Token::DecimalNumericProgramData(10f32));

    }

    #[test]
    fn test_read_suffix_data(){
        assert_eq!(Tokenizer::from_str(b"MOHM").read_suffix_data().unwrap(),
                   Token::SuffixProgramData(b"MOHM"));

    }

    #[test]
    fn test_read_numeric_suffix_data(){
        //let mut tokenizer = Tokenizer::from_str(b"header 25 KHZ , 12.7E6 KOHM.M/S-2");
//        assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramMnemonic(b"header"))));
//        assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramHeaderSeparator)));
//        assert_eq!(tokenizer.next_u8(), Ok(25u8));
//        assert_eq!(tokenizer.next(), Some(Ok(Token::SuffixProgramData(b"KHZ"))));
//        assert_eq!(tokenizer.next(), Some(Ok(Token::ProgramDataSeparator)));
//        assert_eq!(tokenizer.next_f32(), Ok(12.7e6f32));
//        assert_eq!(tokenizer.next(), Some(Ok(Token::SuffixProgramData(b"KOHM.M/S-2"))));

    }

    #[test]
    fn test_read_string_data(){

        assert_eq!(Tokenizer::from_str(b"\"MOHM\",  gui").read_string_data(b'"', true),
                   Ok(Token::StringProgramData(b"MOHM")));

    }

    #[test]
    fn test_read_arb_data(){
        assert_eq!(Tokenizer::from_str(b"02\x01\x02,").read_arbitrary_data(b'2'),
                   Ok(Token::ArbitraryBlockData(&[1,2])));

    }

}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.chars.clone().next()?;
        match x {
            /* Common command prefix */
            b'*' => {
                self.in_common = true;
                self.chars.next();
                if let Some(x) = self.chars.clone().next() {
                    if !x.is_ascii_alphabetic() {
                        return Some(Err(Error::CommandHeaderError))
                    }
                }

                /* Not allowed outside header and strings */
                Some(Ok(Token::HeaderCommonPrefix))
            }
            /* Header mnemonic separator/prefix */
            b':' => {
                self.chars.next();
                //Only one separator is allowed
                if let Some(x) = self.chars.clone().next() {
                    if !x.is_ascii_alphabetic() {
                        return Some(Err(Error::InvalidSeparator))
                    }
                }
                /* Not allowed outside header and strings */
                if !self.in_header || self.in_common {
                    Some(Err(Error::InvalidSeparator))
                }else{
                    Some(Ok(Token::HeaderMnemonicSeparator))
                }
            }
            /* Header query suffix */
            b'?' => {
                self.chars.next();
                //Next character after query must be a space, unit separator or <END>
                if let Some(x) = self.chars.clone().next() {
                    if !x.is_ascii_whitespace() && *x != b';' {
                        return Some(Err(Error::InvalidSeparator))
                    }
                }
                if !self.in_header {
                    Some(Err(Error::InvalidSeparator))
                }else{
                    self.in_header = false;
                    Some(Ok(Token::HeaderQuerySuffix))
                }

            }
            /* Program unit separator */
            b';' => {
                self.chars.next();
                self.skip_ws();
                self.in_header = true;
                self.in_common = false;
                Some(Ok(Token::ProgramMessageUnitSeparator))
            }
            /* Message terminator */
            //END is implied.
            // Parser should reset itself and parse next message as a new message.
            b'\n' => {
                self.chars.next();
                Some(Ok(Token::ProgramMessageTerminator))
            }
            /* Data separator*/
            b',' => {
                self.chars.next();
                if self.in_header {
                    Some(Err(Error::HeaderSeparatorError))
                }else{
                    self.in_numeric = false;
                    self.skip_ws();
                    if let Some(c) = self.chars.clone().next() {
                        if *c == b',' || *c == b';' || *c == b'\n' {
                            return Some(Err(Error::SyntaxError));
                        }
                    }
                    Some(Ok(Token::ProgramDataSeparator))

                }
            }
            /* Whitespace */
            // Separates the header from arguments
            x if x.is_ascii_whitespace() => {
                self.skip_ws();
                /* Header ends */
                self.in_header = false;
                Some(Ok(Token::ProgramHeaderSeparator))
            }
            /* Alphabetic */
            x if x.is_ascii_alphabetic() => {
                /* If still parsing header, it's an mnemonic, else character data */
                if self.in_header {
                    Some(self.read_mnemonic())
                }else if self.in_numeric {
                    Some(self.read_suffix_data())
                }else{
                    Some(self.read_character_data())
                }
            }
            /* Suffix starting with '/' */
            b'/' => {
                if self.in_header {
                    Some(Err(Error::InvalidSeparator))
                }else{
                    Some(self.read_suffix_data())
                }
            }
            /* Number */
            x if x.is_ascii_digit() || *x == b'-' || *x == b'+' || *x == b'.' => {
                if self.in_header {
                    Some(Err(Error::CommandHeaderError))
                }else{
                    self.in_numeric = true;
                    Some(self.read_numeric_data())
                }
            }
            /* Arb. block or non-decimal data */
            b'#' => {
                self.chars.next();
                if self.in_header {
                    Some(Err(Error::CommandHeaderError))
                }else{
                    if let Some(x) = self.chars.next() {
                        Some(match x {
                            /* Arbitrary block */
                            x if *x == b's' => {
                                self.read_utf8_data(*x)
                            }
                            x if x.is_ascii_digit() => {
                                self.read_arbitrary_data(*x)
                            }
                            /*Non-decimal numeric*/
                            _ => self.read_nondecimal_data(*x)
                        })
                    }else{
                        Some(Err(Error::BlockDataError))
                    }
                }
            }
            /* String */
            x if *x == b'\'' || *x == b'"' => {
                if self.in_header {
                    Some(Err(Error::CommandHeaderError))
                }else{
                    Some(self.read_string_data(*x, true))
                }
            }
            /* Unknown/unexpected */
            _ => {
                let x = self.chars.next().unwrap();
                if x.is_ascii() {
                    Some(Err(Error::SyntaxError))
                }else{
                    Some(Err(Error::InvalidCharacter))
                }
            }
        }
    }
}

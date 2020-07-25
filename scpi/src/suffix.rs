use crate::error::{Error, ErrorCode};
use crate::suffix::SuffixUnitElement::Watt;
use core::f32::consts::PI;
use core::slice::Iter;

/// Standard suffix units
///
#[derive(PartialEq, Copy, Clone, Debug, ScpiUnit)]
pub enum SuffixUnitElement {
    ///# Absorbed dose
    #[unit(suffix = b"GY", multiplier = 1.0)]
    Gray, //GY

    ///Radioactive activity
    #[unit(suffix = b"BQ", multiplier = 1.0)]
    Becquerel, //BQ
    ///Amount of substance
    #[unit(suffix = b"MOL", multiplier = 1.0)]
    #[unit(suffix = b"MMOL", multiplier = 1.0e-3)]
    #[unit(suffix = b"UMOL", multiplier = 1.0e-6)]
    Mole, //MOL
    ///Angle
    #[unit(suffix = b"DEG", multiplier = 1.0)]
    Degree, //DEG
    #[unit(suffix = b"GON", multiplier = 1.0)]
    Grade, //GON
    #[unit(suffix = b"MNT", multiplier = 1.0)]
    AMinute, //MNT
    #[unit(suffix = b"RAD", multiplier = 1.0)]
    Radian, //RAD
    #[unit(suffix = b"REV", multiplier = 1.0)]
    Revolution, //REV
    #[unit(suffix = b"SR", multiplier = 1.0)]
    Steradian, //SR
    ///Dose equivalent
    #[unit(suffix = b"SV", multiplier = 1.0)]
    Sievert, //SV
    ///Inductance
    #[unit(suffix = b"H", multiplier = 1.0)]
    #[unit(suffix = b"MH", multiplier = 1.0e-3)]
    #[unit(suffix = b"UH", multiplier = 1.0e-6)]
    #[unit(suffix = b"NH", multiplier = 1.0e-9)]
    Henry, //H
    ///Capacitance
    #[unit(suffix = b"F", multiplier = 1.0)]
    #[unit(suffix = b"MF", multiplier = 1.0e-3)]
    #[unit(suffix = b"UF", multiplier = 1.0e-6)]
    #[unit(suffix = b"NF", multiplier = 1.0e-9)]
    #[unit(suffix = b"PF", multiplier = 1.0e-12)]
    Farad, //F
    ///Electric charge
    #[unit(suffix = b"MAC", multiplier = 1.0e6)]
    #[unit(suffix = b"KC", multiplier = 1.0e3)]
    #[unit(suffix = b"C", multiplier = 1.0)]
    #[unit(suffix = b"MC", multiplier = 1.0e-3)]
    #[unit(suffix = b"UC", multiplier = 1.0e-6)]
    Coulomb, //C
    ///Electric Conductivity
    #[unit(suffix = b"KSIE", multiplier = 1.0e3)]
    #[unit(suffix = b"SIE", multiplier = 1.0)]
    #[unit(suffix = b"MSIE", multiplier = 1.0e-3)]
    #[unit(suffix = b"USIE", multiplier = 1.0e-6)]
    Siemens, //SIE
    ///Current
    #[unit(suffix = b"KA", multiplier = 1.0e3)]
    #[unit(suffix = b"A", multiplier = 1.0)]
    #[unit(suffix = b"MA", multiplier = 1.0e-3)]
    #[unit(suffix = b"UA", multiplier = 1.0e-6)]
    #[unit(suffix = b"NA", multiplier = 1.0e-12)]
    #[unit(suffix = b"PA", multiplier = 1.0e-15)]
    Ampere, //A
    ///Voltage
    #[unit(suffix = b"MAV", multiplier = 1.0e6)]
    #[unit(suffix = b"KV", multiplier = 1.0e3)]
    #[unit(suffix = b"V", multiplier = 1.0)]
    #[unit(suffix = b"MV", multiplier = 1.0e-3)]
    #[unit(suffix = b"UV", multiplier = 1.0e-6)]
    #[unit(suffix = b"NV", multiplier = 1.0e-12)]
    Volt, //V
    /// Electric resistance
    #[unit(suffix = b"GOHM", multiplier = 1.0e9)]
    #[unit(suffix = b"MOHM", multiplier = 1.0e6)]
    #[unit(suffix = b"KOHM", multiplier = 1.0e3)]
    #[unit(suffix = b"OHM", multiplier = 1.0)]
    #[unit(suffix = b"UOHM", multiplier = 1.0e-6)]
    Ohm, //OHM
    ///Energy
    #[unit(suffix = b"TEV", multiplier = 1.0e12)]
    #[unit(suffix = b"GEV", multiplier = 1.0e9)]
    #[unit(suffix = b"MAEV", multiplier = 1.0e6)]
    #[unit(suffix = b"KEV", multiplier = 1.0e3)]
    #[unit(suffix = b"EV", multiplier = 1.0)]
    ElectronVolt, //EV
    #[unit(suffix = b"GJ", multiplier = 1.0e9)]
    #[unit(suffix = b"MAJ", multiplier = 1.0e6)]
    #[unit(suffix = b"KJ", multiplier = 1.0e3)]
    #[unit(suffix = b"J", multiplier = 1.0)]
    #[unit(suffix = b"MJ", multiplier = 1.0e-3)]
    #[unit(suffix = b"UJ", multiplier = 1.0e-6)]
    Joule, //J
    ///Force
    #[unit(suffix = b"GN", multiplier = 1.0e9)]
    #[unit(suffix = b"MAN", multiplier = 1.0e6)]
    #[unit(suffix = b"KN", multiplier = 1.0e3)]
    #[unit(suffix = b"N", multiplier = 1.0)]
    #[unit(suffix = b"MN", multiplier = 1.0e-3)]
    #[unit(suffix = b"UN", multiplier = 1.0e-6)]
    Newton, //N
    ///Frequency
    #[unit(suffix = b"GHZ", multiplier = 1.0e9)]
    #[unit(suffix = b"MHZ", multiplier = 1.0e6)]
    #[unit(suffix = b"KHZ", multiplier = 1.0e3)]
    #[unit(suffix = b"HZ", multiplier = 1.0e0)]
    Hertz, //HZ
    ///Illuminance
    #[unit(suffix = b"LX", multiplier = 1.0)]
    Lux, //LX
    ///Length
    #[unit(suffix = b"ASU", multiplier = 1.0)]
    AstronomicUnit, //ASU
    #[unit(suffix = b"IN", multiplier = 1.0)]
    #[unit(suffix = b"MIL", multiplier = 1.0e-3)]
    Inch, //IN
    #[unit(suffix = b"FT", multiplier = 1.0)]
    Foot, //FT
    #[unit(suffix = b"KM", multiplier = 1.0e3)]
    #[unit(suffix = b"M", multiplier = 1.0)]
    #[unit(suffix = b"MM", multiplier = 1.0e-3)]
    #[unit(suffix = b"UM", multiplier = 1.0e-6)]
    #[unit(suffix = b"NM", multiplier = 1.0e-9)]
    Meter, //M
    #[unit(suffix = b"PRS", multiplier = 1.0)]
    Parsec, //PRS
    #[unit(suffix = b"MI", multiplier = 1.0)]
    Mile, //MI
    #[unit(suffix = b"NIM", multiplier = 1.0)]
    NauticalMile, //NIM
    ///Luminus flux
    #[unit(suffix = b"LM", multiplier = 1.0)]
    Lumen, //LM
    ///Luminus intensity
    #[unit(suffix = b"CD", multiplier = 1.0)]
    Candela, //CD
    ///Magnetic flux
    #[unit(suffix = b"WB", multiplier = 1.0)]
    Weber, //WB
    ///Magnetic field strength
    #[unit(suffix = b"T", multiplier = 1.0)]
    Tesla, //T
    ///Mass
    #[unit(suffix = b"U", multiplier = 1.0)]
    AtomicMass, //U
    #[unit(suffix = b"MAG", multiplier = 1.0e3)]
    #[unit(suffix = b"KG", multiplier = 1.0)]
    #[unit(suffix = b"G", multiplier = 1.0e-3)]
    #[unit(suffix = b"MG", multiplier = 1.0e-6)]
    #[unit(suffix = b"UG", multiplier = 1.0e-9)]
    Gram, //G
    #[unit(suffix = b"TNE", multiplier = 1.0)]
    Tonne, //TNE
    ///Power
    #[unit(suffix = b"GW", multiplier = 1.0e9)]
    #[unit(suffix = b"MAW", multiplier = 1.0e6)]
    #[unit(suffix = b"KW", multiplier = 1.0e3)]
    #[unit(suffix = b"W", multiplier = 1.0)]
    #[unit(suffix = b"MW", multiplier = 1.0e-3)]
    Watt, //W
    ///Pressure
    #[unit(suffix = b"ATM", multiplier = 1.0)]
    Atmosphere, //ATM
    #[unit(suffix = b"INHG", multiplier = 1.0)]
    InchMercury, //INHG
    #[unit(suffix = b"MMHG", multiplier = 1.0)]
    MmMercury, //MMHG
    #[cfg_attr(not(feature = "strict"), unit(suffix = b"MAPAL", multiplier = 1.0e6))]
    #[unit(suffix = b"KPAL", multiplier = 1.0e3)]
    #[unit(suffix = b"PAL", multiplier = 1.0)]
    Pascal, //PAL
    #[unit(suffix = b"TORR", multiplier = 1.0)]
    #[cfg_attr(not(feature = "strict"), unit(suffix = b"MTORR", multiplier = 1.0e-3))]
    Tort, //TORR
    #[unit(suffix = b"BAR", multiplier = 1.0)]
    Bar, //BAR
    ///Ratio
    #[unit(suffix = b"DB", multiplier = 1.0)]
    Decibel, //DB
    #[unit(suffix = b"PCT", multiplier = 1.0)]
    Percent, //PCT
    #[unit(suffix = b"PPM", multiplier = 1.0)]
    PartPerMillion, //PPM
    ///Temperature
    #[unit(suffix = b"CEL", multiplier = 1.0)]
    Celsius, //CEL
    #[unit(suffix = b"FAR", multiplier = 1.0)]
    Fahrenheit, //FAR
    #[unit(suffix = b"K", multiplier = 1.0)]
    Kelvin, //K
    ///Time
    #[unit(suffix = b"S", multiplier = 1.0)]
    Second, //S
    #[unit(suffix = b"D", multiplier = 1.0)]
    Day, //D
    #[unit(suffix = b"HR", multiplier = 1.0)]
    Hour, //HR
    #[unit(suffix = b"MIN", multiplier = 1.0)]
    Minute, //MIN
    #[unit(suffix = b"ANN", multiplier = 1.0)]
    Year, //ANN
    ///Viscosity
    #[unit(suffix = b"ST", multiplier = 1.0)]
    Strokes, //ST
    #[unit(suffix = b"P", multiplier = 1.0)]
    Poise, //P
    ///Volume
    #[unit(suffix = b"L", multiplier = 1.0)]
    Liter, //L
}

/// Error which may be emitted by suffix parser and conversion.
///
#[derive(PartialEq, Copy, Clone, Debug, ScpiError)]
pub enum SuffixError {
    /// Suffix has invalid syntax (example `S//M2` or `S.2`)
    #[error(code=-1, message=b"Syntax")]
    Syntax,
    /// Suffix element has a exponent of zero
    #[error(code=-2, message=b"Zero dimension")]
    ZeroExponent,
    /// Suffix is not recognized
    #[error(code=-3, message=b"Unknown suffix")]
    Unknown,
    /// Conversion only works to certain basic units (eg Second, not Day, etc)
    #[error(code=-4, message=b"Not a base unit")]
    NotABaseUnit,
    /// Trying to convert between incompatible quantities (example: Farad -> Meter)
    #[error(code=-5, message=b"Incompatible quantity")]
    IncompatibleQuantity,
    /// Trying to convert between different dimensions (example Meter^3 -> Meter
    #[error(code=-6, message=b"Incompatibe dimension")]
    IncompatibleDimension,
}

impl From<SuffixError> for Error {
    #[cfg(not(feature = "extended-error"))]
    fn from(_se: SuffixError) -> Self {
        Error::new(ErrorCode::SuffixError)
    }

    #[cfg(feature = "extended-error")]
    fn from(se: SuffixError) -> Self {
        Error::extended(ErrorCode::SuffixError, se.get_message())
    }
}

/// A suffix token
///
#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
    /// A `.`, separates suffix elements
    Separator,
    /// A `/`, separates suffix elements and inverts exponent
    Per,
    /// A element, consists of a single suffix element and exponent `<ELEMENT>[[-]<EXPONENT>]` (example `KHZ` or `S-2`)
    Element(&'a [u8], i8),
}

#[derive(Clone)]
pub struct Tokenizer<'a> {
    chars: Iter<'a, u8>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a [u8]) -> Self {
        Tokenizer { chars: s.iter() }
    }

    fn read_element(&mut self) -> Result<Token<'a>, SuffixError> {
        //Read suffix element
        let s = self.chars.as_slice();
        let mut len = 0u8;
        while self
            .chars
            .clone()
            .next()
            .map_or(false, |ch| ch.is_ascii_alphabetic())
        {
            self.chars.next();
            len += 1;
            if len > 6 {
                return Err(SuffixError::Syntax);
            }
        }
        let element = &s[0..s.len() - self.chars.as_slice().len()];
        if element.is_empty() {
            return Err(SuffixError::Syntax);
        }

        //Try to read an exponent
        let (exponent, len) = if self
            .chars
            .clone()
            .next()
            .map_or(false, |x| x.is_ascii_digit() || *x == b'-' || *x == b'+')
        {
            lexical_core::parse_partial::<i8>(&self.chars.as_slice())
                .map_err(|_| SuffixError::Syntax)
        } else {
            Ok((1i8, 0usize))
        }?;

        if len > 0 {
            self.chars.nth(len - 1).unwrap();
        }

        //Exponent is 0?
        if exponent == 0 {
            Err(SuffixError::ZeroExponent)
        } else {
            Ok(Token::Element(element, exponent))
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, SuffixError>;

    /// Get next suffix token if available
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.chars.clone().next()?;
        match x {
            /* Per */
            b'/' => {
                self.chars.next().unwrap();
                Some(Ok(Token::Per))
            }
            b'.' => {
                self.chars.next().unwrap();
                Some(Ok(Token::Separator))
            }
            x if x.is_ascii_alphabetic() => Some(self.read_element()),
            _ => Some(Err(SuffixError::Syntax)),
        }
    }
}

impl SuffixUnitElement {
    /// Convert value to a new suffix element
    ///
    /// # Arguments
    ///  * `to` - Unit to convert to. Not all units are accepted.
    ///  * `value` - Value to convert
    ///
    /// # Returns
    ///  * `Ok(f32)` if conversion is successful
    ///  * `Err(SuffixError::IncompatibleQuantity)` if the `to` unit does not represent the same quantity
    ///  * `Err(SuffixError::NotABaseUnit)` if the `to` unit is not a basic unit
    ///
    /// # Example
    ///
    /// ```
    /// use scpi::suffix::SuffixUnitElement;
    /// let suffix = SuffixUnitElement::Hour;
    ///
    /// // Convert "1.5 <suffix>" to seconds
    /// let seconds = suffix.convert(SuffixUnitElement::Second, 1.5f32);
    ///
    /// // If the suffix does not represent the same quantity (example "1.5 OHM") an error will be returned
    ///
    /// ```
    ///
    pub fn convert(self, to: SuffixUnitElement, value: f32) -> Result<f32, SuffixError> {
        // Convert to self is simple
        if to == self {
            return Ok(value);
        }
        //If not, manually convert...
        //TODO: Automate with #[derive()]?
        let ret = match to {
            //Angle
            SuffixUnitElement::Radian => match self {
                SuffixUnitElement::Degree => value * PI / 180f32,
                SuffixUnitElement::Grade => value * PI / 200f32,
                SuffixUnitElement::AMinute => value * PI / 180f32 / 60f32,
                SuffixUnitElement::Second => value * PI / 180f32 / 3600f32,
                SuffixUnitElement::Revolution => value * PI * 2f32,
                _ => Err(SuffixError::IncompatibleQuantity)?,
            },
            //Time
            SuffixUnitElement::Second => match self {
                SuffixUnitElement::Minute => value * 60f32,
                SuffixUnitElement::Hour => value * 3600f32,
                SuffixUnitElement::Day => value * 24f32 * 3600f32,
                _ => Err(SuffixError::IncompatibleQuantity)?,
            },
            _ => Err(SuffixError::NotABaseUnit)?,
        };

        Ok(ret)
    }

    /// Convert a suffix to its unit and scale value.
    /// Will also convert any DB<reference unit> to its reference unit and calculate its normal value
    /// (**Note: `DBM` is an exception and is returned as DbMilliWatt, `DBMW` is handled as ordinary**)
    ///
    /// # Arguments
    /// * `str` - Suffix string, example `PCT` or `DBUV`.
    /// * `val` - Value to scale
    pub fn from_str(str: &[u8], val: f32) -> Result<(SuffixUnitElement, f32), SuffixError> {
        #[allow(unused_imports)]
        use crate::lexical_core::Float;
        // If suffix start with "DB", try to parse it as a decibel
        if str
            .get(..2)
            .map_or(false, |s| s.eq_ignore_ascii_case(b"DB"))
        {
            // DBM is a special alias of DBMW
            let (unit, mul): (SuffixUnitElement, f32) = if str.eq_ignore_ascii_case(b"DBM") {
                (Watt, 1e-3)
            } else {
                Self::from_suffix(&str[2..])?
            };

            // Power units have a ratio of 10
            // Quantitative units, 20
            let ratio = match unit {
                SuffixUnitElement::Watt => 10f32,
                _ => 20f32,
            };

            Ok((unit, 10f32.round().powf(val / ratio) * mul))
        } else {
            let (unit, mul) = Self::from_suffix(&str)?;

            Ok((unit, mul * val))
        }
    }
}

#[cfg(test)]
mod test_suffix {
    use crate::error::{Error, ErrorCode};
    use crate::suffix::SuffixError;
    use crate::suffix::{SuffixUnitElement, Token, Tokenizer};

    extern crate std;

    #[test]
    fn test_from_str() {
        // Ok
        let volt = SuffixUnitElement::from_str(b"V", 1.0);
        assert_eq!(volt, Ok((SuffixUnitElement::Volt, 1.0)));
        let millivolt = SuffixUnitElement::from_str(b"mV", 1.0);
        assert_eq!(millivolt, Ok((SuffixUnitElement::Volt, 1.0e-3)));
        let dbvolt = SuffixUnitElement::from_str(b"dBV", 40.0);
        assert_eq!(dbvolt, Ok((SuffixUnitElement::Volt, 100.0)));
        let dbwatt = SuffixUnitElement::from_str(b"dBW", 40.0);
        assert_eq!(dbwatt, Ok((SuffixUnitElement::Watt, 10000.0)));
        let dbm = SuffixUnitElement::from_str(b"DBM", 0.0);
        assert_eq!(dbm, Ok((SuffixUnitElement::Watt, 1e-3)));

        //Not ok
        let invalid = SuffixUnitElement::from_str(b"X", 1.0);
        assert_eq!(invalid, Err(SuffixError::Unknown));
        let invaliddb = SuffixUnitElement::from_str(b"DBX", 1.0);
        assert_eq!(invaliddb, Err(SuffixError::Unknown));
    }

    #[test]
    fn test_get_code() {
        assert_eq!(ErrorCode::CommandError.get_code(), -100);
    }

    #[test]
    fn test_get_message() {
        assert_eq!(ErrorCode::CommandError.get_message(), b"Command error");
    }

    #[test]
    fn test_from_code() {
        assert_eq!(ErrorCode::get_error(-100), Some(ErrorCode::CommandError));
    }

    #[test]
    fn test_read_element() {
        //Ok
        assert_eq!(
            Tokenizer::new(b"M").read_element(),
            Ok(Token::Element(b"M", 1i8))
        );
        assert_eq!(
            Tokenizer::new(b"M2").read_element(),
            Ok(Token::Element(b"M", 2i8))
        );
        assert_eq!(
            Tokenizer::new(b"M+2").read_element(),
            Ok(Token::Element(b"M", 2i8))
        );
        assert_eq!(
            Tokenizer::new(b"M-2").read_element(),
            Ok(Token::Element(b"M", -2i8))
        );
        //
        assert_eq!(
            Tokenizer::new(b"2").read_element(),
            Err(SuffixError::Syntax)
        );
        assert_eq!(
            Tokenizer::new(b"M256").read_element(),
            Err(SuffixError::Syntax)
        );
        assert_eq!(
            Tokenizer::new(b"&2").read_element(),
            Err(SuffixError::Syntax)
        );
        assert_eq!(
            Tokenizer::new(b"S0").read_element(),
            Err(SuffixError::ZeroExponent)
        );
    }

    #[test]
    fn test_suffix_tokenizer() {
        let mut tok = Tokenizer::new(b"M.S-2/M3");
        assert_eq!(tok.next(), Some(Ok(Token::Element(b"M", 1i8))));
        assert_eq!(tok.next(), Some(Ok(Token::Separator)));
        assert_eq!(tok.next(), Some(Ok(Token::Element(b"S", -2i8))));
        assert_eq!(tok.next(), Some(Ok(Token::Per)));
        assert_eq!(tok.next(), Some(Ok(Token::Element(b"M", 3i8))));
        assert_eq!(tok.next(), None);

        let mut tok2 = Tokenizer::new(b"_M.S-2/M3");
        assert_eq!(tok2.next(), Some(Err(SuffixError::Syntax)));
    }

    #[cfg(not(feature = "extended-error"))]
    #[test]
    fn test_suffix_error_into_error() {
        assert_eq!(
            Error::from(SuffixError::Syntax),
            Error::new(ErrorCode::SuffixError)
        )
    }

    #[cfg(feature = "extended-error")]
    #[test]
    fn test_suffix_error_into_error() {
        assert_eq!(
            Error::from(SuffixError::Syntax),
            Error::extended(ErrorCode::SuffixError, b"Syntax")
        )
    }
}

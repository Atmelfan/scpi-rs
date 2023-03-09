//! Handle decimal-data suffixes

#[allow(unused_imports)]
use {
    crate::{
        error::{Error, ErrorCode},
        parser::{
            response::{Formatter, ResponseData},
            tokenizer::Token,
        },
    },
    core::convert::{TryFrom, TryInto},
};

//use crate::scpi1999::numeric::NumericValueDefaults;

/// A logarithmic or linear unit
pub enum Db<V, UNIT> {
    /// No suffix provided, unknown if linear or log
    None(V),
    /// Linear suffix provided
    Linear(UNIT),
    /// Log suffix provided
    Logarithmic(V, UNIT),
}

/// Amplitude specifier
pub enum Amplitude<UNIT> {
    /// No amplitude specifier
    None(UNIT),
    /// `<UNIT>PK`
    ///
    /// Example: `VPK`
    Peak(UNIT),
    /// `<UNIT>PP`
    ///
    /// Example: `VPP`
    PeakToPeak(UNIT),
    /// `<UNIT>RMS`
    ///
    /// Example: `VRMS`
    Rms(UNIT),
}

impl<'a, UNIT> TryFrom<Token<'a>> for Amplitude<UNIT>
where
    UNIT: TryFrom<Token<'a>, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Self, Self::Error> {
        fn ends_with_ignore_ascii(str: &[u8], needle: &[u8]) -> bool {
            let (m, n) = (str.len(), needle.len());
            m >= n && needle.eq_ignore_ascii_case(&str[m - n..])
        }

        match value {
            Token::DecimalNumericSuffixProgramData(num, s) if ends_with_ignore_ascii(s, b"PK") => {
                Ok(Self::Peak(<UNIT>::try_from(
                    Token::DecimalNumericSuffixProgramData(num, &s[..s.len() - 2]),
                )?))
            }
            Token::DecimalNumericSuffixProgramData(num, s) if ends_with_ignore_ascii(s, b"PP") => {
                Ok(Self::PeakToPeak(<UNIT>::try_from(
                    Token::DecimalNumericSuffixProgramData(num, &s[..s.len() - 2]),
                )?))
            }
            Token::DecimalNumericSuffixProgramData(num, s) if ends_with_ignore_ascii(s, b"RMS") => {
                Ok(Self::Rms(<UNIT>::try_from(
                    Token::DecimalNumericSuffixProgramData(num, &s[..s.len() - 3]),
                )?))
            }
            _ => Ok(Self::None(<UNIT>::try_from(value)?)),
        }
    }
}

#[cfg(feature = "unit-angle")]
mod angle {
    use super::*;
    use uom::si::angle::{degree, gon, minute, radian, revolution, second, Angle};

    impl_unit![uom::si::angle::Conversion<V>, Angle, radian;
        b"RAD" => radian,
        b"DEG" => degree,
        b"MNT" => minute,
        b"SEC" => second,
        b"REV" => revolution,
        b"GON" => gon
    ];
}

#[cfg(feature = "unit-capacitance")]
mod capacitance {
    use super::*;
    use uom::si::capacitance::{farad, microfarad, millifarad, nanofarad, picofarad, Capacitance};

    impl_unit![uom::si::capacitance::Conversion<V>, Capacitance, farad;
        b"F" => farad,
        b"MF" => millifarad,
        b"UF" => microfarad,
        b"NF" => nanofarad,
        b"PF" => picofarad
    ];
}

#[cfg(feature = "unit-electric-charge")]
mod electric_charge {
    use super::*;
    use uom::si::electric_charge::{
        ampere_hour, coulomb, kilocoulomb, megacoulomb, microcoulomb, milliampere_hour,
        millicoulomb, ElectricCharge,
    };

    impl_unit![uom::si::electric_charge::Conversion<V>, ElectricCharge, coulomb;
        //Coloumb
        b"MAC" => megacoulomb,
        b"KC" => kilocoulomb,
        b"C" => coulomb,
        b"MC" => millicoulomb,
        b"UC" => microcoulomb,
        //Ampere hour
        b"AH"|b"A.HR" => ampere_hour,
        b"MAH"|b"MA.HR" => milliampere_hour
    ];
}

#[cfg(feature = "unit-electric-current")]
mod electric_current {
    use super::*;
    use uom::si::electric_current::{
        ampere, kiloampere, microampere, milliampere, nanoampere, ElectricCurrent,
    };

    impl_unit![uom::si::electric_current::Conversion<V>, ElectricCurrent, ampere;
        b"KA" => kiloampere,
        b"A" => ampere,
        b"MA" => milliampere,
        b"UA" => microampere,
        b"NA" => nanoampere
    ];

    impl_logarithmic_unit![uom::si::electric_current::Conversion<V>, ElectricCurrent;
        b"DBA" => ampere,
        b"DBMA" => milliampere,
        b"DBUA" => microampere
    ];
}

#[cfg(feature = "unit-electric-potential")]
mod electric_potential {
    use super::*;
    use uom::si::electric_potential::{kilovolt, microvolt, millivolt, volt, ElectricPotential};

    impl_unit![uom::si::electric_potential::Conversion<V>, ElectricPotential, volt;
        b"KV" => kilovolt,
        b"V" => volt,
        b"MV" => millivolt,
        b"UV" => microvolt
    ];

    impl_logarithmic_unit![uom::si::electric_potential::Conversion<V>, ElectricPotential;
        b"DBV" => volt,
        b"DBMV" => millivolt,
        b"DBUV" => microvolt
    ];
}

#[cfg(feature = "unit-electrical-conductance")]
mod electrical_conductance {
    use super::*;
    use uom::si::electrical_conductance::{
        kilosiemens, microsiemens, millisiemens, siemens, ElectricalConductance,
    };

    impl_unit![uom::si::electrical_conductance::Conversion<V>, ElectricalConductance, siemens;
        b"KSIE" => kilosiemens,
        b"SIE" => siemens,
        b"MSIE" => millisiemens,
        b"USIE" => microsiemens
    ];
}

#[cfg(feature = "unit-electrical-resistance")]
mod electrical_resistance {
    use super::*;

    use uom::si::electrical_resistance::{
        gigaohm, kiloohm, megaohm, microohm, ohm, ElectricalResistance,
    };
    impl_unit![uom::si::electrical_resistance::Conversion<V>, ElectricalResistance, ohm;
        b"GOHM" => gigaohm,
        b"MOHM" => megaohm,
        b"KOHM" => kiloohm,
        b"OHM" => ohm,
        b"UOHM" => microohm
    ];
}

#[cfg(feature = "unit-energy")]
mod energy {
    use super::*;
    use uom::si::energy::{
        electronvolt, joule, kilojoule, megajoule, megawatt_hour, microjoule, milliwatt_hour,
        watt_hour, Energy,
    };

    impl_unit![uom::si::energy::Conversion<V>, Energy, joule;
        b"MAJ" => megajoule,
        b"KJ" => kilojoule,
        b"J" => joule,
        b"MJ" => megajoule,
        b"UJ" => microjoule,
        // Watt-hour
        b"MAW.HR" => megawatt_hour,
        b"WH"|b"W.HR" => watt_hour,
        b"MW.HR" => milliwatt_hour,
        // Electronvolt
        b"EV" => electronvolt
    ];
}

#[cfg(feature = "unit-inductance")]
mod inductance {
    use super::*;
    use uom::si::inductance::{henry, microhenry, millihenry, nanohenry, picohenry, Inductance};

    impl_unit![uom::si::inductance::Conversion<V>, Inductance, henry;
        b"H" => henry,
        b"MH" => millihenry,
        b"UH" => microhenry,
        b"NH" => nanohenry,
        b"PH" => picohenry
    ];
}

#[cfg(feature = "unit-power")]
mod power {
    use super::*;
    use uom::si::power::{kilowatt, megawatt, microwatt, milliwatt, watt, Power};

    impl_unit![uom::si::power::Conversion<V>, Power, watt;
        b"MAW" => megawatt,
        b"KW" => kilowatt,
        b"W" => watt,
        b"MW" => milliwatt,
        b"UW" => microwatt
    ];

    impl_logarithmic_unit![uom::si::power::Conversion<V>, Power;
        b"DBW" => watt,
        b"DBMW" | b"DBM" => milliwatt,
        b"DBUW" => microwatt
    ];
}

#[cfg(feature = "unit-ratio")]
mod ratio {
    use super::*;
    use uom::si::ratio::{part_per_million, percent, ratio, Ratio};

    impl_unit![uom::si::ratio::Conversion<V>, Ratio, ratio;
        b"PCT" => percent,
        b"PPM" => part_per_million
    ];

    impl_logarithmic_unit![uom::si::ratio::Conversion<V>, Ratio;
        b"DB" => ratio
    ];
}

#[cfg(feature = "unit-thermodynamic-temperature")]
mod thermodynamic_temperature {
    use super::*;
    use uom::si::thermodynamic_temperature::{
        degree_celsius, degree_fahrenheit, kelvin, ThermodynamicTemperature,
    };

    impl_unit![uom::si::thermodynamic_temperature::Conversion<V>, ThermodynamicTemperature, degree_celsius;
        b"CEL" => degree_celsius,
        b"FAR" => degree_fahrenheit,
        b"K" => kelvin
    ];
}

#[cfg(feature = "unit-time")]
mod time {
    use super::*;
    use uom::si::time::{
        day, hour, microsecond, millisecond, minute, nanosecond, second, year, Time,
    };

    impl_unit![uom::si::time::Conversion<V>, Time, second;
        b"S" => second,
        b"MS" => millisecond,
        b"US" => microsecond,
        b"NS" => nanosecond,
        b"MIN" => minute,
        b"HR" => hour,
        b"D" => day,
        b"ANN" => year
    ];
}

#[cfg(feature = "unit-frequency")]
mod frequency {
    use super::*;
    use uom::si::frequency::{gigahertz, hertz, kilohertz, megahertz, Frequency};

    impl_unit![uom::si::frequency::Conversion<V>, Frequency, hertz;
        b"GHZ" => gigahertz,
        b"MHZ" | b"MAHZ" => megahertz,
        b"KHZ" => kilohertz,
        b"HZ" => hertz
    ];
}

#[cfg(all(feature = "unit-electric-potential", test))]
mod test_suffix {

    extern crate std;

    use crate::{
        parser::suffix::{Amplitude, Db},
        //scpi1999::numeric::NumericValueDefaults,
        tree::prelude::*,
    };
    use core::convert::TryInto;
    use uom::si::f32::*;

    #[test]
    fn test_suffix_amplitude() {
        let none: Amplitude<ElectricPotential> =
            Token::DecimalNumericSuffixProgramData(b"1.0", b"V")
                .try_into()
                .unwrap();
        assert!(matches!(none, Amplitude::None(_)));
        let peak: Amplitude<ElectricPotential> =
            Token::DecimalNumericSuffixProgramData(b"1.0", b"VPK")
                .try_into()
                .unwrap();
        assert!(matches!(peak, Amplitude::Peak(_)));
        let peak_to_peak: Amplitude<ElectricPotential> =
            Token::DecimalNumericSuffixProgramData(b"1.0", b"VPP")
                .try_into()
                .unwrap();
        assert!(matches!(peak_to_peak, Amplitude::PeakToPeak(_)));
        let rms: Amplitude<ElectricPotential> =
            Token::DecimalNumericSuffixProgramData(b"1.0", b"VRMS")
                .try_into()
                .unwrap();
        assert!(matches!(rms, Amplitude::Rms(_)))
    }

    #[test]
    fn test_suffix_logarithmic() {
        let none: Db<f32, ElectricPotential> =
            Token::DecimalNumericProgramData(b"1.0").try_into().unwrap();
        assert!(matches!(none, Db::None(_)));
        let peak: Db<f32, ElectricPotential> = Token::DecimalNumericSuffixProgramData(b"1.0", b"V")
            .try_into()
            .unwrap();
        assert!(matches!(peak, Db::Linear(_)));
        let peak_to_peak: Db<f32, ElectricPotential> =
            Token::DecimalNumericSuffixProgramData(b"1.0", b"DBV")
                .try_into()
                .unwrap();
        assert!(matches!(peak_to_peak, Db::Logarithmic(_, _)));
    }

    // #[test]
    // fn test_suffix_numeric_value() {
    //     let volt_max = ElectricPotential::numeric_value_max();
    //     assert_eq!(volt_max.value, f32::MAX)
    // }
}

#[allow(unused_macros)]
macro_rules! impl_logarithmic_unit {
    ($conversion:path, $unit:ident; $($($suffix:literal)|+ => $subunit:ident),+) => {
        impl<'a, U, V> TryFrom<Token<'a>> for Db<V,$unit<U, V>>
        where
            U: Units<V> + ?Sized,
            V: Num + uom::Conversion<V> + TryFrom<Token<'a>,Error=Error>,
            $unit<U, V>: TryFrom<Token<'a>,Error=Error>,
            $(
            $subunit: $conversion
            ),+
        {
            type Error = Error;

            fn try_from(value: Token<'a>) -> Result<Self, Self::Error> {
                if let Token::DecimalNumericProgramData(_) = value {
                    Ok(Self::None(<V>::try_from(value)?))
                } else if let Token::DecimalNumericSuffixProgramData(num, suffix) = value {
                    match suffix {
                        $(
                        s if $(s.eq_ignore_ascii_case($suffix))||+ => Ok(Self::Logarithmic(
                            <V>::try_from(Token::DecimalNumericProgramData(num))?,
                            <$unit<U,V>>::new::<$subunit>(V::one())
                        ))
                        ),+,
                        _ => Ok(Self::Linear(<$unit<U, V>>::try_from(value)?))
                    }
                } else {
                    Err(ErrorCode::DataTypeError.into())
                }
            }
        }

        //TODO: Test log parsing
    };
}
pub(crate) use impl_logarithmic_unit;

#[allow(unused_macros)]
macro_rules! impl_unit {
    ($conversion:path, $unit:ident, $base:ident; $($($suffix:literal)|+ => $subunit:ident),+) => {
        use uom::{num_traits::Num, si::Units, Conversion};

        impl<'a, U, V> TryFrom<Token<'a>> for $unit<U, V>
        where
            U: Units<V> + ?Sized,
            V: Num + uom::Conversion<V> + TryFrom<Token<'a>, Error=Error>,
            $base: $conversion,
            $(
            $subunit: $conversion
            ),+
        {
            type Error = Error;

            fn try_from(value: Token<'a>) -> Result<Self, Self::Error> {
                if let Token::DecimalNumericProgramData(_) = value {
                    Ok($unit::new::<$base>(<V>::try_from(value)?))
                } else if let Token::DecimalNumericSuffixProgramData(num, suffix) = value
                {
                    match suffix {
                        $(
                        s if $(s.eq_ignore_ascii_case($suffix))||+ => Ok($unit::new::<$subunit>(
                            <V>::try_from(Token::DecimalNumericProgramData(num))?,
                        ))
                        ),+,
                        _ => Err(ErrorCode::IllegalParameterValue.into()),
                    }
                } else {
                    Err(ErrorCode::DataTypeError.into())
                }
            }
        }

        impl<U, V> ResponseData for $unit<U, V> where U: Units<V> + ?Sized, V: Num + Conversion<V> + ResponseData {
            fn format_response_data(
                &self,
                formatter: &mut dyn Formatter,
            ) -> core::result::Result<(), Error> {
                self.value.format_response_data(formatter)
            }
        }

        // impl<U, V, T> NumericValueDefaults for $unit<U, V>
        // where
        //     U: Units<V> + ?Sized,
        //     V: Num + Conversion<V, T = T> + NumericValueDefaults,
        //     $base: Conversion<V, T = T>,
        // {
        //     fn numeric_value_max() -> Self {
        //         Self::new::<$base>(V::numeric_value_max())
        //     }

        //     fn numeric_value_min() -> Self {
        //         Self::new::<$base>(V::numeric_value_min())
        //     }
        // }

        #[cfg(test)]
        #[allow(non_snake_case)]
        mod tests {

            extern crate std;

            use crate::{tree::prelude::*};
            use core::convert::TryInto;
            use uom::si::f32::*;

            #[test]
            fn test_suffix_correct() {
                $(
                $(
                let l: $unit = Token::DecimalNumericSuffixProgramData(b"1.0", $suffix)
                    .try_into()
                    .unwrap();
                assert_eq!(l.get::<super::$subunit>(), 1.0f32);
                )+
                )+

            }

            #[test]
            fn test_suffix_incorrect() {
                // Do not accept incorrect suffix
                let l: Result<$unit, Error> = Token::DecimalNumericSuffixProgramData(b"1.0", b"POTATO")
                    .try_into();
                assert_eq!(l, Err(Error::from(ErrorCode::IllegalParameterValue)));
                // Do not accept incorrect datatype
                let l: Result<$unit, Error> = Token::StringProgramData(b"STRING").try_into();
                assert_eq!(l, Err(Error::from(ErrorCode::DataTypeError)))
                // Do not accept
            }

            #[test]
            fn test_suffix_default() {
                let l: $unit = Token::DecimalNumericProgramData(b"1.0").try_into().unwrap();
                assert_eq!(l.get::< super::$base >(), 1.0f32)
            }
        }
    };
}
pub(crate) use impl_unit;

#[allow(unused_imports)]
use {
    crate::{error::ErrorCode, tokenizer::Token},
    core::convert::{TryFrom, TryInto},
};

#[cfg(any(
    feature = "unit-length",
    feature = "unit-velocity",
    feature = "unit-acceleration",
    feature = "unit-electric-potential",
    feature = "unit-electric-current",
    feature = "unit-electric-conductance",
    feature = "unit-electric-resistance",
    feature = "unit-electric-charge",
    feature = "unit-electric-capacitance",
    feature = "unit-electric-inductance",
    feature = "unit-energy",
    feature = "unit-power",
    feature = "unit-angle",
    feature = "unit-amount-of-substance",
    feature = "unit-magnetic-flux",
    feature = "unit-magnetic-flux-density",
    feature = "unit-ratio",
    feature = "unit-temperature",
    feature = "unit-time",
    feature = "unit-pressure",
    feature = "unit-volume"
))]
#[allow(unused_imports)]
use uom::{num_traits::Num, si::Units, Conversion};

#[cfg(feature = "unit-acceleration")]
use uom::si::acceleration::{
    kilometer_per_second_squared, meter_per_second_squared, micrometer_per_second_squared,
    millimeter_per_second_squared, Acceleration,
};
#[cfg(feature = "unit-amount-of-substance")]
use uom::si::amount_of_substance::{micromole, millimole, mole, AmountOfSubstance};
#[cfg(feature = "unit-angle")]
use uom::si::angle::{degree, gon, minute as aminute, radian, revolution, Angle};
#[cfg(feature = "unit-electric-capacitance")]
use uom::si::capacitance::{farad, microfarad, millifarad, nanofarad, picofarad, Capacitance};
#[cfg(feature = "unit-electric-charge")]
use uom::si::electric_charge::{
    ampere_hour, coulomb, kilocoulomb, megacoulomb, microcoulomb, milliampere_hour, millicoulomb,
    ElectricCharge,
};
#[cfg(feature = "unit-electric-current")]
use uom::si::electric_current::{
    ampere, kiloampere, microampere, milliampere, nanoampere, ElectricCurrent,
};
#[cfg(feature = "unit-electric-potential")]
use uom::si::electric_potential::{kilovolt, microvolt, millivolt, volt, ElectricPotential};
#[cfg(feature = "unit-electric-conductance")]
use uom::si::electrical_conductance::{
    kilosiemens, microsiemens, millisiemens, siemens, ElectricalConductance,
};
#[cfg(feature = "unit-electric-resistance")]
use uom::si::electrical_resistance::{
    gigaohm, kiloohm, megaohm, microohm, ohm, ElectricalResistance,
};
#[cfg(feature = "unit-energy")]
use uom::si::energy::{
    electronvolt, joule, kilojoule, megajoule, megawatt_hour, microjoule, milliwatt_hour,
    watt_hour, Energy,
};
#[cfg(feature = "unit-electric-inductance")]
use uom::si::inductance::{henry, microhenry, millihenry, nanohenry, picohenry, Inductance};
#[cfg(feature = "unit-length")]
use uom::si::length::{
    foot, inch, kilometer, meter, micrometer, mil, mile, millimeter, nanometer, nautical_mile,
    parsec, Length,
};
#[cfg(feature = "unit-magnetic-flux")]
use uom::si::magnetic_flux::{weber, MagneticFlux};
#[cfg(feature = "unit-magnetic-flux-density")]
use uom::si::magnetic_flux_density::{tesla, MagneticFluxDensity};
#[cfg(feature = "unit-power")]
use uom::si::power::{kilowatt, megawatt, microwatt, milliwatt, watt, Power};
#[cfg(feature = "unit-pressure")]
use uom::si::pressure::{
    atmosphere, bar, inch_of_mercury, kilopascal, millimeter_of_mercury, millitorr, pascal, torr,
    Pressure,
};
#[cfg(feature = "unit-ratio")]
use uom::si::ratio::{part_per_million, percent, ratio, Ratio};
#[cfg(feature = "unit-thermodynamic-temperature")]
use uom::si::thermodynamic_temperature::{
    degree_celsius, degree_fahrenheit, kelvin, ThermodynamicTemperature,
};
#[cfg(feature = "unit-time")]
use uom::si::time::{day, hour, microsecond, millisecond, minute, nanosecond, second, year, Time};
#[cfg(feature = "unit-velocity")]
use uom::si::velocity::{
    foot_per_second, inch_per_second, kilometer_per_hour, kilometer_per_second, meter_per_second,
    micrometer_per_second, mile_per_hour, mile_per_second, millimeter_per_second, Velocity,
};
#[cfg(feature = "unit-volume")]
use uom::si::volume::{cubic_meter, cubic_millimeter, liter, milliliter, Volume};

#[cfg(feature = "unit-frequency")]
use uom::si::frequency::{gigahertz, hertz, kilohertz, megahertz, Frequency};

use crate::error::Error;

/// A logarithmic or linear unit
pub enum Db<V, UNIT> {
    /// No suffix provided, unknown if linear or log
    None(V),
    /// Linear suffix provided
    Linear(UNIT),
    /// Log suffix provided
    Logarithmic(V, UNIT),
}

/// Amplitude specifier (
pub enum Amplitude<UNIT> {
    /// No amplitude specifier
    None(UNIT),
    /// <UNIT>PK
    Peak(UNIT),
    /// <UNIT>PP
    PeakToPeak(UNIT),
    /// <UNIT>RMS
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

#[allow(unused_macros)]
macro_rules! try_from_db_unit {
    ($conversion:path, $unit:ident; $($($suffix:literal)|+ => $subunit:ty),+) => {
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
    };
}

#[allow(unused_macros)]
macro_rules! try_from_unit {
    ($conversion:path, $unit:ident, $base:ty; $($($suffix:literal)|+ => $subunit:ty),+) => {
        impl<'a, U, V> TryFrom<Token<'a>> for $unit<U, V>
        where
            U: Units<V> + ?Sized,
            V: Num + uom::Conversion<V> + TryFrom<Token<'a>,Error=Error>,
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
    };
}

#[cfg(feature = "unit-length")]
try_from_unit![uom::si::length::Conversion<V>, Length, meter;
    b"KM" => kilometer,
    b"M" => meter,
    b"MM" => millimeter,
    b"UM" => micrometer,
    b"NM" => nanometer,
    b"NIM" => nautical_mile,
    // Imperial bullshit
    b"MI" => mile,
    b"FT" => foot,
    b"IN" => inch,
    b"MIL" => mil,
    // Parsec
    b"PSC" => parsec
];

#[cfg(feature = "unit-velocity")]
try_from_unit![uom::si::velocity::Conversion<V>, Velocity, meter_per_second;
    b"KM/S"|b"KM.S-1" => kilometer_per_second,
    b"KMH"|b"KM/HR"|b"KM.HR-1" => kilometer_per_hour,
    b"M/S"|b"M.S-1" => meter_per_second,
    b"MM/S"|b"MM.S-1" => millimeter_per_second,
    b"UM/S"|b"UM.S-1" => micrometer_per_second,
    // Imperial bullshit
    b"MI/S"|b"MI.S-1" => mile_per_second,
    b"MPH"|b"MI/HR"|b"MI.HR-1" => mile_per_hour,
    b"FT/S"|b"FT.S-1" => foot_per_second,
    b"IN/S"|b"IN.S-1" => inch_per_second
];

#[cfg(feature = "unit-acceleration")]
try_from_unit![uom::si::acceleration::Conversion<V>, Acceleration, meter_per_second_squared;
    b"KM/S2"|b"KM.S-2" => kilometer_per_second_squared,
    b"M/S2"|b"M.S-2" => meter_per_second_squared,
    b"MM/S2"|b"MM.S-2" => millimeter_per_second_squared,
    b"UM/S2"|b"UM.S-2" => micrometer_per_second_squared
];

#[cfg(feature = "unit-electric-potential")]
try_from_unit![uom::si::electric_potential::Conversion<V>, ElectricPotential, volt;
    b"KV" => kilovolt,
    b"V" => volt,
    b"MV" => millivolt,
    b"UV" => microvolt
];

#[cfg(feature = "unit-electric-potential")]
try_from_db_unit![uom::si::electric_potential::Conversion<V>, ElectricPotential;
    b"DBV" => volt,
    b"DBMV" => millivolt,
    b"DBUV" => microvolt
];

#[cfg(feature = "unit-electric-current")]
try_from_unit![uom::si::electric_current::Conversion<V>, ElectricCurrent, ampere;
    b"KA" => kiloampere,
    b"A" => ampere,
    b"MA" => milliampere,
    b"UA" => microampere,
    b"NA" => nanoampere
];

#[cfg(feature = "unit-electric-current")]
try_from_db_unit![uom::si::electric_current::Conversion<V>, ElectricCurrent;
    b"DBA" => ampere,
    b"DBMA" => milliampere,
    b"DBUA" => microampere
];

#[cfg(feature = "unit-electric-conductance")]
try_from_unit![uom::si::electrical_conductance::Conversion<V>, ElectricalConductance, siemens;
    b"KSIE" => kilosiemens,
    b"SIE" => siemens,
    b"MSIE" => millisiemens,
    b"USIE" => microsiemens
];

#[cfg(feature = "unit-electric-resistance")]
try_from_unit![uom::si::electrical_resistance::Conversion<V>, ElectricalResistance, ohm;
    b"GOHM" => gigaohm,
    b"MOHM" => megaohm,
    b"KOHM" => kiloohm,
    b"OHM" => ohm,
    b"UOHM" => microohm
];

#[cfg(feature = "unit-electric-charge")]
try_from_unit![uom::si::electric_charge::Conversion<V>, ElectricCharge, coulomb;
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

#[cfg(feature = "unit-electric-capacitance")]
try_from_unit![uom::si::capacitance::Conversion<V>, Capacitance, farad;
    b"F" => farad,
    b"MF" => millifarad,
    b"UF" => microfarad,
    b"NF" => nanofarad,
    b"PF" => picofarad
];

#[cfg(feature = "unit-electric-inductance")]
try_from_unit![uom::si::inductance::Conversion<V>, Inductance, henry;
    b"H" => henry,
    b"MH" => millihenry,
    b"UH" => microhenry,
    b"NH" => nanohenry,
    b"PH" => picohenry
];

#[cfg(feature = "unit-energy")]
try_from_unit![uom::si::energy::Conversion<V>, Energy, joule;
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

#[cfg(feature = "unit-power")]
try_from_unit![uom::si::power::Conversion<V>, Power, watt;
    b"MAW" => megawatt,
    b"KW" => kilowatt,
    b"W" => watt,
    b"MW" => milliwatt,
    b"UW" => microwatt
];

#[cfg(feature = "unit-power")]
try_from_db_unit![uom::si::power::Conversion<V>, Power;
    b"DBW" => watt,
    b"DBMW" | b"DBM" => milliwatt,
    b"DBUW" => microwatt
];

#[cfg(feature = "unit-angle")]
try_from_unit![uom::si::angle::Conversion<V>, Angle, radian;
    b"RAD" => radian,
    b"DEG" => degree,
    b"MNT" => aminute,
    b"REV" => revolution,
    b"GON" => gon
];

#[cfg(feature = "unit-amount-of-substance")]
try_from_unit![uom::si::amount_of_substance::Conversion<V>, AmountOfSubstance, mole;
    b"MOL" => mole,
    b"MMOL" => millimole,
    b"UMOL" => micromole
];

#[cfg(feature = "unit-magnetic-flux")]
try_from_unit![uom::si::magnetic_flux::Conversion<V>, MagneticFlux, weber;
    b"WB" => weber
];

#[cfg(feature = "unit-magnetic-flux-density")]
try_from_unit![uom::si::magnetic_flux_density::Conversion<V>, MagneticFluxDensity, tesla;
    b"T" => tesla
];

#[cfg(feature = "unit-ratio")]
try_from_unit![uom::si::ratio::Conversion<V>, Ratio, ratio;
    b"PCT" => percent,
    b"PPM" => part_per_million
];

#[cfg(feature = "unit-ratio")]
try_from_db_unit![uom::si::ratio::Conversion<V>, Ratio;
    b"DB" => ratio
];

#[cfg(feature = "unit-thermodynamic-temperature")]
try_from_unit![uom::si::thermodynamic_temperature::Conversion<V>, ThermodynamicTemperature, degree_celsius;
    b"CEL" => degree_celsius,
    b"FAR" => degree_fahrenheit,
    b"K" => kelvin
];

#[cfg(feature = "unit-time")]
try_from_unit![uom::si::time::Conversion<V>, Time, second;
    b"S" => second,
    b"MS" => millisecond,
    b"US" => microsecond,
    b"NS" => nanosecond,
    b"MIN" => minute,
    b"HR" => hour,
    b"D" => day,
    b"ANN" => year
];

#[cfg(feature = "unit-pressure")]
try_from_unit![uom::si::pressure::Conversion<V>, Pressure, atmosphere;
    b"ATM" => atmosphere,
    b"MMHG" => millimeter_of_mercury,
    b"INHG" => inch_of_mercury,
    b"TORR" => torr,
    b"MTORR" => millitorr,
    b"PAL" => pascal,
    b"KPAL" => kilopascal,
    b"BAR" => bar
];

#[cfg(feature = "unit-volume")]
try_from_unit![uom::si::volume::Conversion<V>, Volume, liter;
    b"L" => liter,
    b"ML" => milliliter,
    b"M3" => cubic_meter,
    b"MM3" => cubic_millimeter
];

#[cfg(feature = "unit-frequency")]
try_from_unit![uom::si::frequency::Conversion<V>, Frequency, hertz;
    b"GHZ" => gigahertz,
    b"MHZ" | b"MAHZ" => megahertz,
    b"KHZ" => kilohertz,
    b"HZ" => hertz
];

#[cfg(all(feature = "unit-length", test))]
mod test_suffix {

    extern crate std;

    use crate::error::{Error, ErrorCode};
    use crate::tokenizer::Token;
    use core::convert::TryInto;
    use uom::si::f32::*;
    use uom::si::length::meter;

    #[test]
    fn test_suffix_correct() {
        let l: Length = Token::DecimalNumericSuffixProgramData(b"1.0", b"M")
            .try_into()
            .unwrap();
        assert_eq!(l.get::<meter>(), 1.0f32)
    }

    #[test]
    fn test_suffix_missmatch() {
        let l: Result<Length, Error> =
            Token::DecimalNumericSuffixProgramData(b"1.0", b"S").try_into();
        assert_eq!(l, Err(Error::from(ErrorCode::IllegalParameterValue)))
    }

    #[test]
    fn test_suffix_datatype() {
        let l: Result<Length, Error> = Token::StringProgramData(b"STRING").try_into();
        assert_eq!(l, Err(Error::from(ErrorCode::DataTypeError)))
    }

    #[test]
    fn test_suffix_default() {
        let l: Length = Token::DecimalNumericProgramData(b"1.0").try_into().unwrap();
        assert_eq!(l.get::<meter>(), 1.0f32)
    }
}

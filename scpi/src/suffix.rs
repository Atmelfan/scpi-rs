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
use {
    crate::{error::ErrorCode, tokenizer::Token},
    core::convert::{TryFrom, TryInto},
    uom::{
        si::{f32::*, Units},
        Conversion,
    },
};

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

#[allow(unused_macros)]
macro_rules! try_from_unit {
    ($unit:ident, $base:ty, $storage:ty; $($($suffix:literal)|+ => $subunit:ty),+) => {
        impl<'a, U> TryFrom<Token<'a>> for $unit<U, $storage>
        where
            U: Units<$storage> + ?Sized,
        {
            type Error = crate::error::Error;

            fn try_from(value: Token) -> Result<Self, Self::Error> {
                if let Token::DecimalNumericProgramData(_) = value {
                    Ok($unit::new::<$base>(<$storage>::try_from(value)?))
                } else if let Token::DecimalNumericSuffixProgramData(num, suffix) = value
                {
                    match suffix {
                        $(
                        s if $(s.eq_ignore_ascii_case($suffix))||+ => Ok($unit::new::<$subunit>(
                            <$storage>::try_from(Token::DecimalNumericProgramData(num))?,
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
try_from_unit![Length, meter, f32;
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
try_from_unit![Velocity, meter_per_second, f32;
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
try_from_unit![Acceleration, meter_per_second_squared, f32;
    b"KM/S2"|b"KM.S-2" => kilometer_per_second_squared,
    b"M/S2"|b"M.S-2" => meter_per_second_squared,
    b"MM/S2"|b"MM.S-2" => millimeter_per_second_squared,
    b"UM/S2"|b"UM.S-2" => micrometer_per_second_squared
];

#[cfg(feature = "unit-electric-potential")]
try_from_unit![ElectricPotential, volt, f32;
    b"KV" => kilovolt,
    b"V" => volt,
    b"MV" => millivolt,
    b"UV" => microvolt
];

#[cfg(feature = "unit-electric-current")]
try_from_unit![ElectricCurrent, ampere, f32;
    b"KA" => kiloampere,
    b"A" => ampere,
    b"MA" => milliampere,
    b"UA" => microampere,
    b"NA" => nanoampere
];

#[cfg(feature = "unit-electric-conductance")]
try_from_unit![ElectricalConductance, siemens, f32;
    b"KSIE" => kilosiemens,
    b"SIE" => siemens,
    b"MSIE" => millisiemens,
    b"USIE" => microsiemens
];

#[cfg(feature = "unit-electric-resistance")]
try_from_unit![ElectricalResistance, ohm, f32;
    b"GOHM" => gigaohm,
    b"MOHM" => megaohm,
    b"KOHM" => kiloohm,
    b"OHM" => ohm,
    b"UOHM" => microohm
];

#[cfg(feature = "unit-electric-charge")]
try_from_unit![ElectricCharge, coulomb, f32;
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
try_from_unit![Capacitance, farad, f32;
    b"F" => farad,
    b"MF" => millifarad,
    b"UF" => microfarad,
    b"NF" => nanofarad,
    b"PF" => picofarad
];

#[cfg(feature = "unit-electric-inductance")]
try_from_unit![Inductance, henry, f32;
    b"H" => henry,
    b"MH" => millihenry,
    b"UH" => microhenry,
    b"NH" => nanohenry,
    b"PH" => picohenry
];

#[cfg(feature = "unit-energy")]
try_from_unit![Energy, joule, f32;
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
try_from_unit![Power, watt, f32;
    b"MAW" => megawatt,
    b"KW" => kilowatt,
    b"W" => watt,
    b"MW" => milliwatt,
    b"UW" => microwatt
];

#[cfg(feature = "unit-angle")]
try_from_unit![Angle, radian, f32;
    b"RAD" => radian,
    b"DEG" => degree,
    b"MNT" => aminute,
    b"REV" => revolution,
    b"GON" => gon
];

#[cfg(feature = "unit-amount-of-substance")]
try_from_unit![AmountOfSubstance, mole, f32;
    b"MOL" => mole,
    b"MMOL" => millimole,
    b"UMOL" => micromole
];

#[cfg(feature = "unit-magnetic-flux")]
try_from_unit![MagneticFlux, weber, f32;
    b"WB" => weber
];

#[cfg(feature = "unit-magnetic-flux-density")]
try_from_unit![MagneticFluxDensity, tesla, f32;
    b"T" => tesla
];

#[cfg(feature = "unit-ratio")]
try_from_unit![Ratio, ratio, f32;
    b"PCT" => percent,
    b"PPM" => part_per_million
];

#[cfg(feature = "unit-thermodynamic-temperature")]
try_from_unit![ThermodynamicTemperature, degree_celsius, f32;
    b"CEL" => degree_celsius,
    b"FAR" => degree_fahrenheit,
    b"K" => kelvin
];

#[cfg(feature = "unit-time")]
try_from_unit![Time, second, f32;
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
try_from_unit![Pressure, atmosphere, f32;
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
try_from_unit![Volume, liter, f32;
    b"L" => liter,
    b"ML" => milliliter,
    b"M3" => cubic_meter,
    b"MM3" => cubic_millimeter
];

#[cfg(test)]
mod test_suffix {
    extern crate std;
}

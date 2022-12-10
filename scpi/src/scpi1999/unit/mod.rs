extern crate self as scpi;
use core::marker::PhantomData;

use crate::{option::ScpiEnum, tree::prelude::*};

use super::*;

pub trait Unit<UNIT> {
    fn set_unit(&mut self, unit: UNIT) -> Result<()>;

    fn get_unit(&mut self) -> UNIT;
}

/// Unit subsystem stub
///
/// Implement this trait to provide default units for other subsystems.
pub trait DefaultUnits {}
impl<T, U> Unit<U> for T
where
    T: DefaultUnits,
    U: Default,
{
    fn set_unit(&mut self, _unit: U) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }

    fn get_unit(&mut self) -> U {
        Default::default()
    }
}

pub struct UnitCommand<UNIT> {
    _phantom: PhantomData<UNIT>,
}

impl<D, UNIT> Command<D> for UnitCommand<UNIT>
where
    D: ScpiDevice + Unit<UNIT>,
    UNIT: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData + Default,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut Context,
        mut args: Arguments,
    ) -> scpi::error::Result<()> {
        let unit: UNIT = args.data()?;
        device.set_unit(unit)?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let unit = device.get_unit();
        response.data(unit).finish()
    }
}

#[derive(Debug, Clone, Copy, ScpiEnum)]
pub enum AngleUnit {
    #[scpi(mnemonic = b"DEG")]
    Degree,
    #[scpi(mnemonic = b"RAD")]
    Radian,
}
pub type UnitAngle = UnitCommand<AngleUnit>;

impl Default for AngleUnit {
    fn default() -> Self {
        Self::Radian
    }
}

#[cfg(feature = "unit-angle")]
impl AngleUnit {
    pub fn convert(&self, value: uom::si::f32::Angle) -> f32 {
        match self {
            AngleUnit::Degree => value.get::<uom::si::angle::degree>(),
            AngleUnit::Radian => value.get::<uom::si::angle::radian>(),
        }
    }
}

#[derive(Debug, Clone, Copy, ScpiEnum)]
pub enum CurrentUnit {
    #[scpi(mnemonic = b"A")]
    Ampere,
    // TODO: Logarithmic units
    // #[scpi(mnemonic = b"DBA")]
    // DbA,
    // #[scpi(mnemonic = b"DBMA")]
    // DbmA,
    // #[scpi(mnemonic = b"DBUA")]
    // DbuA,
}
pub type UnitCurrent = UnitCommand<CurrentUnit>;

impl Default for CurrentUnit {
    fn default() -> Self {
        Self::Ampere
    }
}

#[cfg(feature = "unit-electric-current")]
impl CurrentUnit {
    pub fn convert(&self, value: uom::si::f32::ElectricCurrent) -> f32 {
        match self {
            CurrentUnit::Ampere => value.get::<uom::si::electric_current::ampere>(),
            // PowerUnit::Dbm => todo!(),
            // PowerUnit::DbmW => todo!(),
            // PowerUnit::DbuW => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, ScpiEnum)]
pub enum VoltageUnit {
    #[scpi(mnemonic = b"V")]
    Volt,
    // TODO: Logarithmic units
    // #[scpi(mnemonic = b"DBV")]
    // DbV,
    // #[scpi(mnemonic = b"DBMV")]
    // DbmV,
    // #[scpi(mnemonic = b"DBUV")]
    // DbuV,
}
pub type UnitVoltage = UnitCommand<VoltageUnit>;

impl Default for VoltageUnit {
    fn default() -> Self {
        Self::Volt
    }
}

#[cfg(feature = "unit-electric-potential")]
impl VoltageUnit {
    pub fn convert(&self, value: uom::si::f32::ElectricPotential) -> f32 {
        match self {
            VoltageUnit::Volt => value.get::<uom::si::electric_potential::volt>(),
            // PowerUnit::Dbm => todo!(),
            // PowerUnit::DbmW => todo!(),
            // PowerUnit::DbuW => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, ScpiEnum)]
pub enum PowerUnit {
    #[scpi(mnemonic = b"W")]
    Watt,
    // TODO: Logarithmic units
    // #[scpi(mnemonic = b"DBM")]
    // Dbm,
    // #[scpi(mnemonic = b"DBMW")]
    // DbmW,
    // #[scpi(mnemonic = b"DBUW")]
    // DbuW,
}
pub type UnitPower = UnitCommand<PowerUnit>;

impl Default for PowerUnit {
    fn default() -> Self {
        Self::Watt
    }
}

#[cfg(feature = "unit-power")]
impl PowerUnit {
    pub fn convert(&self, value: uom::si::f32::Power) -> f32 {
        match self {
            PowerUnit::Watt => value.get::<uom::si::power::watt>(),
            // PowerUnit::Dbm => todo!(),
            // PowerUnit::DbmW => todo!(),
            // PowerUnit::DbuW => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

// Can't use ScpiEnum due to long-form being returned instead of short-form like verything else does
impl ResponseData for TemperatureUnit {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        match self {
            TemperatureUnit::Celsius => formatter.push_ascii(b"CEL"),
            TemperatureUnit::Fahrenheit => formatter.push_ascii(b"FAR"),
            TemperatureUnit::Kelvin => formatter.push_ascii(b"K"),
        }
    }
}

// Can't use ScpiEnum due to long-form being returned instead of short-form like verything else does
impl<'a> TryFrom<Token<'a>> for TemperatureUnit {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Self> {
        match value {
            Token::CharacterProgramData(s) => match s {
                x if crate::parser::mnemonic_compare(b"Cel", x) => Ok(Self::Celsius),
                x if crate::parser::mnemonic_compare(b"Far", x) => Ok(Self::Fahrenheit),
                x if crate::parser::mnemonic_compare(b"K", x) => Ok(Self::Kelvin),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            },
            _ => Err(ErrorCode::IllegalParameterValue.into()),
        }
    }
}

pub type UnitTemperature = UnitCommand<TemperatureUnit>;

impl Default for TemperatureUnit {
    fn default() -> Self {
        Self::Celsius
    }
}

#[cfg(feature = "unit-temperature")]
impl TemperatureUnit {
    pub fn convert(&self, value: uom::si::f32::ThermodynamicTemperature) -> f32 {
        match self {
            TemperatureUnit::Celsius => {
                value.get::<uom::si::thermodynamic_temperature::degree_celsius>()
            }
            TemperatureUnit::Fahrenheit => {
                value.get::<uom::si::thermodynamic_temperature::degree_fahrenheit>()
            }
            TemperatureUnit::Kelvin => value.get::<uom::si::thermodynamic_temperature::kelvin>(),
        }
    }
}

#[derive(Debug, Clone, Copy, ScpiEnum)]
pub enum TimeUnit {
    #[scpi(mnemonic = b"HOUR")]
    Hour,
    #[scpi(mnemonic = b"MINute")]
    Minute,
    #[scpi(mnemonic = b"SECond")]
    Second,
}
pub type UnitTime = UnitCommand<TimeUnit>;

impl Default for TimeUnit {
    fn default() -> Self {
        Self::Second
    }
}

#[cfg(feature = "unit-time")]
impl TimeUnit {
    pub fn convert(&self, value: uom::si::f32::Time) -> f32 {
        match self {
            TimeUnit::Hour => value.get::<uom::si::time::hour>(),
            TimeUnit::Minute => value.get::<uom::si::time::minute>(),
            TimeUnit::Second => value.get::<uom::si::time::second>(),
        }
    }
}

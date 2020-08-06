use scpi::error::Result;
use scpi::expression::numeric_list;
use scpi::expression::numeric_list::NumericList;
use scpi::format::{Arbitrary, Character};
use scpi::prelude::*;
use scpi::NumericValues;

//Default commands
use core::convert::{TryFrom, TryInto};
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::{
    ieee488_cls,
    ieee488_ese,
    ieee488_esr,
    ieee488_idn,
    ieee488_opc,
    ieee488_rst,
    ieee488_sre,
    ieee488_stb,
    ieee488_tst,
    ieee488_wai,
    nquery,
    //Helpers
    qonly,
    scpi_crate_version,
    scpi_status,
    scpi_system,
    scpi_tree,
};

use git_version::git_version;
use scpi::suffix::{Amplitude, Db};
use uom::si::angle::{degree, radian};
use uom::si::electric_potential::volt;
use uom::si::power::watt;
use uom::si::ratio::ratio;
use uom::si::{f32, f64};

const GIT_VERSION: &[u8] = git_version!().as_bytes();

pub struct MyDevice;

/// # `[:EXAMple]:HELLO:WORLD?`
/// Example "Hello world" query
///
/// Will return `Hello world` as string response data.
pub struct HelloWorldCommand {}
impl Command for HelloWorldCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response.data(b"Hello world" as &[u8]).finish()
    }
}

/// # `[:EXAMple]:ERRor:CUSTom [<NRf> | <non-decimal numeric>]`
/// Example custom error event
///
/// Will log a custom error with the specified error code (or `1` if not given).
pub struct ErrorCustomCommand {}
impl Command for ErrorCustomCommand {
    nquery!();

    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        let code: i16 = _args
            .next_data(true)?
            .unwrap_or(Token::NonDecimalNumericProgramData(0u32))
            .try_into()?;
        Err(Error::new(ErrorCode::Custom(code, b"Custom error")))
    }
}

/// # `[:EXAMple]:ERRor:EXTended [<NRf> | <non-decimal numeric>]`
/// Example extended error event
///
/// Will log a custom error with extended message and the specified error code (or `1` if not given).
///
/// **Note: This command is only available with the feature `extended-error` enabled**
pub struct ErrorExtendedCommand {}
impl Command for ErrorExtendedCommand {
    nquery!();

    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        let code: i16 = _args
            .next_data(true)?
            .unwrap_or(Token::NonDecimalNumericProgramData(0u32))
            .try_into()?;
        Err(Error::extended(
            ErrorCode::Custom(code, b"Error"),
            b"Additional information",
        ))
    }
}

/// # `[:EXAMple]:ERRor:MULtiple`
///
/// Inserts multiple errors without terminating execution
pub struct ErrorMultipleCommand {}
impl Command for ErrorMultipleCommand {
    nquery!();

    fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        context.push_error(ErrorCode::Custom(1, b"One").into());
        context.push_error(ErrorCode::Custom(2, b"Two").into());
        context.push_error(ErrorCode::Custom(3, b"Three").into());
        Ok(())
    }
}

/// # `[:EXAMple]:NODE:[DEFault]?`
/// Dummy query to demonstrate default commands.
///
/// Will return `DEFault` as character response data.
pub struct ExamNodeDefCommand {}
impl Command for ExamNodeDefCommand {
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        Ok(())
    }

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response.data(Character(b"DEFault")).finish()
    }
}

/// # `[:EXAMple]:NODE:ARGuments`
/// Accepts no arguments
///
/// # `[:EXAMple]:NODE:ARGuments? <NRf> | <non-decimal numeric> [, <string> [, <string> | <arbitrary data>]]`
/// Has one required and one optional argument.
///
pub struct ExamNodeArgCommand {}
impl Command for ExamNodeArgCommand {
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        Ok(())
    }

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let x: u8 = args.next_data(false)?.unwrap().try_into()?;

        let s: &[u8] = args
            .next_data(true)?
            .map_or(Ok(b"default" as &[u8]), |t| t.try_into())?;

        let utf8: &str = args.next_data(true)?.map_or(Ok("💩"), |t| t.try_into())?;

        response.data(x).data(s).data(utf8).finish()
    }
}

/// # `[:EXAMple]:TYPe:NUMeric[:DECimal]? [<NRf> | <MAXimum|MINimum|DEFault>]`
/// Example of a numeric data object, accepts a decimal value or a character data object.
/// Parameter must be within \[0,100\], default is 1.0.
///
/// Will return the given value as decimal response data.
pub struct ExamTypNumDecCommand {}
impl Command for ExamTypNumDecCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let x: f32 = args.next_data(true)?.map_or(Ok(0.0), |t| {
            t.numeric_range(10.0, -10.0, |special| match special {
                NumericValues::Default => Ok(0.0),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })
        })?;
        response.data(x).finish()
    }
}

/// # `[:EXAMple]:TYPe:NUMeric:WATT? [<NRf> | <MAXimum|MINimum|DEFault>]`
/// Example of a numeric data object, accepts a decimal value or a character data object.
///
/// Will return the given value as decimal response data.
pub struct ExamTypNumWattCommand {}
impl Command for ExamTypNumWattCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let x: Db<f32, f32::Power> = args.next_data(true)?.map_or(Ok(Db::None(0.0)), |t| {
            t.numeric(|s| match s {
                NumericValues::Maximum => Ok(Db::Linear(f32::Power::new::<watt>(f32::MAX))),
                NumericValues::Minimum => Ok(Db::Linear(f32::Power::new::<watt>(f32::MIN))),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })
        })?;
        let l: f32::Power = match x {
            // No suffix was given, assume linear
            Db::None(f) => f32::Power::new::<watt>(f),
            // A linear prefix was given
            Db::Linear(v) => v,
            // A logarithmic 'DB' suffix was given
            Db::Logarithmic(d, v) => f32::Ratio::new::<ratio>(10.0f32.powf(d / 10.0)) * v,
        };
        response.header(b"WATT").data(l.get::<watt>()).finish()
    }
}

/// # `[:EXAMple]:TYPe:NUMeric:VOLT? <NRf> [<suffix>]`
/// Identical to `[:EXAMple]:TYPe:NUMeric[:DECimal]?` but accepts a voltage suffix.
///
/// Will return the given value as decimal response data.
pub struct ExamTypNumVoltCommand {}

impl Command for ExamTypNumVoltCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        fn db_to_volt(db: Db<f32, f32::ElectricPotential>) -> f32::ElectricPotential {
            match db {
                // No suffix was given, don't know if it's log or linear
                Db::None(f) => f32::ElectricPotential::new::<volt>(f),
                // A linear prefix was given
                Db::Linear(v) => v,
                // A logarithmic 'DB' suffix was given
                Db::Logarithmic(d, lp) => f32::Ratio::new::<ratio>(10.0f32.powf(d / 10.0)) * lp,
            }
        }

        let max_value = f32::ElectricPotential::new::<volt>(10.0);
        let min_value = f32::ElectricPotential::new::<volt>(0.0);

        let x: Amplitude<Db<f32, f32::ElectricPotential>> =
            args.next_data(true)?
                .map_or(Ok(Amplitude::None(Db::None(0.0))), |t| {
                    t.numeric(|s| match s {
                        //MAX = 10Vpk
                        NumericValues::Maximum => Ok(Amplitude::Peak(Db::Linear(max_value))),
                        //MIN = 0Vpk
                        NumericValues::Minimum => Ok(Amplitude::Peak(Db::Linear(min_value))),
                        //DEFault = 1Vrms
                        NumericValues::Default => {
                            Ok(Amplitude::Rms(Db::Linear(f32::ElectricPotential::new::<
                                volt,
                            >(
                                1.0
                            ))))
                        }
                        //other special values invalid
                        _ => Err(ErrorCode::IllegalParameterValue.into()),
                    })
                })?;
        let vpk: f32::ElectricPotential = match x {
            Amplitude::None(x) | Amplitude::Peak(x) => db_to_volt(x),
            Amplitude::PeakToPeak(x) => db_to_volt(x) * f32::Ratio::new::<ratio>(2.0),
            Amplitude::Rms(x) => db_to_volt(x) * f32::Ratio::new::<ratio>(2.0f32.sqrt()),
        };
        if vpk > max_value || vpk < min_value {
            return Err(ErrorCode::DataOutOfRange.into());
        }

        response
            .header(b"VOLT")
            .header(b"PEAK")
            .data(vpk.get::<volt>())
            .finish()
    }
}

/// # `[:EXAMple]:TYPe:NUMeric:ANGLE? [<NRf> [<suffix>]]`
/// Identical to `[:EXAMple]:TYPe:NUMeric[:DECimal]?` but accepts a angle suffix.
///
/// Will return the given value as decimal response data.
pub struct ExamTypNumAngleCommand {}

impl Command for ExamTypNumAngleCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        //Optional parameter (default value of 1.0f32), accepts volt suffix, accepts MIN/MAX/DEFault
        // Unfortunately doesn't work with Amplitude<> or Db<>
        let x: f64::Angle =
            args.next_data(true)?
                .map_or(Ok(f64::Angle::new::<radian>(1.0)), |t| {
                    t.numeric_range(
                        f64::Angle::new::<degree>(-180.0),
                        f64::Angle::new::<degree>(180.0),
                        |special| match special {
                            NumericValues::Default => Ok(f64::Angle::new::<degree>(0.0)),
                            _ => Err(ErrorCode::IllegalParameterValue.into()),
                        },
                    )
                })?;
        response.header(b"RADian").data(x.get::<radian>()).finish()
    }
}

/// # `[:EXAMple]:TYPe:STRing? [<string>]`
/// Takes one optional string argument.
///
/// Returns given string as a string data object.
pub struct ExamTypStrCommand {}
impl Command for ExamTypStrCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let s: &[u8] = args
            .next_data(true)?
            .unwrap_or(Token::StringProgramData(b"DEFault"))
            .try_into()?;
        response.data(s).finish()
    }
}

/// # `[:EXAMple]:TYPe:ARBitrary? [<arbitrary>]`
/// Takes one optional arbitrary argument.
///
/// Returns given arb as a arb response data.
pub struct ExamTypArbCommand {}
impl Command for ExamTypArbCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let arb: Arbitrary = args
            .next_data(true)?
            .unwrap_or(Token::ArbitraryBlockData(&[0u8, 1u8, 2u8]))
            .try_into()?;
        response.data(arb).finish()
    }
}

/// # `[:EXAMple]:TYPe:LIST:CHANnel? <channel list>`
///
/// **NOT YET IMPLEMENTED**
pub struct ExamTypListChanCommand {}
impl Command for ExamTypListChanCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        _response: &mut ResponseUnit,
    ) -> Result<()> {
        //TODO: Implement this
        todo!()
    }
}

/// # `[:EXAMple]:TYPe:LIST:NUMeric? <numeric list>`
/// Takes one expression argument of the form of a numeric list.
///
/// Returns numbers in list.
pub struct ExamTypListNumCommand {}
impl Command for ExamTypListNumCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let list: NumericList = args.next_data(false)?.unwrap().try_into()?;
        for item in list {
            match item? {
                numeric_list::Token::Numeric(a) => {
                    response.data(f32::try_from(a)?);
                }
                numeric_list::Token::NumericRange(a, b) => {
                    response.data(f32::try_from(a)?);
                    response.data(f32::try_from(b)?);
                }
            }
        }
        response.finish()
    }
}

impl Device for MyDevice {
    fn cls(&mut self) -> Result<()> {
        Ok(())
    }

    fn rst(&mut self) -> Result<()> {
        Ok(())
    }
}

pub const TREE: &Node = scpi_tree![
    // Create default IEEE488 mandated commands
    ieee488_cls!(),
    ieee488_ese!(),
    ieee488_esr!(),
    ieee488_idn!(b"GPA-Robotics", b"T800-101", b"0", GIT_VERSION),
    ieee488_opc!(),
    ieee488_rst!(),
    ieee488_sre!(),
    ieee488_stb!(),
    ieee488_tst!(),
    ieee488_wai!(),
    // Create default SCPI mandated STATus subsystem
    scpi_status!(),
    // Create default SCPI mandated SYSTem subsystem
    scpi_system!(),
    //
    scpi_crate_version!(),
    //Test
    Node {
        name: b"ABORt",
        handler: None,
        optional: false,
        sub: None,
    },
    Node {
        name: b"INITiate",
        handler: None,
        optional: false,
        sub: Some(&[Node {
            name: b"IMMediate",
            handler: None,
            optional: true,
            sub: None,
        }]),
    },
    Node {
        name: b"EXAMple",
        optional: true,
        handler: None,
        sub: Some(&[
            Node {
                name: b"HELLO",
                optional: false,
                handler: None,
                sub: Some(&[Node {
                    name: b"WORLD",
                    optional: true,
                    handler: Some(&HelloWorldCommand {}),
                    sub: None,
                }]),
            },
            Node {
                name: b"ERRor",
                optional: false,
                handler: None,
                sub: Some(&[
                    Node {
                        name: b"CUSTom",
                        optional: false,
                        handler: Some(&ErrorCustomCommand {}),
                        sub: None,
                    },
                    #[cfg(feature = "extended-error")]
                    Node {
                        name: b"EXTended",
                        optional: false,
                        handler: Some(&ErrorExtendedCommand {}),
                        sub: None,
                    },
                    Node {
                        name: b"MULtiple",
                        optional: false,
                        handler: Some(&ErrorMultipleCommand {}),
                        sub: None,
                    },
                ]),
            },
            Node {
                name: b"NODE",
                optional: false,
                handler: None,
                sub: Some(&[
                    Node {
                        name: b"DEFault",
                        optional: true,
                        handler: Some(&ExamNodeDefCommand {}),
                        sub: None,
                    },
                    Node {
                        name: b"ARGuments",
                        optional: true,
                        handler: Some(&ExamNodeArgCommand {}),
                        sub: None,
                    },
                ]),
            },
            Node {
                name: b"TYPes",
                optional: false,
                handler: None,
                sub: Some(&[
                    Node {
                        name: b"NUMeric",
                        optional: false,
                        handler: None,
                        sub: Some(&[
                            Node {
                                name: b"DECimal",
                                optional: true,
                                handler: Some(&ExamTypNumDecCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"WATT",
                                optional: false,
                                handler: Some(&ExamTypNumWattCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"VOLT",
                                optional: false,
                                handler: Some(&ExamTypNumVoltCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"ANGLE",
                                optional: false,
                                handler: Some(&ExamTypNumAngleCommand {}),
                                sub: None,
                            },
                        ]),
                    },
                    Node {
                        name: b"STRing",
                        optional: false,
                        handler: Some(&ExamTypStrCommand {}),
                        sub: None,
                    },
                    Node {
                        name: b"ARBitrary",
                        optional: false,
                        handler: Some(&ExamTypArbCommand {}),
                        sub: None,
                    },
                    Node {
                        name: b"LIST",
                        optional: false,
                        handler: None,
                        sub: Some(&[
                            Node {
                                name: b"NUMeric",
                                optional: true,
                                handler: Some(&ExamTypListNumCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"CHANnel",
                                optional: false,
                                handler: Some(&ExamTypListChanCommand {}),
                                sub: None,
                            }
                        ]),
                    },
                ]),
            },
        ]),
    }
];

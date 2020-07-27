use scpi::error::Result;
use scpi::expression::numeric_list;
use scpi::prelude::*;
use scpi::suffix::SuffixUnitElement;
use scpi::tokenizer::Arbitrary;

//Default commands
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
    scpi_error,
    scpi_status,
    scpi_system,
    scpi_tree,
};
use std::convert::TryInto;

use git_version::git_version;
use std::f32::consts::PI;

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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        response.str_data(b"Hello world")
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
        scpi_error!(ErrorCode::Custom(code, b"Custom error"))
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
        scpi_error!(ErrorCode::Custom(code, b"Error"); b"Additional information")
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        response.ascii_data(b"DEFault")
    }
}

/// # `[:EXAMple]:NODE:ARGuments`
/// Accepts no arguments
///
/// # `[:EXAMple]:NODE:ARGuments? <NRf> | <non-decimal numeric> [, <string>]`
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        let x: u8 = args.next_data(false)?.unwrap().try_into()?;

        let s: &[u8] = args
            .next_data(true)?
            .unwrap_or(Token::StringProgramData(b"DEFault"))
            .try_into()?;

        response.u8_data(x)?;
        response.separator()?;
        response.str_data(s)?;
        Ok(())
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        const MAX: f32 = 100.0;
        const MIN: f32 = 0.0;
        const DEFAULT: f32 = 0.0;
        //Optional value which also accepts MINimum/MAXimum/DEFault
        let x: f32 = args
            .next_data(true)?
            .unwrap_or(Token::DecimalNumericProgramData(DEFAULT))
            .numeric_range(DEFAULT, MIN, MAX)?;
        response.f32_data(x)
    }
}

/// # `[:EXAMple]:TYPe:NUMeric:VOLT? [<NRf> [<suffix>] | <MAXimum|MINimum|DEFault>]`
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        const MAX: f32 = 100.0;
        const MIN: f32 = 0.0;
        const DEFAULT: f32 = 0.0;
        //Optional parameter (default value of 1.0f32), accepts volt suffix, accepts MIN/MAX/DEFault
        let x: f32 = args
            .next_decimal(true, |val, suffix| {
                let (s, v): (SuffixUnitElement, f32) =
                    SuffixUnitElement::from_str(suffix, val).map_err(Error::from)?;
                s.convert(SuffixUnitElement::Volt, v).map_err(Error::from)
            })?
            .unwrap_or(Token::DecimalNumericProgramData(1.0))
            .numeric_range(DEFAULT, MIN, MAX)?;
        response.header_data(b"VOLT")?;
        response.f32_data(x)
    }
}

/// # `[:EXAMple]:TYPe:NUMeric:RADian? [<NRf> [<suffix>] | <MAXimum|MINimum|DEFault>]`
/// Identical to `[:EXAMple]:TYPe:NUMeric[:DECimal]?` but accepts a angle suffix.
///
/// Will return the given value as decimal response data.
pub struct ExamTypNumRadCommand {}
impl Command for ExamTypNumRadCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        const DEFAULT: f32 = 0.0;
        //Optional parameter (default value of 1.0f32), accepts volt suffix, accepts MIN/MAX/DEFault
        let x: f32 = args
            .next_decimal(true, |val, suffix| {
                let (s, v): (SuffixUnitElement, f32) =
                    SuffixUnitElement::from_str(suffix, val).map_err(Error::from)?;
                s.convert(SuffixUnitElement::Radian, v).map_err(Error::from)
            })?
            .unwrap_or(Token::DecimalNumericProgramData(DEFAULT))
            .numeric_range(DEFAULT, -PI, PI)?;
        response.header_data(b"RADian")?;
        response.f32_data(x)
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        let s: &[u8] = args
            .next_data(true)?
            .unwrap_or(Token::StringProgramData(b"DEFault"))
            .try_into()?;
        response.str_data(s)?;
        Ok(())
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        let arb: Arbitrary = args
            .next_data(true)?
            .unwrap_or(Token::ArbitraryBlockData(&[0u8, 1u8, 2u8]))
            .try_into()?;
        response.arb_data(arb)
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
        _response: &mut dyn Formatter,
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
        response: &mut dyn Formatter,
    ) -> Result<()> {
        let list = args.next_data(false)?.unwrap();
        let numbers = list.numeric_list()?;
        for item in numbers {
            let item: numeric_list::Token = item?;
            match item {
                numeric_list::Token::Numeric(a) => {
                    response.isize_data(a)?;
                }
                numeric_list::Token::NumericRange(a, b) => {
                    for x in a..b {
                        response.isize_data(x)?;
                        response.separator()?;
                    }
                    response.isize_data(b)?;
                }
                numeric_list::Token::Separator => {
                    response.separator()?;
                }
            }
        }
        Ok(())
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
                                name: b"VOLT",
                                optional: false,
                                handler: Some(&ExamTypNumVoltCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"RADian",
                                optional: false,
                                handler: Some(&ExamTypNumRadCommand {}),
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

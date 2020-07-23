struct MyDevice;

//use strum::EnumMessage;
use scpi::error::Result;
use scpi::prelude::*;
use std::io;
use std::io::BufRead;

//Default commands
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::suffix::SuffixUnitElement;
use scpi::tokenizer::NumericValues;
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
const GIT_VERSION: &[u8] = git_version!().as_bytes();

struct SensVoltDcCommand;
impl Command for SensVoltDcCommand {
    fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        let x: f32 = args
            .next_data(false)?
            .unwrap()
            .map_character_data(|special| match special {
                x if Token::mnemonic_compare(b"MINimum", x) => {
                    Ok(Token::DecimalNumericProgramData(-34.0))
                }
                x if Token::mnemonic_compare(b"MAXimum", x) => {
                    Ok(Token::DecimalNumericProgramData(54564.0))
                }
                x if Token::mnemonic_compare(b"DEFault", x) => {
                    Ok(Token::DecimalNumericProgramData(0.0))
                }
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })?
            .try_into()?;
        //x *= args.next_suffix_multiplier(Unit::Volt)?;//If no suffix (1.0), else (SUFFIX/Volt). If incompatible, error.
        println!("Value: {}", x);
        Ok(())
    }

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        response.ascii_data(b"[:SENSe]:VOLTage[:DC]?")
    }
}

struct SensVoltAcCommand;
impl Command for SensVoltAcCommand {
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        response.ascii_data(b"[:SENSe]:VOLTage:AC?")
    }
}

/// `EXAMple:HELLO:WORLD?`
/// Example "Hello world" query
struct HelloWorldCommand {}
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

/// `EXAMple:ERRor:CUSTom`
/// Example custom error event
struct ErrorCustomCommand {}
impl Command for ErrorCustomCommand {
    nquery!();

    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        scpi_error!(ErrorCode::Custom(1, b"Custom error"))
    }
}

/// `EXAMple:ERRor:EXTended`
/// Example extended error event
struct ErrorExtendedCommand {}
impl Command for ErrorExtendedCommand {
    nquery!();

    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        scpi_error!(ErrorCode::Custom(1, b"Extended error"); b"Additional information")
    }
}

/// `EXAMple:ERRor:MULtiple`
/// Inserts multiple errors without terminating execution
struct ErrorMultipleCommand {}
impl Command for ErrorMultipleCommand {
    nquery!();

    fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        context.push_error(ErrorCode::Custom(1, b"One").into());
        context.push_error(ErrorCode::Custom(2, b"Two").into());
        context.push_error(ErrorCode::Custom(3, b"Three").into());
        Ok(())
    }
}

/// `EXAMple:NODE:[DEFault]`
/// Dummy command to demonstrate default commands.
///
/// Note: `EXAMple` is actually a default command too, try entering `NODE?`.
struct ExamNodeDefCommand {}
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

/// `EXAMple:NODE:ARGuments`
/// Dummy command to demonstrate default commands.
///
/// `EXAMple:NODE:ARGuments? <NRf> | <non-decimal numeric> [, <string>]`
/// Dummy command to demonstrate default commands.
///
/// Note: `EXAMple` is actually a default command too, try entering `NODE?`.
struct ExamNodeArgCommand {}
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
            .map_or(Ok(b"POTATO".as_ref()), |f| f.try_into())?;

        response.u8_data(x)?;
        response.separator()?;
        response.str_data(s)?;
        Ok(())
    }
}

struct ExamTypNumDecCommand {}
impl Command for ExamTypNumDecCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        //Optional value which also accepts MIN/MAX/DEFault
        let x: f32 = args
            .next_data(true)?
            .unwrap_or(Token::DecimalNumericProgramData(1.0f32))
            .numeric(|n| match n {
                NumericValues::Maximum => Ok(100f32),
                NumericValues::Minimum => Ok(0f32),
                NumericValues::Default => Ok(1f32),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })?;
        response.f32_data(x)?;
        Ok(())
    }
}

struct ExamTypNumVoltCommand {}
impl Command for ExamTypNumVoltCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        //Optional parameter (default value of 1.0f32), accepts volt suffix, accepts MIN/MAX/DEFault
        let x: f32 = args
            .next_decimal(true, |val, suffix| {
                let (s, v): (SuffixUnitElement, f32) = SuffixUnitElement::from_str(suffix, val)
                    .map_err(|_| ErrorCode::SuffixNotAllowed)?;
                s.convert(SuffixUnitElement::Volt, v)
                    .map_err(|_| ErrorCode::SuffixNotAllowed)
            })?
            .unwrap_or(Token::DecimalNumericProgramData(1.0))
            .numeric(|n| match n {
                NumericValues::Maximum => Ok(100f32),
                NumericValues::Minimum => Ok(0f32),
                NumericValues::Default => Ok(1f32),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })?;
        response.f32_data(x)?;
        Ok(())
    }
}

struct ExamTypNumRadCommand {}
impl Command for ExamTypNumRadCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        //Optional parameter (default value of 1.0f32), accepts volt suffix, accepts MIN/MAX/DEFault
        let x: f32 = args
            .next_decimal(true, |val, suffix| {
                let (s, v): (SuffixUnitElement, f32) = SuffixUnitElement::from_str(suffix, val)
                    .map_err(|_| ErrorCode::SuffixNotAllowed)?;
                s.convert(SuffixUnitElement::Radian, v)
                    .map_err(|_| ErrorCode::SuffixNotAllowed)
            })?
            .unwrap_or(Token::DecimalNumericProgramData(1.0))
            .numeric_range(0f32, 100f32, |n| match n {
                NumericValues::Maximum => Ok(100f32),
                NumericValues::Minimum => Ok(0f32),
                NumericValues::Default => Ok(1f32),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })?;
        response.ascii_data(b"RADian ")?;
        response.f32_data(x)?;
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

fn main() {
    let mut my_device = MyDevice {};

    let tree = scpi_tree![
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
                            handler: None,
                            sub: None,
                        },
                        Node {
                            name: b"ARBitrary",
                            optional: false,
                            handler: None,
                            sub: None,
                        },
                    ]),
                },
            ]),
        }
    ];

    let mut errors = ArrayErrorQueue::<[Error; 10]>::new();

    let mut context = Context::new(&mut my_device, &mut errors, &tree);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let message = line.unwrap();

        //Response bytebuffer
        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

        //SCPI tokenizer
        let mut tokenizer = Tokenizer::new(message.as_bytes());

        //loop {
        //Result
        let result = context.exec(&mut tokenizer, &mut buf);

        //Looka like a lot of stuff being allocated but everything is on the stack and lightweight
        use std::str;
        if let Err(err) = result {
            println!("{}", str::from_utf8(err.get_message()).unwrap());
        } else {
            print!("{}", str::from_utf8(buf.as_slice()).unwrap());
            //break;
        }
        //}
    }
}

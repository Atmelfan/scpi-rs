// Test mandated ieee488.2 commands
use scpi::error::Result;
use scpi::prelude::*;

//Default commands
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, nquery, qonly, scpi_status, scpi_system, scpi_tree,
};
use std::convert::TryInto;

mod test_util;
use scpi::tokenizer::{Arbitrary, Character};
use test_util::*;

extern crate std;

macro_rules! numeric_command {
    ($name:ident : $typ:ty, $format:ident) => {
        struct $name {}

        impl Command for $name {
            qonly!();

            fn query(
                &self,
                _context: &mut Context,
                args: &mut Tokenizer,
                response: &mut dyn Formatter,
            ) -> Result<()> {
                let x: $typ = args.next_data(false)?.unwrap().try_into()?;
                response.$format(x)
            }
        }
    };
}

macro_rules! add_numeric_command {
    ($cmd:literal : $name:ident ) => {
        Node {
            name: $cmd,
            optional: false,
            handler: Some(&$name {}),
            sub: None,
        }
    };
}

const IEEE488_TREE: &Node = scpi_tree![
    // Create default IEEE488 mandated commands
    ieee488_cls!(),
    ieee488_ese!(),
    ieee488_esr!(),
    ieee488_idn!(b"GPA-Robotics", b"T800-101", b"0", b"0"),
    ieee488_opc!(),
    ieee488_rst!(),
    ieee488_sre!(),
    ieee488_stb!(),
    ieee488_tst!(),
    ieee488_wai!(),
    scpi_status!(),
    scpi_system!(),
    add_numeric_command!(b"*STR": StrCommand),
    add_numeric_command!(b"*ARB": ArbCommand),
    add_numeric_command!(b"*CHR": ChrCommand),
    add_numeric_command!(b"*UTF8": Utf8Command),
    add_numeric_command!(b"*F32": F32Command),
    add_numeric_command!(b"*F32ISINF": F32IsInfCommand),
    add_numeric_command!(b"*F32ISNAN": F32IsNanCommand),
    add_numeric_command!(b"*U32": U32Command) // add_numeric_command!(b"*U32": U32Command),
                                              // add_numeric_command!(b"*I16": I16Command),
                                              // add_numeric_command!(b"*U16": U16Command),
                                              // add_numeric_command!(b"*I8": I8Command),
                                              // add_numeric_command!(b"*U8": U8Command)
];

struct TestDevice {}

impl TestDevice {
    pub fn new() -> Self {
        TestDevice {}
    }
}

impl Device for TestDevice {
    fn cls(&mut self) -> Result<()> {
        Ok(())
    }

    fn rst(&mut self) -> Result<()> {
        Ok(())
    }

    fn tst(&mut self) -> Result<()> {
        Ok(())
    }
}

numeric_command!(F32Command: f32, f32_data);

struct F32IsNanCommand {}
impl Command for F32IsNanCommand {
    nquery!();

    fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        let x: f32 = args.next_data(false)?.unwrap().try_into()?;
        if x.is_nan() {
            Ok(())
        } else {
            Err(ErrorCode::ExecutionError.into())
        }
    }
}

struct F32IsInfCommand {}
impl Command for F32IsInfCommand {
    nquery!();

    fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        let x: f32 = args.next_data(false)?.unwrap().try_into()?;
        if x.is_infinite() {
            Ok(())
        } else {
            Err(ErrorCode::ExecutionError.into())
        }
    }
}

#[test]
fn test_f32() {
    context!(ctx, dev);
    let valid = ["1.0", "+1", "-1", "1e10"];
    for s in &valid {
        let cmd = format!("{cmd} {value}", cmd = "*F32?", value = s);
        execute_str!(ctx, cmd.as_bytes() => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response.is_empty(), false);
        });
    }
    let infinite = ["inf", "ninf", "INFINITY", "NINFINITY"];
    for s in &infinite {
        let cmd = format!("{cmd} {value}", cmd = "*F32ISINF", value = s);
        execute_str!(ctx, cmd.as_bytes() => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response.is_empty(), true);
        });
    }
    execute_str!(ctx, b"*F32ISNAN NAN" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response.is_empty(), true);
    });
}

#[test]
fn test_f32_datatype_error() {
    context!(ctx, dev);
    execute_str!(ctx, b"*F32? 'STRING'" => result, response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
        assert_eq!(response.is_empty(), true);
    });
    execute_str!(ctx, b"*F32? INVALID" => result, response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
        assert_eq!(response.is_empty(), true);
    });
}

numeric_command!(U32Command: u32, u32_data);
#[test]
fn test_u32() {
    context!(ctx, dev);
    execute_str!(ctx, b"*U32? 42" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"42\n");
    });
}
#[test]
fn test_u32_hex() {
    context!(ctx, dev);
    execute_str!(ctx, b"*U32? #H002A" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"42\n");
    });
}
#[test]
fn test_u32_octal() {
    context!(ctx, dev);
    execute_str!(ctx, b"*U32? #Q52" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"42\n");
    });
}
#[test]
fn test_u32_binary() {
    context!(ctx, dev);
    execute_str!(ctx, b"*U32? #B101010" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"42\n");
    });
}
#[test]
fn test_u32_datatype_error() {
    context!(ctx, dev);
    execute_str!(ctx, b"*U32? 'STRING'" => result, response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
        assert_eq!(response.is_empty(), true);
    });
}
// numeric_command!(I32Command: i32, i32_data);
// numeric_command!(U16Command: u16, u16_data);
// numeric_command!(I16Command: i16, i16_data);
// numeric_command!(U8Command: u8, u8_data);
// numeric_command!(I8Command: i8, i8_data);

numeric_command!(StrCommand: &[u8], str_data);
#[test]
fn test_str() {
    context!(ctx, dev);
    execute_str!(ctx, b"*STR? 'STRING'" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"\"STRING\"\n");
    });
    execute_str!(ctx, b"*STR? CHRDATA" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
    execute_str!(ctx, b"*STR? 1.0" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
}

numeric_command!(ArbCommand: Arbitrary, arb_data);
#[test]
fn test_arb() {
    context!(ctx, dev);
    execute_str!(ctx, b"*ARB? #203ABC" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"#13ABC\n");
    });
    execute_str!(ctx, b"*ARB? CHRDATA" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
    execute_str!(ctx, b"*ARB? 1.0" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
}

numeric_command!(ChrCommand: Character, character_data);
#[test]
fn test_chr() {
    context!(ctx, dev);
    execute_str!(ctx, b"*CHR? CHRDATA" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"CHRDATA\n");
    });
    execute_str!(ctx, b"*CHR? 'CHRDATA'" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
    execute_str!(ctx, b"*CHR? 1.0" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
}

struct Utf8Command {}
impl Command for Utf8Command {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()> {
        let x: &str = args.next_data(false)?.unwrap().try_into()?;
        response.arb_data(Arbitrary(x.as_bytes()))
    }
}
#[test]
fn test_utf8() {
    context!(ctx, dev);
    execute_str!(ctx, b"*UTF8? 'STRING'" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"#16STRING\n");
    });
    execute_str!(ctx, b"*UTF8? #206STRING" => result, _response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"#16STRING\n");
    });
    #[cfg(feature = "arbitrary-utf8-string")]
    execute_str!(ctx, b"*UTF8? #s'STRING'" => result, _response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"#16STRING\n");
    });
    execute_str!(ctx, b"*UTF8? 1.0" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
    });
}

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
use std::convert::{TryFrom, TryInto};

mod test_util;
use scpi::expression::numeric_list;
use scpi::expression::numeric_list::NumericList;
use scpi::format::{Arbitrary, Character};
use scpi::NumericValues;

extern crate std;

macro_rules! numeric_command {
    ($name:ident : $typ:ty) => {
        struct $name {}

        impl Command for $name {
            qonly!();

            fn query(
                &self,
                _context: &mut Context,
                args: &mut Tokenizer,
                response: &mut ResponseUnit,
            ) -> Result<()> {
                let x: $typ = args.next_data(false)?.unwrap().try_into()?;
                response.data(x).finish()
            }
        }
    };
    ($name:ident, $inf:ident, $nan:ident : $typ:ty) => {
        numeric_command!($name: $typ);

        struct $inf {}
        impl Command for $inf {
            nquery!();

            fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<()> {
                let x: $typ = args.next_data(false)?.unwrap().try_into()?;
                if x.is_infinite() {
                    Ok(())
                } else {
                    Err(ErrorCode::ExecutionError.into())
                }
            }
        }

        struct $nan {}
        impl Command for $nan {
            nquery!();

            fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<()> {
                let x: $typ = args.next_data(false)?.unwrap().try_into()?;
                if x.is_nan() {
                    Ok(())
                } else {
                    Err(ErrorCode::ExecutionError.into())
                }
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
            sub: &[],
        }
    };
}

macro_rules! test_integer {
    ($test:ident; $cmd:literal, $min:expr, $max:expr) => {
    mod $test {
        use super::*;
        #[test]
        fn test_integer() {
            context!(ctx, dev);

            execute_str!(ctx, format!("{cmd} 42", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response, b"42\n");
            });
        }
        #[test]
        fn test_rounding() {
            context!(ctx, dev);
            execute_str!(ctx, format!("{cmd} 0.4;{cmd} 0.6", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response, b"0;1\n");
            });
        }
        #[test]
        fn test_max_min() {
            context!(ctx, dev);
            execute_str!(ctx, format!("{cmd} MAX;{cmd} MIN", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response, format!("{max};{min}\n", max=$max, min=$min).as_bytes());
            });
        }
        #[test]
        fn test_hex() {
            context!(ctx, dev);
            execute_str!(ctx, format!("{cmd} #H002A", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response, b"42\n");
            });
        }
        #[test]
        fn test_octal() {
            context!(ctx, dev);
            execute_str!(ctx, format!("{cmd} #Q52", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response, b"42\n");
            });
        }
        #[test]
        fn test_binary() {
            context!(ctx, dev);
            execute_str!(ctx, format!("{cmd} #B101010", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response, b"42\n");
            });
        }
        #[test]
        fn test_datatype_error() {
            context!(ctx, dev);
            execute_str!(ctx, format!("{cmd} (1)", cmd=$cmd).as_bytes() => result, response {
                assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
                assert_eq!(response.is_empty(), true);
            });
        }
    }
    };
}

macro_rules! test_real {
    ($test:ident; $cmd:literal, $inf:literal, $nan:literal, $min:expr, $max:expr) => {
    mod $test {
        use super::*;
        #[test]
        fn test_decimal() {
            context!(ctx, dev);
            let valid = ["1.0", "+1", "-1", "1e10", "+1.3E-1"];
            for s in &valid {
                let cmd = format!("{cmd} {value}", cmd = $cmd, value = s);
                execute_str!(ctx, cmd.as_bytes() => result, response {
                    assert_eq!(result, Ok(()));
                    assert_eq!(response.is_empty(), false);
                });
            }
        }

        #[test]
        fn test_inf() {
            context!(ctx, dev);
            let infinite = ["inf", "ninf", "INFINITY", "NINFINITY"];
            for s in &infinite {
                let cmd = format!("{cmd} {value}", cmd = $inf, value = s);
                execute_str!(ctx, cmd.as_bytes() => result, response {
                    assert_eq!(result, Ok(()));
                    assert_eq!(response.is_empty(), true);
                });
            }
        }

        #[test]
        fn test_nan() {
            context!(ctx, dev);
            let cmd = format!("{cmd} NAN", cmd = $nan);
            execute_str!(ctx, cmd.as_bytes() => result, response {
                assert_eq!(result, Ok(()));
                assert_eq!(response.is_empty(), true);
            });
        }

        #[test]
        fn test_datatype_error() {
            context!(ctx, dev);
            let cmd1 = format!("{cmd} 'STRING'", cmd = $nan);
            execute_str!(ctx, cmd1.as_bytes() => result, response {
                assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
                assert_eq!(response.is_empty(), true);
            });
            let cmd2 = format!("{cmd} INVALID", cmd = $nan);
            execute_str!(ctx, cmd2.as_bytes() => result, response {
                assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
                assert_eq!(response.is_empty(), true);
            });
        }
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
    add_numeric_command!(b"*NUM": NumCommand),
    add_numeric_command!(b"*NUMRANGE": NumRangeCommand),
    add_numeric_command!(b"*NUMLIST": NumericListCommand),
    add_numeric_command!(b"*F64": F64Command),
    add_numeric_command!(b"*F64ISINF": F64IsInfCommand),
    add_numeric_command!(b"*F64ISNAN": F64IsNanCommand),
    add_numeric_command!(b"*F32": F32Command),
    add_numeric_command!(b"*F32ISINF": F32IsInfCommand),
    add_numeric_command!(b"*F32ISNAN": F32IsNanCommand),
    add_numeric_command!(b"*BOOL": BoolCommand),
    add_numeric_command!(b"*U64": U64Command),
    add_numeric_command!(b"*I64": I64Command),
    add_numeric_command!(b"*U32": U32Command),
    add_numeric_command!(b"*I32": I32Command),
    add_numeric_command!(b"*U16": U16Command),
    add_numeric_command!(b"*I16": I16Command),
    add_numeric_command!(b"*U8": U8Command),
    add_numeric_command!(b"*I8": I8Command),
    add_numeric_command!(b"*USIZE": UsizeCommand),
    add_numeric_command!(b"*ISIZE": IsizeCommand)
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

numeric_command!(F32Command, F32IsInfCommand, F32IsNanCommand: f32);
test_real!(real_f32; "*F32?", "*F32ISINF", "*F32ISNAN", f32::MIN, f32::MAX);

numeric_command!(F64Command, F64IsInfCommand, F64IsNanCommand: f64);
test_real!(real_f64; "*F64?", "*F64ISINF", "*F64ISNAN", f64::MIN, f64::MAX);

numeric_command!(U64Command: u64);
test_integer!(integer_u64; "*U64?", u64::MIN, u64::MAX);

numeric_command!(I64Command: i64);
test_integer!(integer_i64; "*I64?", i64::MIN, i64::MAX);

numeric_command!(U32Command: u32);
test_integer!(integer_u32; "*U32?", u32::MIN, u32::MAX);

numeric_command!(I32Command: i32);
test_integer!(integer_i32; "*I32?", i32::MIN, i32::MAX);

numeric_command!(U16Command: u16);
test_integer!(integer_u16; "*U16?", u16::MIN, u16::MAX);

numeric_command!(I16Command: i16);
test_integer!(integer_i16; "*I16?", i16::MIN, i16::MAX);

numeric_command!(U8Command: u8);
test_integer!(integer_u8; "*u8?", u8::MIN, u8::MAX);

numeric_command!(I8Command: i8);
test_integer!(integer_i8; "*I8?", i8::MIN, i8::MAX);

numeric_command!(UsizeCommand: usize);
test_integer!(integer_usize; "*USIZE?", usize::MIN, usize::MAX);

numeric_command!(IsizeCommand: isize);
test_integer!(integer_isize; "*ISIZE?", isize::MIN, isize::MAX);

struct NumCommand {}
impl Command for NumCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        const MAX: f32 = 100.0;
        const MIN: f32 = -100.0;
        const DEFAULT: f32 = 0.0;
        let mut option = 0;
        let x: f32 = args.next_data(false)?.unwrap().numeric(|n| match n {
            NumericValues::Maximum => {
                option = 1;
                Ok(MAX)
            }
            NumericValues::Minimum => {
                option = 2;
                Ok(MIN)
            }
            NumericValues::Default => {
                option = 3;
                Ok(DEFAULT)
            }
            NumericValues::Up => {
                option = 4;
                Ok(1.0f32)
            }
            NumericValues::Down => {
                option = 5;
                Ok(-1.0f32)
            }
            _ => Err(ErrorCode::IllegalParameterValue.into()),
        })?;
        response.data(x).data(option).finish()
    }
}
#[test]
fn test_numeric() {
    context!(ctx, dev);
    execute_str!(ctx, b"*NUM? 42" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"42.0,0\n");
    });
    execute_str!(ctx, b"*NUM? MAX" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"100.0,1\n");
    });
    execute_str!(ctx, b"*NUM? MIN" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"-100.0,2\n");
    });
    execute_str!(ctx, b"*NUM? DEF" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0.0,3\n");
    });
    execute_str!(ctx, b"*NUM? UP" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1.0,4\n");
    });
    execute_str!(ctx, b"*NUM? DOWN" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"-1.0,5\n");
    });
}

struct NumRangeCommand {}
impl Command for NumRangeCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let x: f32 = args
            .next_data(false)?
            .unwrap()
            .numeric_range(-1f32, 1f32, |special| match special {
                NumericValues::Default => Ok(0f32),
                _ => Err(ErrorCode::IllegalParameterValue.into()),
            })?;
        response.data(x).finish()
    }
}
#[test]
fn test_numeric_range() {
    context!(ctx, dev);
    execute_str!(ctx, b"*NUMRANGE? 0.5" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0.5\n");
    });
    execute_str!(ctx, b"*NUMRANGE? MAX" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1.0\n");
    });
    execute_str!(ctx, b"*NUMRANGE? MIN" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"-1.0\n");
    });
    execute_str!(ctx, b"*NUMRANGE? 5" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataOutOfRange)));
    });
    execute_str!(ctx, b"*NUMRANGE? -5" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::DataOutOfRange)));
    });
}

numeric_command!(StrCommand: &[u8]);
mod string {
    use super::*;
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
}

numeric_command!(ArbCommand: Arbitrary);
mod arbitrary {
    use super::*;
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
}

numeric_command!(ChrCommand: Character);
mod character {
    use super::*;
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
}

struct Utf8Command {}
impl Command for Utf8Command {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let x: &str = args.next_data(false)?.unwrap().try_into()?;
        response.data(Arbitrary(x.as_bytes())).finish()
    }
}
mod utf8 {
    use super::*;
    #[test]
    fn test_utf8() {
        context!(ctx, dev);
        execute_str!(ctx, b"*UTF8? 'STRING'" => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response, b"#16STRING\n");
        });
        execute_str!(ctx, b"*UTF8? #206STRING" => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response, b"#16STRING\n");
        });
        #[cfg(feature = "arbitrary-utf8-string")]
        execute_str!(ctx, "*UTF8? #s'STRÄNG'".as_bytes() => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response, "#17STRÄNG\n".as_bytes());
        });
        execute_str!(ctx, b"*UTF8? 1.0" => result, _response {
            assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
        });
    }
}

struct NumericListCommand {}
impl Command for NumericListCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        let numbers: NumericList = args.next_data(false)?.unwrap().try_into()?;
        for item in numbers {
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

mod numericlist {
    use super::*;
    #[test]
    fn test_numeric_list() {
        context!(ctx, dev);
        execute_str!(ctx, b"*NUMLIST? (1,2,3:5)" => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response, b"1.0,2.0,3.0,5.0\n");
        });
    }
}

numeric_command!(BoolCommand: bool);
mod boolean {
    use super::*;
    #[test]
    fn test_bool_numeric() {
        context!(ctx, dev);
        //Accept only NR1 1,0
        execute_str!(ctx, b"*BOOL? 0;*BOOL? 1" => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response, b"0;1\n");
        });
        execute_str!(ctx, b"*BOOL? 1.0" => result, _response {
            assert_eq!(result, Ok(()));
        });
        execute_str!(ctx, b"*BOOL? -1" => result, _response {
            assert_eq!(result, Ok(()));
        });
    }
    #[test]
    fn test_bool_character() {
        context!(ctx, dev);
        //Accept only ON,OFF
        execute_str!(ctx, b"*BOOL? ON;*BOOL? OFF" => result, response {
            assert_eq!(result, Ok(()));
            assert_eq!(response, b"1;0\n");
        });
        execute_str!(ctx, b"*BOOL? POTATO" => result, _response {
            assert_eq!(result, Err(Error::from(ErrorCode::IllegalParameterValue)));
        });
        execute_str!(ctx, b"*BOOL? (@1)" => result, _response {
            assert_eq!(result, Err(Error::from(ErrorCode::DataTypeError)));
        });
    }
}

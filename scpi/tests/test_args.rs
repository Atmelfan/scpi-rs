// Test mandated ieee488.2 commands
use scpi::error::Result;
use scpi::prelude::*;

//Default commands
use scpi::ieee488::mandatory::*;
use scpi::scpi1999::mandatory::*;
use scpi::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, qonly, scpi_status, scpi_system,
};
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

mod util;
use scpi::format::{Arbitrary, Character};

use util::TestDevice;

extern crate std;

#[derive(Default)]
struct EchoCommand<T>(PhantomData<T>);

impl<T> EchoCommand<T> {
    const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Command<TestDevice> for EchoCommand<T>
where
    T: for<'a> TryFrom<Token<'a>, Error = Error> + Data,
{
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: T = args.next()?.try_into()?;
        response.data(x).finish()
    }
}

struct StrEchoCommand;

impl Command<TestDevice> for StrEchoCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: &[u8] = args.next()?.try_into()?;
        response.data(x).finish()
    }
}

struct ArbEchoCommand;

impl Command<TestDevice> for ArbEchoCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: Arbitrary = args.next()?.try_into()?;
        response.data(x).finish()
    }
}

struct ChrEchoCommand;

impl Command<TestDevice> for ChrEchoCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: Character = args.next()?.try_into()?;
        response.data(x).finish()
    }
}

trait InfOrNan {
    fn is_t_inf(&self) -> bool;
    fn is_t_nan(&self) -> bool;
}

impl InfOrNan for f32 {
    fn is_t_inf(&self) -> bool {
        self.is_infinite()
    }

    fn is_t_nan(&self) -> bool {
        self.is_nan()
    }
}

impl InfOrNan for f64 {
    fn is_t_inf(&self) -> bool {
        self.is_infinite()
    }

    fn is_t_nan(&self) -> bool {
        self.is_nan()
    }
}

/// Used to test floating point number conversion for (N)INFINITY values
struct IsInf<T>(PhantomData<T>);

impl<T> IsInf<T> {
    const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Command<TestDevice> for IsInf<T>
where
    T: for<'a> TryFrom<Token<'a>, Error = Error> + Data + InfOrNan,
{
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: T = args.next()?.try_into()?;
        response.data(x.is_t_inf()).finish()
    }
}

/// Used to test floating point number conversion for NAN values
struct IsNan<T>(PhantomData<T>);

impl<T> IsNan<T> {
    const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Command<TestDevice> for IsNan<T>
where
    T: for<'a> TryFrom<Token<'a>, Error = Error> + Data + InfOrNan,
{
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: T = args.next()?.try_into()?;
        response.data(x.is_t_nan()).finish()
    }
}

macro_rules! add_numeric_command {
    ($cmd:literal : $name:expr ) => {
        Leaf {
            name: $cmd,
            default: false,
            handler: $name,
        }
    };
}

macro_rules! test_integer {
    ($test:ident; $cmd:literal, $min:expr, $max:expr) => {
        mod $test {
            use super::*;
            #[test]
            fn test_integer() {
                let mut dev = TestDevice::new();
                let res = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} 42", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap();
                assert_eq!(res.as_slice(), b"42\n");
            }
            #[test]
            fn test_rounding() {
                let mut dev = TestDevice::new();
                let res = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} 0.4;{cmd} 0.6", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap();
                assert_eq!(res.as_slice(), b"0;1\n");
            }
            #[test]
            fn test_max_min() {
                let mut dev = TestDevice::new();
                let res = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} MAX;{cmd} MIN", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap();
                assert_eq!(
                    res.as_slice(),
                    format!("{max};{min}\n", max = $max, min = $min).as_bytes()
                );
            }
            #[test]
            fn test_hex() {
                let mut dev = TestDevice::new();
                let res = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} #H002A", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap();
                assert_eq!(res.as_slice(), b"42\n");
            }
            #[test]
            fn test_octal() {
                let mut dev = TestDevice::new();
                let res = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} #Q52", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap();
                assert_eq!(res.as_slice(), b"42\n");
            }
            #[test]
            fn test_binary() {
                let mut dev = TestDevice::new();
                let res = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} #B101010", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap();
                assert_eq!(res.as_slice(), b"42\n");
            }
            #[test]
            fn test_datatype_error() {
                let mut dev = TestDevice::new();
                let err = util::test_execute_str(
                    &IEEE488_TREE,
                    format!("{cmd} '42'", cmd = $cmd).as_bytes(),
                    &mut dev,
                )
                .unwrap_err();
                assert_eq!(err, Error::from(ErrorCode::DataTypeError));
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
                let mut dev = TestDevice::new();

                let valid = [("1.0", "1.0\n"), ("+1", "1.0\n"), ("-1", "-1.0\n"), ("1e10", "1.0e10\n"), ("+1.3E-1", "0.13\n")];
                for s in &valid {
                    
                    let cmd = format!("{cmd} {value}", cmd = $cmd, value = s.0);
                    let res =
                        util::test_execute_str(&IEEE488_TREE, cmd.as_bytes(), &mut dev).unwrap();
                        println!("{}=>{}", s.0, std::str::from_utf8(res.as_slice()).unwrap());
                    assert_eq!(res.as_slice(), s.1.as_bytes());
                }
            }

            #[test]
            fn test_inf() {
                let mut dev = TestDevice::new();

                let infinite = ["inf", "ninf", "INFINITY", "NINFINITY"];
                for s in &infinite {
                    let cmd = format!("{cmd} {value}", cmd = $inf, value = s);
                    let res =
                        util::test_execute_str(&IEEE488_TREE, cmd.as_bytes(), &mut dev).unwrap();
                    assert_eq!(res.as_slice(), b"1\n");
                }

                let cmd = format!("{cmd} 1.0", cmd = $nan);
                let res = util::test_execute_str(&IEEE488_TREE, cmd.as_bytes(), &mut dev).unwrap();
                assert_eq!(res.as_slice(), b"0\n");
            }

            #[test]
            fn test_nan() {
                let mut dev = TestDevice::new();

                let cmd = format!("{cmd} NAN;{cmd} 1.0", cmd = $nan);
                let res = util::test_execute_str(&IEEE488_TREE, cmd.as_bytes(), &mut dev).unwrap();
                assert_eq!(res.as_slice(), b"1;0\n");
            }

            #[test]
            fn test_datatype_error() {
                let mut dev = TestDevice::new();

                let cmd1 = format!("{cmd} 'STRING'", cmd = $nan);
                let res =
                    util::test_execute_str(&IEEE488_TREE, cmd1.as_bytes(), &mut dev).unwrap_err();
                assert_eq!(res, Error::from(ErrorCode::DataTypeError));

                let cmd2 = format!("{cmd} INVALID", cmd = $nan);
                let res =
                    util::test_execute_str(&IEEE488_TREE, cmd2.as_bytes(), &mut dev).unwrap_err();
                assert_eq!(res, Error::from(ErrorCode::DataTypeError));
            }
        }
    };
}

const IEEE488_TREE: &Node<TestDevice> = &Branch {
    name: b"",
    sub: &[
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
        add_numeric_command!(b"*STR": &StrEchoCommand),
        add_numeric_command!(b"*ARB": &ArbEchoCommand),
        add_numeric_command!(b"*CHR": &ChrEchoCommand),
        add_numeric_command!(b"*UTF8": &Utf8Command::new()),
        add_numeric_command!(b"*F64": &EchoCommand::<f64>::new()),
        add_numeric_command!(b"*F64ISINF": &IsInf::<f64>::new()),
        add_numeric_command!(b"*F64ISNAN": &IsNan::<f64>::new()),
        add_numeric_command!(b"*F32": &EchoCommand::<f32>::new()),
        add_numeric_command!(b"*F32ISINF": &IsInf::<f32>::new()),
        add_numeric_command!(b"*F32ISNAN": &IsNan::<f32>::new()),
        add_numeric_command!(b"*BOOL": &EchoCommand::<bool>::new()),
        add_numeric_command!(b"*U64": &EchoCommand::<u64>::new()),
        add_numeric_command!(b"*I64": &EchoCommand::<i64>::new()),
        add_numeric_command!(b"*U32": &EchoCommand::<u32>::new()),
        add_numeric_command!(b"*I32": &EchoCommand::<i32>::new()),
        add_numeric_command!(b"*U16": &EchoCommand::<u16>::new()),
        add_numeric_command!(b"*I16": &EchoCommand::<i16>::new()),
        add_numeric_command!(b"*U8": &EchoCommand::<u8>::new()),
        add_numeric_command!(b"*I8": &EchoCommand::<i8>::new()),
        add_numeric_command!(b"*USIZE": &EchoCommand::<usize>::new()),
        add_numeric_command!(b"*ISIZE": &EchoCommand::<isize>::new()),
    ],
};

test_real!(real_f32; "*F32?", "*F32ISINF?", "*F32ISNAN?", f32::MIN, f32::MAX);

test_real!(real_f64; "*F64?", "*F64ISINF?", "*F64ISNAN?", f64::MIN, f64::MAX);

test_integer!(integer_u64; "*U64?", u64::MIN, u64::MAX);

test_integer!(integer_i64; "*I64?", i64::MIN, i64::MAX);

test_integer!(integer_u32; "*U32?", u32::MIN, u32::MAX);

test_integer!(integer_i32; "*I32?", i32::MIN, i32::MAX);

test_integer!(integer_u16; "*U16?", u16::MIN, u16::MAX);

test_integer!(integer_i16; "*I16?", i16::MIN, i16::MAX);

test_integer!(integer_u8; "*u8?", u8::MIN, u8::MAX);

test_integer!(integer_i8; "*I8?", i8::MIN, i8::MAX);

test_integer!(integer_usize; "*USIZE?", usize::MIN, usize::MAX);

test_integer!(integer_isize; "*ISIZE?", isize::MIN, isize::MAX);

mod string {
    use super::*;
    #[test]
    fn test_str() {
        let mut dev = TestDevice::new();

        let res =
            util::test_execute_str(&IEEE488_TREE, "*STR? 'STRING'".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"\"STRING\"\n");

        let res = util::test_execute_str(&IEEE488_TREE, "*STR? CHRDATA".as_bytes(), &mut dev)
            .unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));

        let res =
            util::test_execute_str(&IEEE488_TREE, "*STR? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

mod arbitrary {
    use super::*;
    #[test]
    fn test_arb() {
        let mut dev = TestDevice::new();

        let res =
            util::test_execute_str(&IEEE488_TREE, "*ARB? #203ABC".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"#13ABC\n");

        let res = util::test_execute_str(&IEEE488_TREE, "*ARB? CHRDATA".as_bytes(), &mut dev)
            .unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));

        let res =
            util::test_execute_str(&IEEE488_TREE, "*ARB? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

mod character {
    use super::*;
    #[test]
    fn test_chr() {
        let mut dev = TestDevice::new();

        let res =
            util::test_execute_str(&IEEE488_TREE, "*CHR? CHRDATA".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"CHRDATA\n");

        let res = util::test_execute_str(&IEEE488_TREE, "*CHR? 'CHRDATA'".as_bytes(), &mut dev)
            .unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));

        let res =
            util::test_execute_str(&IEEE488_TREE, "*CHR? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

struct Utf8Command {}

impl Utf8Command {
    const fn new() -> Self {
        Self {}
    }
}

impl Command<TestDevice> for Utf8Command {
    qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: &str = args.next()?.try_into()?;
        response.data(Arbitrary(x.as_bytes())).finish()
    }
}
mod utf8 {
    //! Test parsing of UTF8 str type
    //! Parser will accept either a IEEE488 string type or arbitray data and check if the data is valid Utf8

    use super::*;
    #[test]
    fn test_utf8() {
        let mut dev = TestDevice::new();

        let res =
            util::test_execute_str(&IEEE488_TREE, "*UTF8? 'STRING'".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"#16STRING\n");

        let res = util::test_execute_str(&IEEE488_TREE, "*UTF8? #206STRING".as_bytes(), &mut dev)
            .unwrap();
        assert_eq!(res.as_slice(), b"#16STRING\n");

        let res =
            util::test_execute_str(&IEEE488_TREE, "*UTF8? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

mod boolean {
    use super::*;
    #[test]
    fn test_bool_numeric() {
        let mut dev = TestDevice::new();

        let res = util::test_execute_str(
            &IEEE488_TREE,
            "*BOOL? 0;*BOOL? 1;*BOOL? -1;*BOOL? 1.0".as_bytes(),
            &mut dev,
        )
        .unwrap();
        assert_eq!(res.as_slice(), b"0;1;1;1\n");
    }

    #[test]
    fn test_bool_character() {
        let mut dev = TestDevice::new();

        let res =
            util::test_execute_str(&IEEE488_TREE, "*BOOL? ON;*BOOL? OFF".as_bytes(), &mut dev)
                .unwrap();
        assert_eq!(res.as_slice(), b"1;0\n");

        let res = util::test_execute_str(&IEEE488_TREE, "*BOOL? POTATO".as_bytes(), &mut dev)
            .unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::IllegalParameterValue));

        let res =
            util::test_execute_str(&IEEE488_TREE, "*BOOL? (@1)".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

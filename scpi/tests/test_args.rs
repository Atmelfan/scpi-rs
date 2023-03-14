mod util;

use scpi::cmd_qonly;
use scpi::{error::Result, tree::prelude::*};
use util::TestDevice;

use std::convert::TryFrom;
use std::marker::PhantomData;

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
    T: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData,
{
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: T = params.next_data()?;
        response.data(x).finish()
    }
}

struct StrEchoCommand;

impl Command<TestDevice> for StrEchoCommand {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: &[u8] = params.next_data()?;
        response.data(x).finish()
    }
}

struct ArbEchoCommand;

impl Command<TestDevice> for ArbEchoCommand {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: Arbitrary = params.next_data()?;
        response.data(x).finish()
    }
}

struct ChrEchoCommand;

impl Command<TestDevice> for ChrEchoCommand {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: Character = params.next_data()?;
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
    T: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData + InfOrNan,
{
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: T = params.next_data()?;
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
    T: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData + InfOrNan,
{
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: T = params.next_data()?;
        response.data(x.is_t_nan()).finish()
    }
}

#[cfg(feature = "alloc")]
struct VecData;

#[cfg(feature = "alloc")]
impl Command<TestDevice> for VecData {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x = vec![1, 2, 3];
        response.data(x).finish()
    }
}

#[cfg(feature = "arrayvec")]
struct ArrayVecData;

#[cfg(feature = "arrayvec")]
impl Command<TestDevice> for ArrayVecData {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        use arrayvec::ArrayVec;

        let x = ArrayVec::from([1, 2, 3]);
        response.data(x).finish()
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
                let mut dev = TestDevice;
                let res = util::test_execute_str(
                    &TEST_TREE,
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
                    &TEST_TREE,
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
                    &TEST_TREE,
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
                    &TEST_TREE,
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
                    &TEST_TREE,
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
                    &TEST_TREE,
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
                    &TEST_TREE,
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

                let valid = [
                    ("1.0", "1.0\n"),
                    ("+1", "1.0\n"),
                    ("-1", "-1.0\n"),
                    ("1e10", "1.0e10\n"),
                    ("+1.3E-1", "0.13\n"),
                ];
                for s in &valid {
                    let cmd = format!("{cmd} {value}", cmd = $cmd, value = s.0);
                    let res = util::test_execute_str(&TEST_TREE, cmd.as_bytes(), &mut dev).unwrap();
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
                    let res = util::test_execute_str(&TEST_TREE, cmd.as_bytes(), &mut dev).unwrap();
                    assert_eq!(res.as_slice(), b"1\n");
                }

                let cmd = format!("{cmd} 1.0", cmd = $nan);
                let res = util::test_execute_str(&TEST_TREE, cmd.as_bytes(), &mut dev).unwrap();
                assert_eq!(res.as_slice(), b"0\n");
            }

            #[test]
            fn test_nan() {
                let mut dev = TestDevice::new();

                let cmd = format!("{cmd} NAN;{cmd} 1.0", cmd = $nan);
                let res = util::test_execute_str(&TEST_TREE, cmd.as_bytes(), &mut dev).unwrap();
                assert_eq!(res.as_slice(), b"1;0\n");
            }

            #[test]
            fn test_datatype_error() {
                let mut dev = TestDevice::new();

                let cmd1 = format!("{cmd} 'STRING'", cmd = $nan);
                let res =
                    util::test_execute_str(&TEST_TREE, cmd1.as_bytes(), &mut dev).unwrap_err();
                assert_eq!(res, Error::from(ErrorCode::DataTypeError));

                let cmd2 = format!("{cmd} INVALID", cmd = $nan);
                let res =
                    util::test_execute_str(&TEST_TREE, cmd2.as_bytes(), &mut dev).unwrap_err();
                assert_eq!(res, Error::from(ErrorCode::DataTypeError));
            }
        }
    };
}

const TEST_TREE: &Node<TestDevice> = &Branch {
    name: b"",
    default: false,
    sub: &[
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
        #[cfg(feature = "alloc")]
        add_numeric_command!(b"*VEC": &VecData),
        #[cfg(feature = "arrayvec")]
        add_numeric_command!(b"*ARRAYVEC": &ArrayVecData),
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

        let res = util::test_execute_str(TEST_TREE, "*STR? 'STRING'".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"\"STRING\"\n");

        let res =
            util::test_execute_str(TEST_TREE, "*STR? CHRDATA".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));

        let res = util::test_execute_str(TEST_TREE, "*STR? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

mod arbitrary {
    use super::*;
    #[test]
    fn test_arb() {
        let mut dev = TestDevice::new();

        let res = util::test_execute_str(TEST_TREE, "*ARB? #203ABC".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"#13ABC\n");

        let res =
            util::test_execute_str(TEST_TREE, "*ARB? CHRDATA".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));

        let res = util::test_execute_str(TEST_TREE, "*ARB? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

mod character {
    use super::*;
    #[test]
    fn test_chr() {
        let mut dev = TestDevice::new();

        let res = util::test_execute_str(TEST_TREE, "*CHR? CHRDATA".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"CHRDATA\n");

        let res =
            util::test_execute_str(TEST_TREE, "*CHR? 'CHRDATA'".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));

        let res = util::test_execute_str(TEST_TREE, "*CHR? 1.0".as_bytes(), &mut dev).unwrap_err();
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
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let x: &str = params.next_data()?;
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
            util::test_execute_str(TEST_TREE, "*UTF8? 'STRING'".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"#16STRING\n");

        let res =
            util::test_execute_str(TEST_TREE, "*UTF8? #206STRING".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"#16STRING\n");

        let res = util::test_execute_str(TEST_TREE, "*UTF8? 1.0".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

mod boolean {
    use super::*;
    #[test]
    fn test_bool_numeric() {
        let mut dev = TestDevice::new();

        let res = util::test_execute_str(
            TEST_TREE,
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
            util::test_execute_str(TEST_TREE, "*BOOL? ON;*BOOL? OFF".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"1;0\n");

        let res =
            util::test_execute_str(TEST_TREE, "*BOOL? POTATO".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::IllegalParameterValue));

        let res =
            util::test_execute_str(TEST_TREE, "*BOOL? (@1)".as_bytes(), &mut dev).unwrap_err();
        assert_eq!(res, Error::from(ErrorCode::DataTypeError));
    }
}

#[cfg(feature = "alloc")]
mod test_vec {
    //! Test parsing of UTF8 str type
    //! Parser will accept either a IEEE488 string type or arbitray data and check if the data is valid Utf8

    use super::*;
    #[test]
    fn test_vec() {
        let mut dev = TestDevice::new();

        let res = util::test_execute_str(TEST_TREE, "*VEC?".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"1,2,3\n");
    }
}

#[cfg(feature = "arrayvec")]
mod test_arrayvec {
    //! Test parsing of UTF8 str type
    //! Parser will accept either a IEEE488 string type or arbitray data and check if the data is valid Utf8

    use super::*;
    #[test]
    fn test_vec() {
        let mut dev = TestDevice::new();

        let res = util::test_execute_str(TEST_TREE, "*ARRAYVEC?".as_bytes(), &mut dev).unwrap();
        assert_eq!(res.as_slice(), b"1,2,3\n");
    }
}

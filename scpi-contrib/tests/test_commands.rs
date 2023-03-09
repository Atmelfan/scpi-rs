// Test mandated ieee488.2 commands
use scpi::error::Result;
use scpi::prelude::*;

//Default commands

use scpi::{cmd_nquery, cmd_qonly, tree::prelude::*};
use scpi_contrib::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, scpi1999::prelude::*, scpi_status, scpi_system,
};

mod util;
use util::TestDevice;

extern crate std;

const IEEE488_TREE: Node<TestDevice> = Branch {
    name: b"",
    default: false,
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
        Leaf {
            name: b"*ERR",
            default: false,
            handler: &ErrorCommand,
        },
        Leaf {
            name: b"*OPER",
            default: false,
            handler: &OperCommand {},
        },
        Leaf {
            name: b"*QUES",
            default: false,
            handler: &QuesCommand {},
        },
        Leaf {
            name: b"*QUERY",
            default: false,
            handler: &QueryCommand {},
        },
        Leaf {
            name: b"*EVENT",
            default: false,
            handler: &EventCommand {},
        },
    ],
};

struct ErrorCommand;

impl Command<TestDevice> for ErrorCommand {
    cmd_nquery!();

    fn event(
        &self,
        device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
    ) -> Result<()> {
        let errcode: i16 = params.next_data()?;
        device.handle_error(
            ErrorCode::get_error(errcode)
                .unwrap_or(ErrorCode::Custom(errcode, b"Custom Error"))
                .into(),
        );
        Ok(())
    }
}

struct OperCommand {}

impl Command<TestDevice> for OperCommand {
    cmd_nquery!();

    fn event(
        &self,
        device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
    ) -> Result<()> {
        let condition: u16 = params.next_data()?;
        device
            .get_register_mut::<Operation>()
            .set_condition(condition);
        Ok(())
    }
}

struct QuesCommand {}

impl Command<TestDevice> for QuesCommand {
    cmd_nquery!();

    fn event(
        &self,
        device: &mut TestDevice,
        _context: &mut Context,
        mut params: Parameters,
    ) -> Result<()> {
        let condition: u16 = params.next_data()?;
        device
            .get_register_mut::<Questionable>()
            .set_condition(condition);

        Ok(())
    }
}

struct QueryCommand {}

impl Command<TestDevice> for QueryCommand {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(0i32).finish()
    }
}

struct EventCommand {}

impl Command<TestDevice> for EventCommand {
    cmd_nquery!();

    fn event(
        &self,
        _device: &mut TestDevice,
        _context: &mut Context,
        _params: Parameters,
    ) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_qonly() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*query", &mut dev).unwrap_err();
    assert_eq!(res, Error::from(ErrorCode::UndefinedHeader));

    let res = util::test_execute_str(&IEEE488_TREE, b"*query?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");
}

#[test]
fn test_nquery() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*event?", &mut dev).unwrap_err();
    assert_eq!(res, Error::from(ErrorCode::UndefinedHeader));

    let res = util::test_execute_str(&IEEE488_TREE, b"*event", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"");
}

#[test]
fn test_cls() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR 1;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"8\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"*CLS;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");
}

#[test]
fn test_rst() {
    let mut dev = TestDevice::new();
    let _res = util::test_execute_str(&IEEE488_TREE, b"*RST", &mut dev).unwrap();
}

#[test]
fn test_tst() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*TST?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");
}

#[test]
fn test_ese() {
    //Test setting/getting event status enable register
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*ESE?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"*ESE 255", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"");

    let res = util::test_execute_str(&IEEE488_TREE, b"*ESE?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"255\n");
}

#[test]
fn test_sre() {
    //Test setting/getting service request enable register
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*SRE?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"*SRE 255", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"");

    let res = util::test_execute_str(&IEEE488_TREE, b"*SRE?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"255\n");
}

#[test]
fn test_opc() {
    // Do not support overlapped commands so
    // OPC is a NOP command
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*RST;*OPC?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"1\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"*RST;*OPC;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"1\n");
}

#[test]
fn test_wai() {
    // Do not support overlapped commands so
    // WAI is a NOP command
    let mut dev = TestDevice::new();

    let _res = util::test_execute_str(&IEEE488_TREE, b"*WAI", &mut dev).unwrap();
}

#[test]
fn test_esr() {
    // Test ESR register getting set by errors
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");

    //Command error
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -100;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"32\n");
    //Execution error
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -200;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"16\n");
    //Device-specific error
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -300;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"8\n");
    //Query error
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -400;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"4\n");
    //Power-on event
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -500;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"128\n");
    //User request event
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -600;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"64\n");
    //Request control
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -700;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"2\n");
    //Operation complete
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -800;*ESR?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"1\n");
}

#[test]
fn test_stb() {
    let mut dev = TestDevice::new();
    let res = util::test_execute_str(&IEEE488_TREE, b"*STB?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");

    // Trigger a command error, ESE is cleared so it doesn't show up in STB
    // Error queue is not empty so it shows up
    let res = util::test_execute_str(&IEEE488_TREE, b"*ERR -100;*STB?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"4\n");

    // Enable command error in ESE
    let res = util::test_execute_str(&IEEE488_TREE, b"*ESE 32;*STB?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"36\n");

    //Enable Service request (for error queue bit) and see that ESB, error queue bit
    let res = util::test_execute_str(&IEEE488_TREE, b"*SRE 4;*STB?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"100\n");

    //Clear esr, MSS should still be set
    let res = util::test_execute_str(&IEEE488_TREE, b"*ESR?;*STB?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"32;68\n");

    // Clear error
    let _res = util::test_execute_str(&IEEE488_TREE, b"SYST:ERR?", &mut dev).unwrap();

    // STB is cleared as no errors are in queue
    let res = util::test_execute_str(&IEEE488_TREE, b"*STB?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0\n");
}

#[test]
fn test_syst_err() {
    // Test SYSTem:ERRor commands
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"syst:err:next?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0,\"No error\"\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"*err -100;syst:err?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"-100,\"Command error\"\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"*err -100;*err -200;syst:err:count?;all?",
        &mut dev,
    )
    .unwrap();
    assert_eq!(
        res.as_slice(),
        b"2;-100,\"Command error\",-200,\"Execution error\"\n"
    );

    let res = util::test_execute_str(&IEEE488_TREE, b"syst:err:next?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0,\"No error\"\n");
}

#[test]
fn test_syst_version() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(&IEEE488_TREE, b"syst:vers?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"1999.0\n");
}

#[test]
fn test_stat_operation() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"stat:oper:cond?;ptr #H00FF;ntr #HFF00",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"0\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"*oper #HFFFF;stat:oper:cond?;event?",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"32767;255\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"*oper #H0000;stat:oper:cond?;event?",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"0;32512\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"stat:oper:cond?;event?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0;0\n");

    //Check enable/enable?
    let res =
        util::test_execute_str(&IEEE488_TREE, b"stat:oper:enable 65535;enable?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"32767\n");

    //Check that operation summary bit is set in STB
    let res = util::test_execute_str(&IEEE488_TREE, b"*stb?;*oper #HFFFF;*stb?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0;128\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"stat:preset;:stat:oper:enable?;ptr?;ntr?",
        &mut dev,
    )
    .unwrap();
    println!("{:?}", std::str::from_utf8(res.as_slice()).unwrap());
    assert_eq!(res.as_slice(), b"0;32767;0\n");
}

#[test]
fn test_stat_questionable() {
    let mut dev = TestDevice::new();

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"stat:ques:cond?;ptr #H00FF;ntr #HFF00",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"0\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"*ques #HFFFF;stat:ques:cond?;event?",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"32767;255\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"*ques #H0000;stat:ques:cond?;event?",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"0;32512\n");

    let res = util::test_execute_str(&IEEE488_TREE, b"stat:ques:cond?;event?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0;0\n");

    //Check enable/enable?
    let res =
        util::test_execute_str(&IEEE488_TREE, b"stat:ques:enable 65535;enable?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"32767\n");

    //Check that operation summary bit is set in STB
    let res = util::test_execute_str(&IEEE488_TREE, b"*stb?;*ques #HFFFF;*stb?", &mut dev).unwrap();
    assert_eq!(res.as_slice(), b"0;8\n");

    let res = util::test_execute_str(
        &IEEE488_TREE,
        b"stat:preset;:stat:ques:enable?;ptr?;ntr?",
        &mut dev,
    )
    .unwrap();
    assert_eq!(res.as_slice(), b"0;32767;0\n");
}

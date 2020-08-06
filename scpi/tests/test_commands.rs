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

extern crate std;

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
    Node {
        name: b"*ERR",
        optional: false,
        handler: Some(&ErrorCommand {}),
        sub: None,
    },
    Node {
        name: b"*OPER",
        optional: false,
        handler: Some(&OperCommand {}),
        sub: None,
    },
    Node {
        name: b"*QUES",
        optional: false,
        handler: Some(&QuesCommand {}),
        sub: None,
    },
    Node {
        name: b"*QUERY",
        optional: false,
        handler: Some(&QueryCommand {}),
        sub: None,
    },
    Node {
        name: b"*EVENT",
        optional: false,
        handler: Some(&EventCommand {}),
        sub: None,
    }
];

struct ErrorCommand {}

impl Command for ErrorCommand {
    nquery!();

    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        let errcode: i16 = args.next_data(false)?.unwrap().try_into()?;
        context.push_error(
            ErrorCode::get_error(errcode)
                .unwrap_or(ErrorCode::Custom(errcode, b""))
                .into(),
        );
        Ok(())
    }
}

struct OperCommand {}

impl Command for OperCommand {
    nquery!();

    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        let condition: u16 = args.next_data(false)?.unwrap().try_into()?;
        context.operation.set_condition(condition);
        Ok(())
    }
}

struct QuesCommand {}

impl Command for QuesCommand {
    nquery!();

    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        let condition: u16 = args.next_data(false)?.unwrap().try_into()?;
        context.questionable.set_condition(condition);
        Ok(())
    }
}

struct QueryCommand {}

impl Command for QueryCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response.data(0i32).finish()
    }
}

struct EventCommand {}

impl Command for EventCommand {
    nquery!();

    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        Ok(())
    }
}

struct TestDevice {
    pub cls: bool,
    pub rst: bool,
    pub tst: bool,
}

impl TestDevice {
    pub fn new() -> Self {
        TestDevice {
            cls: false,
            rst: false,
            tst: false,
        }
    }
}

impl Device for TestDevice {
    fn cls(&mut self) -> Result<()> {
        self.cls = true;
        Ok(())
    }

    fn rst(&mut self) -> Result<()> {
        self.rst = true;
        Ok(())
    }

    fn tst(&mut self) -> Result<()> {
        self.tst = true;
        Ok(())
    }
}

#[test]
fn test_qonly() {
    context!(ctx, dev);
    execute_str!(ctx, b"*query?;*query" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::UndefinedHeader)));
    });
}

#[test]
fn test_nquery() {
    context!(ctx, dev);
    execute_str!(ctx, b"*event;*event?" => result, _response {
        assert_eq!(result, Err(Error::from(ErrorCode::UndefinedHeader)));
    });
}

#[test]
fn test_cls() {
    context!(ctx, dev);
    execute_str!(ctx, b"*cls" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response.is_empty(), true);
    });
    check_esr!(ctx);
    assert_eq!(dev.cls, true);
}

#[test]
fn test_rst() {
    context!(ctx, dev);
    execute_str!(ctx, b"*rst" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response.is_empty(), true);
    });
    check_esr!(ctx);
    assert_eq!(dev.rst, true);
}

#[test]
fn test_tst() {
    //
    context!(ctx, dev);
    execute_str!(ctx, b"*tst?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0\n");
    });
    check_esr!(ctx);
    assert_eq!(dev.tst, true);
}

#[test]
fn test_ese() {
    //Test setting/getting event status enable register
    context!(ctx, dev);
    execute_str!(ctx, b"*ese?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0\n");
    });
    execute_str!(ctx, b"*ese 255" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response.is_empty(), true);
    });
    execute_str!(ctx, b"*ese?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"255\n");
    });
}

#[test]
fn test_sre() {
    //Test setting/getting service request enable register
    context!(ctx, dev);
    execute_str!(ctx, b"*sre?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0\n");
    });
    execute_str!(ctx, b"*sre 255" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response.is_empty(), true);
    });
    execute_str!(ctx, b"*sre?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"255\n");
    });
}

#[test]
fn test_opc() {
    // Do not support overlapped commands so
    // OPC is a NOP command
    context!(ctx, dev);
    execute_str!(ctx, b"*rst;*opc?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1\n");
    });
    execute_str!(ctx, b"*rst;*opc;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1\n");
    });
}

#[test]
fn test_wai() {
    // Do not support overlapped commands so
    // WAI is a NOP command
    context!(ctx, dev);
    execute_str!(ctx, b"*wai" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response.is_empty(), true);
    });
}

#[test]
fn test_esr() {
    // Test ESR register getting set by errors
    context!(ctx, dev);
    execute_str!(ctx, b"*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0\n");
    });
    //Command error
    execute_str!(ctx, b"*err -100;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"32\n");
    });
    //Execution error
    execute_str!(ctx, b"*err -200;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"16\n");
    });
    //Device-specific error
    execute_str!(ctx, b"*err -300;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"8\n");
    });
    //Query error
    execute_str!(ctx, b"*err -400;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"4\n");
    });
    //Power-on event
    execute_str!(ctx, b"*err -500;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"128\n");
    });
    //User request event
    execute_str!(ctx, b"*err -600;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"64\n");
    });
    //Request control
    execute_str!(ctx, b"*err -700;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"2\n");
    });
    //Operation complete
    execute_str!(ctx, b"*err -800;*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1\n");
    });
}

#[test]
fn test_stb() {
    // STB should be zero
    context!(ctx, dev);
    execute_str!(ctx, b"*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"16\n");
    });
    //Trigger a command error and se that MAV is set
    execute_str!(ctx, b"*err -100;*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"20\n");
    });
    //Enable Command error in ESE and see that ESB, error queue bit and MA is set
    execute_str!(ctx, b"*ese 32;*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"52\n");
    });
    //Enable Service request (for error queue bit) and see that ESB, error queue bit and MA is set
    execute_str!(ctx, b"*sre 4;*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"116\n");
    });
    //Clear esr, MAV and MSS should still be set
    execute_str!(ctx, b"*esr?;*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"32;84\n");
    });
    // Clear error
    execute_str!(ctx, b"syst:err?" => result, _response {
        assert_eq!(result, Ok(()));
    });
    // MAV is set
    execute_str!(ctx, b"*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"16\n");
    });
}

#[test]
fn test_syst_err() {
    // Test SYSTem:ERRor commands
    context!(ctx, dev);
    execute_str!(ctx, b"syst:err:next?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0,\"No error\"\n");
    });
    execute_str!(ctx, b"*err -100;syst:err?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"-100,\"Command error\"\n");
    });
    execute_str!(ctx, b"*err -100;*err -200;syst:err:count?;all?" => result, response {
        assert_eq!(result, Ok(()));
        println!("{:?}\n{:?}",b"2;-100,\"Command error\",-200,\"Execution error\"\n", response);
        assert_eq!(response.eq_ignore_ascii_case(b"2;-100,\"Command error\",-200,\"Execution error\"\n"), true);
    });
}

#[test]
fn test_syst_version() {
    context!(ctx, dev);
    execute_str!(ctx, b"syst:version?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1999.0\n");
    });
}

#[test]
fn test_stat_operation() {
    context!(ctx, dev);
    execute_str!(ctx, b"stat:oper:cond?;ptr #H00FF;ntr #HFF00" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0\n");
    });
    execute_str!(ctx, b"*oper #HFFFF;stat:oper:cond?;event?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"32767;255\n");
    });
    execute_str!(ctx, b"*oper #H0000;stat:oper:cond?;event?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0;32512\n");
    });
    execute_str!(ctx, b"stat:oper:cond?;event?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0;0\n");
    });
    //Check enable/enable?
    execute_str!(ctx, b"stat:oper:enable 65535;enable?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"32767\n");
    });
    //Check that operation summary bit is set in STB
    execute_str!(ctx, b"*stb?;*oper #HFFFF;*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"16;144\n");
    });
    execute_str!(ctx, b"stat:preset;:stat:oper:enable?;ptr?;ntr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0;32767;0\n");
    });
}

#[test]
fn test_stat_questionable() {
    context!(ctx, dev);
    execute_str!(ctx, b"stat:ques:cond?;ptr #H00FF;ntr #HFF00" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0\n");
    });
    execute_str!(ctx, b"*ques #HFFFF;stat:ques:cond?;event?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"32767;255\n");
    });
    execute_str!(ctx, b"*ques #H0000;stat:ques:cond?;event?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0;32512\n");
    });
    execute_str!(ctx, b"stat:ques:cond?;event?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0;0\n");
    });
    //Check enable/enable?
    execute_str!(ctx, b"stat:ques:enable 65535;enable?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"32767\n");
    });
    //Check that operation summary bit is set in STB
    execute_str!(ctx, b"*stb?;*ques #HFFFF;*stb?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"16;24\n");
    });
    execute_str!(ctx, b"stat:preset;:stat:ques:enable?;ptr?;ntr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"0;32767;0\n");
    });
}

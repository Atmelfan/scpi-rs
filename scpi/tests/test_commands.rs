// Test mandated ieee488.2 commands
use scpi::error::Result;
use scpi::prelude::*;

//Default commands
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, nquery, scpi_status, scpi_system, scpi_tree,
};
use std::convert::TryInto;

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

macro_rules! context {
    ($context:ident, $dev:ident) => {
        // Infrastructure
        let mut $dev = TestDevice::new();
        let mut errors = ArrayErrorQueue::<[Error; 10]>::new();
        let mut $context = Context::new(&mut $dev, &mut errors, IEEE488_TREE);
    };
}

macro_rules! execute_str {
    ($context:expr, $s:expr => $res:ident, $dat:ident $x:tt) => {
        //Response bytebuffer
        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();
        //SCPI tokenizer
        let mut tokenizer = Tokenizer::new($s);
        //Result
        let $res = $context.exec(&mut tokenizer, &mut buf);
        let $dat = buf.as_slice();
        $x;
    };
}

macro_rules! check_esr {
    ($context:ident == $esr:literal) => {
    execute_str!($context, b"*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, $esr);
    });
    };
    ($context:ident) => {
    check_esr!($context == b"0\n");
    };
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
    //All 8 errors logged?
    execute_str!(ctx, b"syst:err:count?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"8\n");
    });
}

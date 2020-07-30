// Test mandated ieee488.2 commands
use scpi::error::Result;
use scpi::prelude::*;

//Default commands
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, scpi_status, scpi_system, scpi_tree,
};

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
    scpi_system!()
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
}

#[test]
fn test_branching() {
    // Test that parsing continues on the current branch
    context!(ctx, dev);
    execute_str!(ctx, b"syst:version?;err:next?;count?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1999.0;0,\"No error\";0\n");
    });
}

#[test]
fn test_root() {
    // Test that parser can continue from root
    context!(ctx, dev);
    execute_str!(ctx, b"syst:version?;:syst:err:next?;count?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1999.0;0,\"No error\";0\n");
    });
}

#[test]
fn test_simple() {
    // Test that simple commands does not change branch
    context!(ctx, dev);
    execute_str!(ctx, b"syst:version?;*ese?;err:next?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1999.0;0;0,\"No error\"\n");
    });
}

#[test]
fn test_default() {
    // Test that default commands does not change branch
    context!(ctx, dev);
    execute_str!(ctx, b"syst:version?;err?;version?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, b"1999.0;0,\"No error\";1999.0\n");
    });
}

// Util macros to setup context and tree
#[macro_export]
macro_rules! context {
    ($context:ident, $dev:ident) => {
        // Infrastructure
        let mut $dev = TestDevice::new();
        let mut errors = ArrayErrorQueue::<[Error; 10]>::new();
        let mut $context = Context::new(&mut $dev, &mut errors, IEEE488_TREE);
    };
}

#[macro_export]
macro_rules! execute_str {
    ($context:expr, $s:expr => $res:ident, $dat:ident $x:tt) => {
        //Response bytebuffer
        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();
        //Result
        let $res = $context.run($s, &mut buf);
        let $dat = buf.as_slice();
        $x;
    };
}


pub use scpi::error::Result;
pub use scpi::prelude::*;
//Default commands
pub use scpi::ieee488::commands::*;
pub use scpi::scpi::commands::*;
pub use scpi::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, nquery, qonly, scpi_status, scpi_system, scpi_tree,
};

pub const IEEE488_TREE: &Node = scpi_tree![
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

pub struct TestDevice {}

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

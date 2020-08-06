#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate scpi;

mod fuzz_util;
use fuzz_util::*;

use scpi::error::Result;
use scpi::prelude::*;
//Default commands
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, nquery, qonly, scpi_status, scpi_system, scpi_tree,
};
use scpi::tokenizer::{Arbitrary, Character, NumericValues};

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

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    context!(ctx, dev);
    execute_str!(ctx, data => _result, _response {
        //Don't care about results, just don't crash
    });
});

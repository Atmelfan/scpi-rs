#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate scpi;

mod fuzz_common;
use fuzz_common::*;

use scpi::expression::numeric_list::{NumericList};


fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut numbers = NumericList::new(data);
    while let Some(Ok(_)) = numbers.next() { }
});

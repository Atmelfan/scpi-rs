#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate scpi;

use scpi::parser::expression::numeric_list::{NumericList};


fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut numbers = NumericList::new(data);
    while let Some(Ok(_)) = numbers.next() { }
});

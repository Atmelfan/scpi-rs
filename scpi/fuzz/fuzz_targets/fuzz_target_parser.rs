#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate scpi;

mod fuzz_common;
use fuzz_common::*;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    context!(ctx, dev);
    execute_str!(ctx, data => _result, _response {
        //Don't care about results, just don't crash
    });
});

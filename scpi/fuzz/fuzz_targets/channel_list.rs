#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate scpi;

mod fuzz_common;
use fuzz_common::*;

use scpi::expression::channel_list::{ChannelList};


fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    if let Some(mut channels) = ChannelList::new(data) {
        while let Some(Ok(_)) = channels.next() { }
    }
});

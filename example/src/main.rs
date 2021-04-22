#![cfg_attr(feature = "no_std", no_std)]

use scpi::prelude::*;

#[cfg(not(feature = "no_std"))]
use std::{
    io::{self, BufRead},
    str,
};

mod common;
use common::*;

fn main() {
    let mut my_device = MyDevice {};

    let mut errors = ArrayErrorQueue::<[Error; 10]>::new();
    let mut context = Context::new(&mut my_device, &mut errors, TREE);

    //Response bytebuffer
    let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

    #[cfg(not(feature = "no_std"))]
    {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let message = line.unwrap();

            //Result
            let result = context.run(message.as_bytes(), &mut buf);

            if let Err(err) = result {
                println!("{}", str::from_utf8(err.get_message()).unwrap());
            } else {
                print!("{}", str::from_utf8(buf.as_slice()).unwrap());
                //break;
            }
            //}
        }
    }
    #[cfg(feature = "no_std")]
    {
        // Dummy to test no_std compiles
        context.run(b"*idn?", &mut buf).unwrap();
        assert!(!buf.is_empty())
    }
}

use scpi::prelude::*;
use std::io;
use std::io::BufRead;

mod common;
use common::*;

fn main() {
    let mut my_device = MyDevice {};

    let mut errors = ArrayErrorQueue::<[Error; 10]>::new();
    let mut context = Context::new(&mut my_device, &mut errors, TREE);

    //Response bytebuffer
    let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let message = line.unwrap();

        //Result
        let result = context.run(message.as_bytes(), &mut buf);

        use std::str;
        if let Err(err) = result {
            println!("{}", str::from_utf8(err.get_message()).unwrap());
        } else {
            print!("{}", str::from_utf8(buf.as_slice()).unwrap());
            //break;
        }
        //}
    }
}

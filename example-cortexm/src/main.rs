#![no_std]
#![no_main]
extern crate panic_halt;

struct MyDevice;

//use strum::EnumMessage;
use scpi::command::Command;
use scpi::Device;
use scpi::tree::Node;
use scpi::tokenizer::Tokenizer;
use scpi::error::{Error, ErrorQueue, ArrayErrorQueue};
use scpi::response::{Formatter, ArrayVecFormatter};
use scpi::Context;
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;

use core::str;

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

impl Device for MyDevice {
    fn cls(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn rst(&mut self) -> Result<(), Error> {
        Ok(())
    }

}

#[entry]
fn main() -> ! {

    let mut my_device = MyDevice { };

    let mut tree = Node {name: b"ROOT", optional: true, handler: None, sub: Some(&[
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
        // Create default SCPI mandated STATus subsystem
        scpi_status!(),
        // Create default SCPI mandated SYSTem subsystem
        scpi_system!(),
        Node {name: b"SENSe", optional: true,
            handler: None,
            sub: Some(&[
                Node {name: b"VOLTage", optional: false,
                    handler: None,
                    sub: Some(&[
                        Node {name: b"DC", optional: true,
                            handler: Some(&SensVoltDcCommand{}),
                            sub: None
                        }
                    ])
                }
            ])
        },

    ])};



    let mut errors = ArrayErrorQueue::<[Error; 10]>::new();

    let mut context = Context::new(&mut my_device, &mut errors, &mut tree);

    let messages = ["*IDN?; *ESR?", "this:is:an:invalid:command?", "*ESR?", "syst:err?", "system:error?"];

    for message in messages.iter() {
        //Response bytebuffer
        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

        hprintln!("SCPI >> {}", message);

        //SCPI tokenizer
        let mut tokenizer = Tokenizer::from_str(message.as_bytes());

        //Result
        let result = context.exec(&mut tokenizer, &mut buf);

        if let Err(err) = result {
            hprintln!("ERROR: {}\n", str::from_utf8(err.get_message().unwrap()).unwrap());
        } else if !buf.is_empty() {
            hprintln!("RESPONSE: {}", str::from_utf8(buf.as_slice()).unwrap());
        }
    }


    panic!();
}

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
use scpi::ieee488::Context;
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

    fn oper_event(&self) -> u16 {
        unimplemented!()
    }

    fn oper_condition(&self) -> u16 {
        unimplemented!()
    }

    fn ques_event(&self) -> u16 {
        unimplemented!()
    }

    fn ques_condition(&self) -> u16 {
        unimplemented!()
    }
}

#[entry]
fn main() -> ! {

    let mut my_device = MyDevice { };

    let mut tree = Node {name: b"ROOT", optional: true, handler: None, sub: Some(&[
            Node {name: b"*IDN", optional: false,
                handler: Some(&IdnCommand{
                    manufacturer: b"GPA-Robotics",
                    model: b"T800",
                    serial: b"101",
                    firmware: b"0"
                }),
                sub: None
            },
            Node {name: b"*RST", optional: false,
                handler: Some(&RstCommand{}),
                sub: None
            },
            Node {name: b"*CLS", optional: false,
                handler: Some(&ClsCommand{}),
                sub: None
            },
            Node {name: b"*ESE", optional: false,
                handler: Some(&EseCommand{}),
                sub: None
            },
            Node {name: b"*ESR", optional: false,
                handler: Some(&EsrCommand{}),
                sub: None
            },
            Node {name: b"SYSTem", optional: false,
                handler: None,
                sub: Some(&[
                    Node {name: b"ERRor", optional: false,
                        handler: None,
                        sub: Some(&[
                            Node {name: b"ALL", optional: false,
                                handler: Some(&SystErrAllCommand{}),
                                sub: None
                            },
                            Node {name: b"NEXT", optional: true,
                                handler: Some(&SystErrNextCommand{}),
                                sub: None
                            },
                            Node {name: b"COUNt", optional: false,
                                handler: Some(&SystErrCounCommand{}),
                                sub: None
                            }

                        ])
                    },
                    Node {name: b"VERSion", optional: false,
                        handler: Some(&SystVersCommand{
                            year: 1999,
                            rev: 0
                        }),
                        sub: None
                    }
                ])
            }
        ])
    };



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

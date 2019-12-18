struct MyDevice;

//use strum::EnumMessage;
use scpi::command::Command;
use scpi::{Context, Device};
use scpi::commands::*;
use scpi::tree::Node;
use scpi::tokenizer::Tokenizer;
use scpi::error::Error;
use scpi::response::{Formatter, ArrayVecFormatter};




struct SensVoltDcCommand;
impl Command for SensVoltDcCommand {
    fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
        args.next_data(true)?;
        Ok(())
    }

    fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        context.response.ascii_data(b"[:SENSe]:VOLTage[:DC]?")
    }
}

struct SensVoltAcCommand;
impl Command for SensVoltAcCommand {
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        Err(Error::UndefinedHeader)
    }

    fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        context.response.ascii_data(b"[:SENSe]:VOLTage:AC?")
    }
}

impl Device for MyDevice {
    fn cls(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn rst(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn error_enqueue(&self, _err: Error) -> Result<(), Error> {
        Ok(())
    }

    fn error_dequeue(&self) -> Error {
        Error::NoError
    }

    fn error_len(&self) -> u32 {
        0
    }

    fn error_clear(&self) {

    }

    fn oper_status(&self) -> u16 {
        unimplemented!()
    }

    fn ques_status(&self) -> u16 {
        unimplemented!()
    }

}


fn main(){

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
        Node {name: b"SENSe", optional: true,
            handler: None,
            sub: Some(&[
                Node {name: b"VOLTage", optional: false,
                    handler: None,
                    sub: Some(&[
                        Node {name: b"DC", optional: true,
                            handler: Some(&SensVoltDcCommand{}),
                            sub: None
                        },
                        Node {name: b"AC", optional: false,
                            handler: Some(&SensVoltAcCommand{}),
                            sub: None
                        }
                    ])
                },
                Node {name: b"CURRent", optional: false,
                    handler: None,
                    sub: Some(&[
                        Node {name: b"DC", optional: true,
                            handler: None,
                            sub: None
                        },
                        Node {name: b"AC", optional: false,
                            handler: None,
                            sub: None
                        }
                    ])
                }
            ])
        }])
    };

    //
    let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

    let mut context = Context::new(&mut my_device, &mut buf, &mut tree);

    let mut tokenizer = Tokenizer::from_str(b"VOLT?; *IDN? ; *RST; VOLT:AC?; :sense:voltage:dc?; *ESE 128; *ESE?");

    let result = context.exec(&mut tokenizer);

    use std::str;
    println!("{}", str::from_utf8(buf.as_slice()).unwrap());



}
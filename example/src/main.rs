struct MyDevice;

//use strum::EnumMessage;
use scpi::command::Command;
use scpi::Device;
use scpi::tree::Node;
use scpi::tokenizer::{Tokenizer, Token};
use scpi::error::{Error, ErrorQueue, ArrayErrorQueue};
use scpi::response::{Formatter, ArrayVecFormatter};
use std::io;
use std::io::BufRead;
use scpi::ieee488::Context;
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use std::convert::TryInto;


struct SensVoltDcCommand;
impl Command for SensVoltDcCommand {
    fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
        let x: f32 = args.next_data(false)?.unwrap().map_special(|special| match special {
            x if Token::mnemonic_compare(b"MINimum", x) => Ok(Token::DecimalNumericProgramData(-34.0)),
            x if Token::mnemonic_compare(b"MAXimum", x) => Ok(Token::DecimalNumericProgramData(54564.0)),
            x if Token::mnemonic_compare(b"DEFault", x) => Ok(Token::DecimalNumericProgramData(0.0)),
            _ => Err(Error::IllegalParameterValue)
        })?.try_into()?;
        //x *= args.next_suffix_multiplier(Unit::Volt)?;//If no suffix (1.0), else (SUFFIX/Volt). If incompatible, error.
        println!("Value: {}", x);
        Ok(())
    }

    fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error> {
        response.ascii_data(b"[:SENSe]:VOLTage[:DC]?")
    }
}

struct SensVoltAcCommand;
impl Command for SensVoltAcCommand {
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        Err(Error::UndefinedHeader)
    }

    fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error> {
        response.ascii_data(b"[:SENSe]:VOLTage:AC?")
    }
}

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



    let mut errors = ArrayErrorQueue::<[Error; 10]>::new();

    let mut context = Context::new(&mut my_device, &mut errors, &mut tree);


    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let message = line.unwrap();

        //Response bytebuffer
        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

        //SCPI tokenizer
        let mut tokenizer = Tokenizer::from_str(message.as_bytes());

        loop {
            //Result
            let result = context.exec(&mut tokenizer, &mut buf);

            //Looka like a lot of stuff being allocated but everything is on the stack and lightweight
            use std::str;
            if let Err(err) = result {
                println!("{}", str::from_utf8(err.get_message().unwrap()).unwrap());
            } else {
                print!("{}", str::from_utf8(buf.as_slice()).unwrap());
                break;
            }
        }


    }




}
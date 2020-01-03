struct MyDevice;

//use strum::EnumMessage;
use scpi::response::{Formatter, ArrayVecFormatter};
use std::io;
use std::io::BufRead;
use scpi::prelude::*;

//Default commands
use scpi::ieee488::commands::*;
use scpi::scpi::commands::*;
use scpi::{
    ieee488_cls,
    ieee488_ese,
    ieee488_esr,
    ieee488_idn,
    ieee488_opc,
    ieee488_rst,
    ieee488_sre,
    ieee488_stb,
    ieee488_tst,
    ieee488_wai,
    scpi_status,
    scpi_system
};
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
        Ok(())
    }

    fn rst(&mut self) -> Result<(), Error> {
        Ok(())
    }

}


fn main(){

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
        },


        ])
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
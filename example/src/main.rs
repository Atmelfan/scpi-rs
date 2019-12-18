struct MyDevice;

//use strum::EnumMessage;
use scpi::command::Command;
use scpi::{Context, Device};
use scpi::commands::IdnCommand;
use scpi::tree::Node;
use scpi::tokenizer::Tokenizer;
use scpi::error::Error;
use core::fmt;



struct RstCommand;
impl Command for RstCommand {
    fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        writeln!(context.writer, "*RST").unwrap();
        context.device.rst()
    }

    fn query(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        Err(Error::UndefinedHeader)
    }
}

struct SensVoltDcCommand;
impl Command for SensVoltDcCommand {
    fn event(&self, _context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
        args.next_data(true)?;
        Ok(())
    }

    fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        writeln!(context.writer, "SENSe:VOLTage:DC?").unwrap();
        Ok(())
    }
}

struct SensVoltAcCommand;
impl Command for SensVoltAcCommand {
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        Err(Error::UndefinedHeader)
    }

    fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        writeln!(context.writer, "SENSe:VOLTage:AC?").unwrap();
        Ok(())
    }
}

impl Device for MyDevice {
    fn cls(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn rst(&mut self) -> Result<(), Error> {
        println!("Device reset");
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

struct MyWriter {
    
}

impl fmt::Write for MyWriter {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
       print!("{}", s);
        Ok(())
    }
}


fn main(){

    let command = b"VOLT?; *IDN? ; *RST; VOLT:AC?; :sense:voltage:dc?";

    let mut my_device = MyDevice { };

    let mut tree = root![
        Node {name: b"*IDN", optional: false,
            handler: Some(idn!(b"GPA-Robotics", b"SCPI-RS")),
            sub: None
        },
        Node {name: b"*RST", optional: false,
            handler: Some(&RstCommand{}),
            sub: None
        },
        Node {name: b"*CLS", optional: false,
            handler: Some(&RstCommand{}),
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
        }
    ];

    let mut writer = MyWriter{};

    let mut context = Context::new(&mut my_device, &mut writer, &mut tree);

    let mut tokenizer = Tokenizer::from_str(command);

    let result = context.exec(&mut tokenizer);

    if let Err(err) = result {
        println!("Command result: {}", String::from_utf8(err.get_message().unwrap().to_vec()).unwrap());
    }else{
        println!("Command result: Success");
    }



}
mod util;
use std::fs;

use scpi::{
    command::Todo, error::Result, ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc,
    ieee488_rst, ieee488_sre, ieee488_stb, ieee488_tst, ieee488_wai, prelude::*, qonly,
    scpi_status, scpi_system,
};

struct NumCommand;
impl Command<util::TestDevice> for NumCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut util::TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        // Parse a <numeric>
        let x: NumericValue<f32> = args.data()?;

        // Use builder to resolve special values
        let value = x
            // Specify required max,min values
            .with(100.0, -100.0)
            // Specify the default value
            .default(Default::default())
            // Finish the builder and resolve the final value,
            // Provided clousure is called when `AUTO` is used to determine the final value
            .finish_auto(|| 42.0)?;

        // Sanity check
        let option = match x {
            NumericValue::Value(_) => 0,
            NumericValue::Maximum => 1,
            NumericValue::Minimum => 2,
            NumericValue::Default => 3,
            NumericValue::Up => 4,
            NumericValue::Down => 5,
            NumericValue::Auto => 6,
        };
        response.data(value).data(option).finish()
    }
}

struct ChannelListCommand;

impl Command<util::TestDevice> for ChannelListCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut util::TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let numbers: channel_list::ChannelList = args.data()?;
        for item in numbers {
            match item? {
                channel_list::Token::ChannelSpec(t) => {
                    let ch: usize = t.try_into()?;
                    response.data(ch);
                }
                channel_list::Token::ChannelRange(a, b) => {
                    let a: usize = a.try_into()?;
                    let b: usize = b.try_into()?;
                    for i in a..=b {
                        response.data(i);
                    }
                }
                channel_list::Token::PathName(s) => {
                    response.data(s);
                }
                channel_list::Token::ModuleChannel(_, _) => todo!(),
            }
        }
        response.finish()
    }
}

struct NumericListCommand;

impl Command<util::TestDevice> for NumericListCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut util::TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let numbers: numeric_list::NumericList = args.data()?;
        for item in numbers {
            match item? {
                numeric_list::Token::Numeric(a) => {
                    response.data(f32::try_from(a)?);
                }
                numeric_list::Token::NumericRange(a, b) => {
                    response.data(f32::try_from(a)?);
                    response.data(f32::try_from(b)?);
                }
            }
        }
        response.finish()
    }
}

const IEEE488_TREE: Node<util::TestDevice> = Branch {
    name: b"",
    sub: &[
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
        scpi_status!(),
        scpi_system!(),
        Leaf {
            name: b"*NUM",
            default: false,
            handler: &NumCommand,
        },
        Leaf {
            name: b"*CHLIST",
            default: false,
            handler: &ChannelListCommand,
        },
        Leaf {
            name: b"*NUMLIST",
            default: false,
            handler: &NumericListCommand,
        },
        Branch {
            name: b"BRANch",
            sub: &[
                // Anonymous default leaf acts as handler for branch
                Leaf {
                    name: b"",
                    default: true,
                    handler: &Todo,
                },
                Leaf {
                    name: b"LEAF",
                    default: true,
                    handler: &Todo,
                },
            ],
        },
    ],
};

#[test]
fn quick_test() {
    let paths = fs::read_dir("tests/csv").unwrap();

    for entry in paths {
        let entry = entry.unwrap();
        match entry.path().extension() {
            Some(ext) if ext == "csv" => {
                println!("Checking: {}", entry.path().display());
                let mut dev = util::TestDevice::new();
                util::test_file(&mut dev, &IEEE488_TREE, entry.path())
            }
            _ => continue,
        }
    }
}

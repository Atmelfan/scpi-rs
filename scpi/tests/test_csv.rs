mod util;
use std::fs;

use scpi::{
    error::Result,
    parser::expression::{channel_list, numeric_list},
    tree::prelude::*,
};
// Commands
use scpi::cmd_qonly;

struct ChannelListCommand;

impl Command<util::TestDevice> for ChannelListCommand {
    cmd_qonly!();

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
    cmd_qonly!();

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

struct IdCommand(usize);

impl Command<util::TestDevice> for IdCommand {
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut util::TestDevice,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(self.0).finish()
    }
}

const IEEE488_TREE: Node<util::TestDevice> = Branch {
    name: b"",
    default: false,
    sub: &[
        Leaf {
            name: b"*COM",
            default: false,
            handler: &IdCommand(100),
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
            name: b"INITiate",
            default: false,
            sub: &[Branch {
                name: b"IMMediate",
                default: true,
                sub: &[Leaf {
                    name: b"ALL",
                    default: true,
                    handler: &IdCommand(0),
                }],
            }],
        },
        Branch {
            name: b"CONFigure",
            default: false,
            sub: &[
                // Anonymous default leaf acts as handler for branch
                Leaf {
                    name: b"",
                    default: true,
                    handler: &IdCommand(1),
                },
                Branch {
                    name: b"SCALar",
                    default: true,
                    sub: &[Branch {
                        name: b"VOLTage",
                        default: false,
                        sub: &[
                            Leaf {
                                name: b"AC",
                                default: false,
                                handler: &IdCommand(2),
                            },
                            Leaf {
                                name: b"DC",
                                default: true,
                                handler: &IdCommand(3),
                            },
                        ],
                    }],
                },
            ],
        },
        Branch {
            name: b"SYSTem",
            default: false,
            sub: &[
                Branch {
                    name: b"ERRor",
                    default: false,
                    sub: &[
                        Leaf {
                            name: b"ALL",
                            default: false,
                            handler: &IdCommand(11),
                        },
                        Leaf {
                            name: b"COUNt",
                            default: false,
                            handler: &IdCommand(12),
                        },
                        Leaf {
                            name: b"NEXT",
                            default: true,
                            handler: &IdCommand(13),
                        },
                    ],
                },
                Leaf {
                    name: b"VERSion",
                    default: false,
                    handler: &IdCommand(10),
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
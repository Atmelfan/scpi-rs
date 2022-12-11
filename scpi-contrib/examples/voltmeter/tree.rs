use scpi::tree::{command::Todo, prelude::*};

use scpi_contrib::scpi1999::trigger::{
    abort::AbortCommand, initiate::InitImmAllCommand, TrigSeqSourceCommand,
};
use scpi_contrib::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, scpi_status, scpi_system,
};

use crate::Voltmeter;

macro_rules! Leaf {
    ($name:literal => $handler:expr) => {
        Leaf {
            name: $name,
            default: false,
            handler: $handler,
        }
    };
    (default $name:literal => $handler:expr) => {
        Leaf {
            name: $name,
            default: true,
            handler: $handler,
        }
    };
}

macro_rules! Branch {
    ($name:literal; $($child:expr),+) => {
        Branch {
            name: $name,
            default: false,
            sub: &[
                $($child),+
            ],
        }
    };
    ($name:literal => $handler:expr; $($child:expr),+) => {
        Branch {
            name: $name,
            default: false,
            sub: &[
                Leaf!{ b"" => $handler },
                $($child),+
            ],
        }
    };
    (default $name:literal; $($child:expr),+) => {
        Branch {
            name: $name,
            default: true,
            sub: &[
                $($child),+
            ],
        }
    };
}

macro_rules! scpi_configure {
    ($($child:expr),+) => {
        Branch {
            name: b"CONFigure",
            default: false,
            sub: &[
                Leaf {
                    name: b"",
                    default: true,
                    handler: &Todo,
                },
                $($child),+
            ],
        }
    };
}

pub(crate) const TREE: Node<Voltmeter> = Branch! { b"";
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
    //
    Leaf!{ b"ABORt" => &AbortCommand },
    // # CONFigure
    scpi_configure! {
        Branch!{ default b"SCALar";
            Branch!{ b"VOLTage";
                Leaf!(b"AC" => &Todo),
                Leaf!(default b"DC" => &Todo)
            },
            Branch!{ b"CURRent";
                Leaf!(b"AC" => &Todo),
                Leaf!(default b"DC" => &Todo)
            },
            Leaf!(b"RESistance" => &Todo),
            Leaf!(b"FRESistance" => &Todo)
        }
    },
    Branch! {b"FETCh";
        Branch!{default b"SCALar";
            Branch![b"VOLTage";
                Leaf!(b"AC" => &Todo),
                Leaf!(default b"DC" => &Todo)
            ],
            Branch![b"CURRent";
                Leaf!(b"AC" => &Todo),
                Leaf!(default b"DC" => &Todo)
            ],
            Leaf!(b"RESistance" => &Todo),
            Leaf!(b"FRESistance" => &Todo)
        }
    },
    Branch!{b"INITiate";
        Branch!{default b"IMMediate";
            Leaf!(default b"ALL" => &InitImmAllCommand)
        }
    },
    Branch {
        name: b"MEASure",
        default: false,
        sub: &[Branch {
            name: b"SCALar",
            default: true,
            sub: &[
                // # :VOLTage([:DC]|:AC)
                Branch {
                    name: b"VOLTage",
                    default: false,
                    sub: &[
                        Leaf {
                            name: b"DC",
                            default: true,
                            handler: &Todo,
                        },
                        Leaf {
                            name: b"AC",
                            default: false,
                            handler: &Todo,
                        },
                    ],
                },
                Branch {
                    name: b"CURRent",
                    default: false,
                    sub: &[
                        Leaf {
                            name: b"DC",
                            default: true,
                            handler: &Todo,
                        },
                        Leaf {
                            name: b"AC",
                            default: false,
                            handler: &Todo,
                        },
                    ],
                },
            ],
        }],
    },
    Branch {
        name: b"READ",
        default: false,
        sub: &[Branch {
            name: b"SCALar",
            default: true,
            sub: &[
                // # :VOLTage([:DC]|:AC)
                Branch {
                    name: b"VOLTage",
                    default: false,
                    sub: &[
                        Leaf {
                            name: b"DC",
                            default: true,
                            handler: &Todo,
                        },
                        Leaf {
                            name: b"AC",
                            default: false,
                            handler: &Todo,
                        },
                    ],
                },
                Branch {
                    name: b"CURRent",
                    default: false,
                    sub: &[
                        Leaf {
                            name: b"DC",
                            default: true,
                            handler: &Todo,
                        },
                        Leaf {
                            name: b"AC",
                            default: false,
                            handler: &Todo,
                        },
                    ],
                },
            ],
        }],
    },
    Branch {
        name: b"ROUTe",
        default: false,
        sub: &[Leaf {
            name: b"TERMinals",
            default: false,
            handler: &Todo,
        }],
    },
    Branch {
        name: b"SENSe",
        default: false,
        sub: &[
            Branch {
                name: b"FUNCtion",
                default: false,
                sub: &[Leaf {
                    name: b"ON",
                    default: true,
                    handler: &InitImmAllCommand,
                }],
            },
            Branch {
                name: b"VOLTage",
                default: false,
                sub: &[
                    Branch {
                        name: b"AC",
                        default: false,
                        sub: &[
                            Branch {
                                name: b"RANGe",
                                default: false,
                                sub: &[
                                    Leaf {
                                        name: b"UPPer",
                                        default: true,
                                        handler: &Todo,
                                    },
                                    Leaf {
                                        name: b"AUTO",
                                        default: false,
                                        handler: &Todo,
                                    },
                                ],
                            },
                            Leaf {
                                name: b"RESolution",
                                default: false,
                                handler: &Todo,
                            },
                        ],
                    },
                    Branch {
                        name: b"DC",
                        default: true,
                        sub: &[
                            Branch {
                                name: b"RANGe",
                                default: false,
                                sub: &[
                                    Leaf {
                                        name: b"UPPer",
                                        default: true,
                                        handler: &Todo,
                                    },
                                    Leaf {
                                        name: b"AUTO",
                                        default: false,
                                        handler: &Todo,
                                    },
                                ],
                            },
                            Leaf {
                                name: b"RESolution",
                                default: false,
                                handler: &Todo,
                            },
                        ],
                    },
                ],
            },
            Branch {
                name: b"CURRent",
                default: false,
                sub: &[
                    Branch {
                        name: b"AC",
                        default: false,
                        sub: &[
                            Branch {
                                name: b"RANGe",
                                default: false,
                                sub: &[
                                    Leaf {
                                        name: b"UPPer",
                                        default: true,
                                        handler: &Todo,
                                    },
                                    Leaf {
                                        name: b"AUTO",
                                        default: false,
                                        handler: &Todo,
                                    },
                                ],
                            },
                            Leaf {
                                name: b"RESolution",
                                default: false,
                                handler: &Todo,
                            },
                        ],
                    },
                    Branch {
                        name: b"DC",
                        default: true,
                        sub: &[
                            Branch {
                                name: b"RANGe",
                                default: false,
                                sub: &[
                                    Leaf {
                                        name: b"UPPer",
                                        default: true,
                                        handler: &Todo,
                                    },
                                    Leaf {
                                        name: b"AUTO",
                                        default: false,
                                        handler: &Todo,
                                    },
                                ],
                            },
                            Leaf {
                                name: b"RESolution",
                                default: false,
                                handler: &Todo,
                            },
                        ],
                    },
                ],
            },
            Branch {
                name: b"RESistance",
                default: false,
                sub: &[
                    Branch {
                        name: b"RANGe",
                        default: false,
                        sub: &[
                            Leaf {
                                name: b"UPPer",
                                default: true,
                                handler: &Todo,
                            },
                            Leaf {
                                name: b"AUTO",
                                default: false,
                                handler: &Todo,
                            },
                        ],
                    },
                    Leaf {
                        name: b"RESolution",
                        default: false,
                        handler: &Todo,
                    },
                ],
            },
            Branch {
                name: b"FRESistance",
                default: false,
                sub: &[
                    Branch {
                        name: b"RANGe",
                        default: false,
                        sub: &[
                            Leaf {
                                name: b"UPPer",
                                default: true,
                                handler: &Todo,
                            },
                            Leaf {
                                name: b"AUTO",
                                default: false,
                                handler: &Todo,
                            },
                        ],
                    },
                    Leaf {
                        name: b"RESolution",
                        default: false,
                        handler: &Todo,
                    },
                ],
            },
        ],
    },
    Branch {
        name: b"TRIGger",
        default: false,
        sub: &[Branch {
            name: b"SEQuence",
            default: true,
            sub: &[
                Leaf {
                    name: b"COUNt",
                    default: false,
                    handler: &Todo,
                },
                Leaf {
                    name: b"DELay",
                    default: false,
                    handler: &Todo,
                },
                Leaf {
                    name: b"SOURce",
                    default: false,
                    handler: &TrigSeqSourceCommand,
                },
            ],
        }],
    }
};

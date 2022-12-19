use scpi::{
    tree::{command::Todo, prelude::*},
    Branch, Leaf, Root,
};

use scpi_contrib::{sense::function::SensFuncOnCommand, trigger::{TrigSeqCountCommand, TrigSeqDelayCommand}};
use scpi_contrib::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, scpi_status, scpi_system,
};
use scpi_contrib::{
    measurement::{ConfScalFuncCommand, FetchCommand, FetchScalFuncCommand},
    scpi1999::{
        measurement::ConfigureCommand,
        trigger::{abort::AbortCommand, initiate::InitImmAllCommand, TrigSeqSourceCommand},
    },
    trg::TrgCommand,
};

use crate::{device::measure::ScalVoltAc, Voltmeter};

pub(crate) const TREE: Node<Voltmeter> = Root!{
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
    // *TRG
    Leaf!(b"*TRG" => &TrgCommand),
    //
    Leaf!(b"ABORt" => &AbortCommand),
    // CONFigure
    Branch!{b"CONFigure" => &ConfigureCommand;
        Branch!{default b"SCALar";
            Branch!{ b"VOLTage";
                Leaf!(b"AC" => &ConfScalFuncCommand::<ScalVoltAc>::new()),
                Leaf!(default b"DC" => &ConfScalFuncCommand::<ScalVoltAc>::new())
            }
        }
    },
    // FETCh?
    Branch!{b"FETCh" => &FetchCommand;
        Branch!{default b"SCALar";
            Branch![b"VOLTage";
                // FETCh[:SCALar]:VOLTage:AC?
                Leaf!(b"AC" => &FetchScalFuncCommand::<ScalVoltAc>::new()),
                // FETCh[:SCALar]:VOLTage[:DC]?
                Leaf!(default b"DC" => &Todo)
            ]
        }
    },
    Branch!{b"INITiate";
        Branch!{default b"IMMediate";
            Leaf!(default b"ALL" => &InitImmAllCommand)
        }
    },
    Branch!{b"MEASure";
        Branch!{default b"SCALar";
            // # :VOLTage([:DC]|:AC)
            Branch!{b"VOLTage";
                Leaf!(b"AC" => &Todo),
                Leaf!(default b"DC" => &Todo)
            }
        }
    },
    Branch!{b"READ";
        Branch!{default b"SCALar";
            // # :VOLTage([:DC]|:AC)
            Branch!{ b"VOLTage";
                Leaf!(b"AC" => &Todo),
                Leaf!(default b"DC" => &Todo)
            }
        }
    },
    Branch!{ b"ROUTe";
        Leaf!(b"TERMinals" => &Todo)
    },
    Branch!{b"SENSe";
        Branch!{b"FUNCtion";
            Leaf!(default b"ON" => &SensFuncOnCommand)
        },
        Branch!{b"VOLTage";
            Branch!{b"AC";
                Branch!{b"RANGe";
                    Leaf!(b"UPPer" => &Todo),
                    Leaf!(b"AUTO" => &Todo)
                },
                Leaf!(b"RESolution" => &Todo)
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
            }
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
        }
    },
    Branch!{b"TRIGger";
        Branch!{default b"SEQuence";
            Leaf!(b"COUNt" => &TrigSeqCountCommand),
            Leaf!(b"DELay" => &TrigSeqDelayCommand),
            Leaf!(b"SOURce" => &TrigSeqSourceCommand)
        }
    }
};

use scpi::{
    tree::{command::Todo, prelude::*},
    Branch, Leaf, Root,
};

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
use scpi_contrib::{
    sense::{
        common::{SensRangAutoCommand, SensRangUpperCommand, SensResolutionCommand},
        function::SensFuncOnCommand,
    },
    trigger::{TrigSeqCountCommand, TrigSeqDelayCommand},
};

use crate::{device::ScalVoltAc, Voltmeter};

pub(crate) const TREE: Node<Voltmeter> = Root! {
    // Add mandatory IEEE488 commands
    ieee488_cls!(),
    ieee488_ese!(),
    ieee488_esr!(),
    ieee488_idn!(b"scpi-rs", b"digital_voltmeter", b"0", b"0"),
    ieee488_opc!(),
    ieee488_rst!(),
    ieee488_sre!(),
    ieee488_stb!(),
    ieee488_tst!(),
    ieee488_wai!(),
    // Add mandatory SCPI commands
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
                Leaf!(default b"DC" => &Todo)
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
                    Leaf!(default b"UPPer" => &SensRangUpperCommand::<ScalVoltAc>::new()),
                    Leaf!(b"AUTO" => &SensRangAutoCommand::<ScalVoltAc>::new())
                },
                Leaf!(b"RESolution" => &SensResolutionCommand::<ScalVoltAc>::new())
            },
            Branch!{default b"DC";
                Branch!{b"RANGe";
                    Leaf!(default b"UPPer" => &Todo),
                    Leaf!(b"AUTO" => &Todo)
                },
                Leaf!(b"RESolution" => &Todo)
            }
        }
        // TODO: Demo more functions
    },
    Branch!{b"TRIGger";
        Branch!{default b"SEQuence";
            Leaf!(b"COUNt" => &TrigSeqCountCommand),
            Leaf!(b"DELay" => &TrigSeqDelayCommand),
            Leaf!(b"SOURce" => &TrigSeqSourceCommand)
        }
    }
};

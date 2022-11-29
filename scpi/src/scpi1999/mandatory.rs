//! Mandated SCPI commands.

pub use super::status::*;
pub use super::system::*;

/// Create a `STATus:` tree branch
#[macro_export]
macro_rules! scpi_status {
    ($($node:expr),*) => {
        $crate::prelude::Branch {
            name: b"STATus",
            sub: &[
                $crate::prelude::Branch {
                    name: b"OPERation",
                    sub: &[
                        $crate::prelude::Leaf {
                            name: b"EVENt",
                            default: true,
                            handler: &StatOperEvenCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"CONDition",
                            default: false,
                            handler: &StatOperCondCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"ENABle",
                            default: false,
                            handler: &StatOperEnabCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"NTRansition",
                            default: false,
                            handler: &StatOperNtrCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"PTRansition",
                            default: false,
                            handler: &StatOperPtrCommand::new(),
                        },
                    ],
                },
                $crate::prelude::Branch {
                    name: b"QUEStionable",
                    sub: &[
                        $crate::prelude::Leaf {
                            name: b"EVENt",
                            default: true,
                            handler: &StatQuesEvenCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"CONDition",
                            default: false,
                            handler: &StatQuesCondCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"ENABle",
                            default: false,
                            handler: &StatQuesEnabCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"NTRansition",
                            default: false,
                            handler: &StatQuesNtrCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"PTRansition",
                            default: false,
                            handler: &StatQuesPtrCommand::new(),
                        },
                    ],
                },
                $crate::prelude::Leaf {
                    name: b"PRESet",
                    default: false,
                    handler: &StatPresCommand,
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

/// Create a `SYSTem:` tree branch
#[macro_export]
macro_rules! scpi_system {
    ($($node:expr),*) => {
        $crate::prelude::Branch {
            name: b"SYSTem",
            sub: &[
                $crate::prelude::Branch {
                    name: b"ERRor",
                    sub: &[
                        $crate::prelude::Leaf {
                            name: b"NEXT",
                            default: true,
                            handler: &SystErrNextCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"ALL",
                            default: false,
                            handler: &SystErrAllCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"COUNt",
                            default: false,
                            handler: &SystErrCounCommand,
                        },
                    ],
                },
                $crate::prelude::Leaf {
                    name: b"VERSion",
                    default: false,
                    handler: &SystVersCommand { year: 1999, rev: 0 }
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

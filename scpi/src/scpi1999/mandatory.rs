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
                            handler: &$crate::scpi1999::status::StatOperEvenCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"CONDition",
                            default: false,
                            handler: &$crate::scpi1999::status::StatOperCondCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"ENABle",
                            default: false,
                            handler: &$crate::scpi1999::status::StatOperEnabCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"NTRansition",
                            default: false,
                            handler: &$crate::scpi1999::status::StatOperNtrCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"PTRansition",
                            default: false,
                            handler: &$crate::scpi1999::status::StatOperPtrCommand::new(),
                        },
                    ],
                },
                $crate::prelude::Branch {
                    name: b"QUEStionable",
                    sub: &[
                        $crate::prelude::Leaf {
                            name: b"EVENt",
                            default: true,
                            handler: &$crate::scpi1999::status::StatQuesEvenCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"CONDition",
                            default: false,
                            handler: &$crate::scpi1999::status::StatQuesCondCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"ENABle",
                            default: false,
                            handler: &$crate::scpi1999::status::StatQuesEnabCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"NTRansition",
                            default: false,
                            handler: &$crate::scpi1999::status::StatQuesNtrCommand::new(),
                        },
                        $crate::prelude::Leaf {
                            name: b"PTRansition",
                            default: false,
                            handler: &$crate::scpi1999::status::StatQuesPtrCommand::new(),
                        },
                    ],
                },
                $crate::prelude::Leaf {
                    name: b"PRESet",
                    default: false,
                    handler: &$crate::scpi1999::status::StatPresCommand,
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
                            handler: &$crate::scpi1999::system::SystErrNextCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"ALL",
                            default: false,
                            handler: &$crate::scpi1999::system::SystErrAllCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"COUNt",
                            default: false,
                            handler: &$crate::scpi1999::system::SystErrCounCommand,
                        },
                    ],
                },
                $crate::prelude::Leaf {
                    name: b"VERSion",
                    default: false,
                    handler: &$crate::scpi1999::system::SystVersCommand { year: 1999, rev: 0 }
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

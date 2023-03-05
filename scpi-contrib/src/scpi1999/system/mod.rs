//! # SYSTem Subsystem
//! The SYSTem subsystem collects the functions that are not related to instrument
//! performance. Examples include functions for performing general housekeeping and
//! functions related to setting global configurations, such as TIME or SECurity

use scpi::{cmd_qonly, error::Result, tree::prelude::*};

use crate::ScpiDevice;

#[cfg(feature="unproven")]
// LFRequency requires frequency units
pub mod lfrequency;

#[cfg(feature="unproven")]
pub mod capability;

pub mod error;

///## 21.21 :VERSion?
///> `SYSTem:VERSion?` query returns an <NR2> formatted numeric value corresponding to the SCPI version
///> number for which the instrument complies. The response shall have the form YYYY.V where
///> the Ys represent the year-version (i.e. 1990) and the V represents an approved revision
///> number for that year. If no approved revisions are claimed, then this extension shall be 0.
pub struct SystVersionCommand {
    pub year: u16,
    pub rev: u8,
}

impl SystVersionCommand {
    pub const fn new(year: u16, rev: u8) -> Self {
        Self { year, rev }
    }
}

impl ResponseData for &SystVersionCommand {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        self.year.format_response_data(formatter)?;
        formatter.push_byte(b'.')?;
        self.rev.format_response_data(formatter)
    }
}

impl<D> Command<D> for SystVersionCommand
where
    D: ScpiDevice,
{
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(self).finish()
    }
}

/// Create a `SYSTem:` tree branch
#[macro_export]
macro_rules! scpi_system {
    ($($node:expr),*) => {
        scpi::tree::prelude::Branch {
            name: b"SYSTem",
            default: false,
            sub: &[
                scpi::tree::prelude::Branch {
                    name: b"ERRor",
                    default: false,
                    sub: &[
                        scpi::tree::prelude::Leaf {
                            name: b"NEXT",
                            default: true,
                            handler: &$crate::scpi1999::system::error::SystErrNextCommand,
                        },
                        scpi::tree::prelude::Leaf {
                            name: b"ALL",
                            default: false,
                            handler: &$crate::scpi1999::system::error::SystErrAllCommand,
                        },
                        scpi::tree::prelude::Leaf {
                            name: b"COUNt",
                            default: false,
                            handler: &$crate::scpi1999::system::error::SystErrCountCommand,
                        },
                    ],
                },
                scpi::tree::prelude::Leaf {
                    name: b"VERSion",
                    default: false,
                    handler: &$crate::scpi1999::system::SystVersionCommand { year: 1999, rev: 0 }
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

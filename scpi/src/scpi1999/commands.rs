//! Contains basic implementations of SCPI mandated and optional commands.
//!
//!
//!
use crate::error::Result;
use crate::prelude::*;
use crate::{nquery, qonly};

use core::convert::TryInto;

///## 21.8.8 \[NEXT\]?
///> `SYSTem:ERRor:NEXT?` queries the error/event queue for the next item and removes it
///> from the queue. The response returns the full queue item consisting of an integer and a string
///> as described in the introduction to the SYSTem:ERRor subsystem.
pub struct SystErrNextCommand;

impl Command for SystErrNextCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response.data(context.errors.pop_front_error()).finish()
    }
}

///## 21.8.6 COUNt?
///> `SYSTem:ERRor:COUNt?` queries the error/event queue for the number of unread items. As
///> errors and events may occur at any time, more items may be present in the queue at the time
///> it is actually read.
///>
///> Note: If the queue is empty, the response is 0.
pub struct SystErrCounCommand;

impl Command for SystErrCounCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response.data(context.errors.len()).finish()
    }
}

///## 21.8.5.1 ALL?
///> `SYSTem:ERRor:ALL?` queries the error/event queue for all the unread items and
///> removes them from the queue. The response returns a comma separated list of only the
///> error/event code numbers in FIFO order. If the queue is empty, the response is 0.
pub struct SystErrAllCommand;

impl Command for SystErrAllCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        if context.errors.is_empty() {
            response.data(Error::new(ErrorCode::NoError)).finish()
        } else {
            loop {
                let err = context.errors.pop_front_error();
                if err == ErrorCode::NoError {
                    break;
                }
                response.data(err);
            }
            response.finish()
        }
    }
}

///## 21.21 :VERSion?
///> `SYSTem:VERSion?` query returns an <NR2> formatted numeric value corresponding to the SCPI version
///> number for which the instrument complies. The response shall have the form YYYY.V where
///> the Ys represent the year-version (i.e. 1990) and the V represents an approved revision
///> number for that year. If no approved revisions are claimed, then this extension shall be 0.
pub struct SystVersCommand {
    pub year: u16,
    pub rev: u8,
}

impl Data for &SystVersCommand {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        self.year.format_response_data(formatter)?;
        formatter.push_byte(b'.')?;
        self.rev.format_response_data(formatter)
    }
}

impl Command for SystVersCommand {
    qonly!();

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response.data(self).finish()
    }
}

///## 20.1.4 \[:EVENt\]?
///> `STATus:OPERation:EVENt?`
///> This query returns the contents of the event register associated with the status structure
///> defined in the command.
///> The response is (NR1 NUMERIC RESPONSE DATA) (range: 0 through 32767) unless
///> changed by the :FORMat:SREGister command.
///>
///> Note that reading the event register clears it.
pub struct StatOperEvenCommand;

impl Command for StatOperEvenCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(core::mem::replace(&mut context.operation.event, 0) & 0x7FFFu16)
            .finish()
    }
}

///## 20.1.2 :CONDition?
///> `STATus:OPERation:CONDition?`
///> Returns the contents of the condition register associated with the status structure defined in
///> the command. Reading the condition register is nondestructive.
pub struct StatOperCondCommand;

impl Command for StatOperCondCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(context.operation.condition & 0x7FFFu16)
            .finish()
    }
}

///## 20.1.3 :ENABle \<NRf\> | \<non-decimal numeric\>
///> `STATus:OPERation:ENABle`
///> Sets the enable mask which allows true conditions in the event register to be reported in the
///> summary bit. If a bit is 1 in the enable register and its associated event bit transitions to true,
///> a positive transition will occur in the associated summary bit.
///> The command accepts parameter values of either format in the range 0 through 65535
///> (decimal) without error.
///>
///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
///> Note that 32767 is the maximum value returned as the most-significant bit of the register
///> cannot be set true.
pub struct StatOperEnabCommand;

impl Command for StatOperEnabCommand {
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        context.operation.enable = args.next_data(false)?.unwrap().try_into()?;
        Ok(())
    }

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response.data(context.operation.enable & 0x7FFFu16).finish()
    }
}

///# 20.1.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:OPERation:NTRansition`
///> Sets the negative transition filter. Setting a bit in the negative transition filter shall cause a 1
///> to 0 transition in the corresponding bit of the associated condition register to cause a 1 to be
///> written in the associated bit of the corresponding event register.
///> The command accepts parameter values of either format in the range 0 through 65535
///> (decimal) without error.
///>
///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
///> Note that 32767 is the maximum value returned as the most-significant bit of the register
///> cannot be set true.
pub struct StatOperNtrCommand;

impl Command for StatOperNtrCommand {
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        context.operation.ntr_filter = args.next_data(false)?.unwrap().try_into()?;
        Ok(())
    }

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(context.operation.ntr_filter & 0x7FFFu16)
            .finish()
    }
}

///# 20.1.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
///> STATus:OPERation:PTRansition
///> Sets the positive transition filter. Setting a bit in the positive transition filter shall cause a 0 to
///> transition in the corresponding bit of the associated condition register to cause a 1 to be
///> written in the associated bit of the corresponding event register.
///> The command accepts parameter values of either format in the range 0 through 65535
///> (decimal) without error.
///>
///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
///> Note that 32767 is the maximum value returned as the most-significant bit of the register
///> cannot be set true.
pub struct StatOperPtrCommand;

impl Command for StatOperPtrCommand {
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        context.operation.ptr_filter = args.next_data(false)?.unwrap().try_into()?;
        Ok(())
    }

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(context.operation.ptr_filter & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.4 \[:EVENt\]?
///> `STATus:QUEStionable:EVENt?`
///> Defined the same as STATus:OPERation:EVENt. See Section 20.1.4 for details.
pub struct StatQuesEvenCommand;

impl Command for StatQuesEvenCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(core::mem::replace(&mut context.questionable.event, 0) & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.2 :CONDition?
///> `STATus:QUEStionable:CONDition?`
///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
pub struct StatQuesCondCommand;

impl Command for StatQuesCondCommand {
    qonly!();

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response
            .data(context.questionable.condition & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.3 :ENABle \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:ENABle`
///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
pub struct StatQuesEnabCommand;

impl Command for StatQuesEnabCommand {
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        context.questionable.enable = args.next_data(false)?.unwrap().try_into()?;
        Ok(())
    }

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(context.questionable.enable & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:NTRansition`
///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
pub struct StatQuesNtrCommand;

impl Command for StatQuesNtrCommand {
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        context.questionable.ntr_filter = args.next_data(false)?.unwrap().try_into()?;
        Ok(())
    }

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(context.questionable.ntr_filter & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:PTRansition`
///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
pub struct StatQuesPtrCommand;

impl Command for StatQuesPtrCommand {
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        context.questionable.ptr_filter = args.next_data(false)?.unwrap().try_into()?;
        Ok(())
    }

    fn query(
        &self,
        context: &mut Context,
        _args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        response
            .data(context.questionable.ptr_filter & 0x7FFFu16)
            .finish()
    }
}

///## 20.2 :PRESet
///> `STATus:PRESet`
///> The PRESet command is an event that configures the SCPI and device-dependent status data
///> structures such that device-dependent events are reported at a higher level through the
///> mandatory part of the status-reporting mechanism. Device-dependent events are summarized
///> in the mandatory structures. The mandatory structure is defined in part by IEEE 488.2;
///> SCPI-required structures compose the rest. The mandatory part of the status-reporting
///> mechanism provides a device-independent interface for determining the gross status of a
///> device.
pub struct StatPresCommand;

impl Command for StatPresCommand {
    nquery!();
    fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        context.questionable.preset();
        context.operation.preset();
        context.device.preset()
    }
}

/// Create a `STATus:` tree branch
#[macro_export]
macro_rules! scpi_status {
    ($($node:expr),*) => {
        Node {
            name: b"STATus",
            optional: false,
            handler: None,
            sub: &[
                Node {
                    name: b"OPERation",
                    optional: false,
                    handler: None,
                    sub: &[
                        Node {
                            name: b"CONDition",
                            optional: false,
                            handler: Some(&StatOperCondCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"ENABle",
                            optional: false,
                            handler: Some(&StatOperEnabCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"EVENt",
                            optional: true,
                            handler: Some(&StatOperEvenCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"NTRansition",
                            optional: false,
                            handler: Some(&StatOperNtrCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"PTRansition",
                            optional: false,
                            handler: Some(&StatOperPtrCommand {}),
                            sub: &[],
                        },
                    ],
                },
                Node {
                    name: b"QUEStionable",
                    optional: false,
                    handler: None,
                    sub: &[
                        Node {
                            name: b"CONDition",
                            optional: false,
                            handler: Some(&StatQuesCondCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"ENABle",
                            optional: false,
                            handler: Some(&StatQuesEnabCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"EVENt",
                            optional: true,
                            handler: Some(&StatQuesEvenCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"NTRansition",
                            optional: false,
                            handler: Some(&StatQuesNtrCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"PTRansition",
                            optional: false,
                            handler: Some(&StatQuesPtrCommand {}),
                            sub: &[],
                        },
                    ],
                },
                Node {
                    name: b"PRESet",
                    optional: false,
                    handler: Some(&StatPresCommand {}),
                    sub: &[],
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
        Node {
            name: b"SYSTem",
            optional: false,
            handler: None,
            sub: &[
                Node {
                    name: b"ERRor",
                    optional: false,
                    handler: None,
                    sub: &[
                        Node {
                            name: b"ALL",
                            optional: false,
                            handler: Some(&SystErrAllCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"NEXT",
                            optional: true,
                            handler: Some(&SystErrNextCommand {}),
                            sub: &[],
                        },
                        Node {
                            name: b"COUNt",
                            optional: false,
                            handler: Some(&SystErrCounCommand {}),
                            sub: &[],
                        },
                    ],
                },
                Node {
                    name: b"VERSion",
                    optional: false,
                    handler: Some(&SystVersCommand { year: 1999, rev: 0 }),
                    sub: &[],
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

//! Contains basic implementations of SCPI mandated and optional commands.
//!
//!
//!
use crate::error::Result;
use crate::prelude::*;
use crate::tokenizer::Arguments;
use crate::{nquery, qonly};

use core::convert::TryInto;

use super::ScpiDevice;

///## 21.8.8 \[NEXT\]?
///> `SYSTem:ERRor:NEXT?` queries the error/event queue for the next item and removes it
///> from the queue. The response returns the full queue item consisting of an integer and a string
///> as described in the introduction to the SYSTem:ERRor subsystem.
pub struct SystErrNextCommand;

impl<D> Command<D> for SystErrNextCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response.data(device.pop_front_error()).finish()
    }
}

///## 21.8.6 COUNt?
///> `SYSTem:ERRor:COUNt?` queries the error/event queue for the number of unread items. As
///> errors and events may occur at any time, more items may be present in the queue at the time
///> it is actually read.
///>
///> Note: If the queue is empty, the response is 0.
pub struct SystErrCounCommand;

impl<D> Command<D> for SystErrCounCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response.data(device.num_errors()).finish()
    }
}

///## 21.8.5.1 ALL?
///> `SYSTem:ERRor:ALL?` queries the error/event queue for all the unread items and
///> removes them from the queue. The response returns a comma separated list of only the
///> error/event code numbers in FIFO order. If the queue is empty, the response is 0.
pub struct SystErrAllCommand;

impl<D> Command<D> for SystErrAllCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        if device.is_empty() {
            response.data(Error::new(ErrorCode::NoError)).finish()
        } else {
            loop {
                let err = device.pop_front_error();
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

impl<D> Command<D> for SystVersCommand
where
    D: ScpiDevice,
{
    qonly!();

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

///## 20.1.4 \[:EVENt\]?
///> `STATus:OPERation:EVENt?`
///> This query returns the contents of the event register associated with the status structure
///> defined in the command.
///> The response is (NR1 NUMERIC RESPONSE DATA) (range: 0 through 32767) unless
///> changed by the :FORMat:SREGister command.
///>
///> Note that reading the event register clears it.
pub struct StatOperEvenCommand;

impl<D> Command<D> for StatOperEvenCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(core::mem::replace(&mut device.operation_mut().event, 0) & 0x7FFFu16)
            .finish()
    }
}

///## 20.1.2 :CONDition?
///> `STATus:OPERation:CONDition?`
///> Returns the contents of the condition register associated with the status structure defined in
///> the command. Reading the condition register is nondestructive.
pub struct StatOperCondCommand;

impl<D> Command<D> for StatOperCondCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.operation().condition & 0x7FFFu16)
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

impl<D> Command<D> for StatOperEnabCommand
where
    D: ScpiDevice,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.operation_mut().enable = args.next()?.try_into()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.operation().enable & 0x7FFFu16).finish()
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

impl<D> Command<D> for StatOperNtrCommand
where
    D: ScpiDevice,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.operation_mut().ntr_filter = args.next()?.try_into()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.operation().ntr_filter & 0x7FFFu16)
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

impl<D> Command<D> for StatOperPtrCommand
where
    D: ScpiDevice,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.operation_mut().ptr_filter = args.next()?.try_into()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.operation().ptr_filter & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.4 \[:EVENt\]?
///> `STATus:QUEStionable:EVENt?`
///> Defined the same as STATus:OPERation:EVENt. See Section 20.1.4 for details.
pub struct StatQuesEvenCommand;

impl<D> Command<D> for StatQuesEvenCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {

        response
            .data(core::mem::replace(&mut device.questionable_mut().event, 0) & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.2 :CONDition?
///> `STATus:QUEStionable:CONDition?`
///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
pub struct StatQuesCondCommand;

impl<D> Command<D> for StatQuesCondCommand
where
    D: ScpiDevice,
{
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response
            .data(device.questionable().condition & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.3 :ENABle \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:ENABle`
///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
pub struct StatQuesEnabCommand;

impl<D> Command<D> for StatQuesEnabCommand
where
    D: ScpiDevice,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.questionable_mut().enable = args.next()?.try_into()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.questionable().enable & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:NTRansition`
///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
pub struct StatQuesNtrCommand;

impl<D> Command<D> for StatQuesNtrCommand
where
    D: ScpiDevice,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.questionable_mut().ntr_filter = args.next()?.try_into()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.questionable().ntr_filter & 0x7FFFu16)
            .finish()
    }
}

///# 20.3.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:PTRansition`
///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
pub struct StatQuesPtrCommand;

impl<D> Command<D> for StatQuesPtrCommand
where
    D: ScpiDevice,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.questionable_mut().ptr_filter = args.next()?.try_into()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.questionable().ptr_filter & 0x7FFFu16)
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

impl<D> Command<D> for StatPresCommand
where
    D: ScpiDevice,
{
    nquery!();
    fn event(&self, device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
        device.questionable_mut().preset();
        device.operation_mut().preset();
        device.exec_preset()
    }
}

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
                            handler: &StatOperEvenCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"CONDition",
                            default: false,
                            handler: &StatOperCondCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"ENABle",
                            default: false,
                            handler: &StatOperEnabCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"NTRansition",
                            default: false,
                            handler: &StatOperNtrCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"PTRansition",
                            default: false,
                            handler: &StatOperPtrCommand,
                        },
                    ],
                },
                $crate::prelude::Branch {
                    name: b"QUEStionable",
                    sub: &[
                        $crate::prelude::Leaf {
                            name: b"EVENt",
                            default: true,
                            handler: &StatQuesEvenCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"CONDition",
                            default: false,
                            handler: &StatQuesCondCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"ENABle",
                            default: false,
                            handler: &StatQuesEnabCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"NTRansition",
                            default: false,
                            handler: &StatQuesNtrCommand,
                        },
                        $crate::prelude::Leaf {
                            name: b"PTRansition",
                            default: false,
                            handler: &StatQuesPtrCommand,
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

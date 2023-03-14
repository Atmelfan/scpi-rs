//! # STATus Subsystem
//! This subsystem controls the SCPI-defined status-reporting structures. SCPI defines, in
//! addition to those in IEEE 488.2, QUEStionable, OPERation, Instrument SUMmary and
//! INSTrument registers.
//!
//! These registers conform to the IEEE 488.2 specification and each
//! may be comprised of a condition register, an event register, an enable register, and negative
//! and positive transition filters. The purpose and definition of the SCPI-defined registers is
//! described in “Volume 1: Syntax and Style”.
//!
//! SCPI also defines an IEEE 488.2 queue for status. The queue provides a human readable
//! record of instrument events. The application programmer may individually enable events
//! into the queue. STATus:PRESet enables errors and disables all other events. If the summary
//! of the queue is reported, it shall be reported in bit 2 of the status byte register. A subset of
//! error/event numbers is defined by SCPI. Additional error/event numbers will be defined at a
//! later date.
//!
use scpi::{cmd_both, cmd_nquery, cmd_qonly, error::Result, tree::prelude::*};

use core::marker::PhantomData;

use super::{BitFlags, EventRegisterName, GetEventRegister, ScpiDevice};

pub mod operation;
pub mod questionable;

///## 20.2 :PRESet
///> `STATus:PRESet`
///> The PRESet command is an event that configures the SCPI and device-dependent status data
///> structures such that device-dependent events are reported at a higher level through the
///> mandatory part of the status-reporting mechanism. Device-dependent events are summarized
///> in the mandatory structures. The mandatory structure is defined in part by IEEE 488.2;
///> SCPI-required structures compose the rest. The mandatory part of the status-reporting
///> mechanism provides a device-independent interface for determining the gross status of a
///> device.
pub struct StatPresetCommand;

impl<D> Command<D> for StatPresetCommand
where
    D: ScpiDevice,
{
    cmd_nquery!();
    fn event(&self, device: &mut D, _context: &mut Context, _params: Parameters) -> Result<()> {
        device.preset()
    }
}

///> `EVENt?`
///> Defined the same as STATus:OPERation:EVENt. See Section 20.1.4 for details.
pub struct EventCommand<T>(PhantomData<T>);

impl<T> EventCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for EventCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    cmd_qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(core::mem::replace(&mut device.register_mut().event, 0) & 0x7FFFu16)
            .finish()
    }
}

///> `CONDition?`
///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
pub struct ConditionCommand<T>(PhantomData<T>);

impl<T> ConditionCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for ConditionCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    cmd_qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response
            .data(device.register().condition & 0x7FFFu16)
            .finish()
    }
}

///> `ENABle`
///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
pub struct EnableCommand<T>(PhantomData<T>);

impl<T> EnableCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for EnableCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    cmd_both!();

    fn event(&self, device: &mut D, _context: &mut Context, mut params: Parameters) -> Result<()> {
        device.register_mut().enable = params.next_data()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.register().enable & 0x7FFFu16).finish()
    }
}

///> `NTRansition`
///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
pub struct NTransitionCommand<T>(PhantomData<T>);

impl<T> NTransitionCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for NTransitionCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    cmd_both!();

    fn event(&self, device: &mut D, _context: &mut Context, mut params: Parameters) -> Result<()> {
        device.register_mut().ntr_filter = params.next_data()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.register().ntr_filter & 0x7FFFu16)
            .finish()
    }
}

///> `PTRansition`
///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
pub struct PTransitionCommand<T>(PhantomData<T>);

impl<T> PTransitionCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for PTransitionCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    cmd_both!();

    fn event(&self, device: &mut D, _context: &mut Context, mut params: Parameters) -> Result<()> {
        device.register_mut().ptr_filter = params.next_data()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(device.register().ptr_filter & 0x7FFFu16)
            .finish()
    }
}

/// Create command nodes for a SCPI registers like `OPERation`, `QUEStionable`, or custom event registers.
#[macro_export]
macro_rules! scpi_register {
    ($name:literal, $register:path) => {
        $crate::scpi_register!($name, $register; )
    };
    ($name:literal, $register:path; $($node:expr),*) => {
        scpi::tree::prelude::Branch {
            name: $name,
            default: false,
            sub: &[
                scpi::tree::prelude::Leaf {
                    name: b"EVENt",
                    default: true,
                    handler: &$crate::scpi1999::status::EventCommand::<$register>::new(),
                },
                scpi::tree::prelude::Leaf {
                    name: b"CONDition",
                    default: false,
                    handler: &$crate::scpi1999::status::ConditionCommand::<$register>::new(),
                },
                scpi::tree::prelude::Leaf {
                    name: b"ENABle",
                    default: false,
                    handler: &$crate::scpi1999::status::EnableCommand::<$register>::new(),
                },
                scpi::tree::prelude::Leaf {
                    name: b"NTRansition",
                    default: false,
                    handler: &$crate::scpi1999::status::NTransitionCommand::<$register>::new(),
                },
                scpi::tree::prelude::Leaf {
                    name: b"PTRansition",
                    default: false,
                    handler: &$crate::scpi1999::status::PTransitionCommand::<$register>::new(),
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

/// Create a `STATus:` tree branch
#[macro_export]
macro_rules! scpi_status {
    ($($node:expr),*) => {
        scpi::tree::prelude::Branch {
            name: b"STATus",
            default: false,
            sub: &[
                $crate::scpi_register!(b"OPERation", $crate::scpi1999::status::operation::Operation),
                $crate::scpi_register!(b"QUEStionable", $crate::scpi1999::status::questionable::Questionable),
                scpi::tree::prelude::Leaf {
                    name: b"PRESet",
                    default: false,
                    handler: &$crate::scpi1999::status::StatPresetCommand,
                },
                $(
                    $node
                ),*
            ],
        }
    };
}

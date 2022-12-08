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
use crate::error::Result;
use crate::prelude::*;
use crate::{nquery, qonly};

use core::marker::PhantomData;

use super::{BitFlags, EventRegisterName, GetEventRegister, ScpiDevice};

pub struct Operation;
impl EventRegisterName for Operation {
    type BitFlags = OperationBits;
}

/// The OPERation status register contains conditions which are part of the instrument’s normal
/// operation.
pub enum OperationBits {
    /// The instrument is currently performing a calibration.
    Calibrating = 0,
    /// The instrument is waiting for signals it controls to stabilize
    /// enough to begin measurements.
    Settling = 1,
    /// The instrument is currently changing its range.
    Ranging = 2,
    /// A sweep is in progress.
    Sweeping = 3,
    /// The instrument is actively measuring.
    Measuring = 4,
    /// The instrument is in a “wait for trigger” state of the
    /// trigger model.
    WaitingForTrig = 5,
    /// The instrument is in a “wait for arm” state of the trigger
    /// model.
    WaitingForArm = 6,
    /// The instrument is currently performing a correction.
    Correcting = 7,
    /// Available to designer.
    Designer1 = 8,
    /// Available to designer.
    Designer2 = 9,
    /// Available to designer.
    Designer3 = 10,
    /// Available to designer.
    Designer4 = 11,
    /// Available to designer.
    Designer5 = 12,
    /// One of n multiple logical instruments is
    /// reporting OPERational status.
    InstrumentSummary = 13,
    /// A user-defined programming is currently in the run
    /// state.
    ProgramRunning = 14,
}

impl BitFlags<u16> for OperationBits {
    fn get_mask(self) -> u16 {
        1 << (self as u16)
    }

    fn get_pos(self) -> u16 {
        self as u16
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
pub type StatOperEvenCommand = EventCommand<Operation>;

///## 20.1.2 :CONDition?
///> `STATus:OPERation:CONDition?`
///> Returns the contents of the condition register associated with the status structure defined in
///> the command. Reading the condition register is nondestructive.
pub type StatOperCondCommand = CondCommand<Operation>;

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
pub type StatOperEnabCommand = EnabCommand<Operation>;

pub struct Questionable;
impl EventRegisterName for Questionable {
    type BitFlags = QuestionableBits;
}

/// The QUEStionable status register set contains bits which give an indication of the quality of
/// various aspects of the signal.
pub enum QuestionableBits {
    /// Indicates that the data is currently being acquired or generated
    SummaryVoltage = 0,
    SummaryCurrent = 1,
    SummaryTime = 2,
    SummaryPower = 3,
    SummaryTemperature = 4,
    SummaryFrequency = 5,
    SummaryPhase = 6,
    SummaryModulation = 7,
    SummaryCalibration = 8,
    Designer1 = 9,
    Designer2 = 10,
    Designer3 = 11,
    Designer4 = 12,
    InstrumentSummary = 13,
    /// Bit 14 is defined as the Command Warning bit. This bit indicates a non-fatal warning that
    /// relates to the instrument’s interpretation of a command, query, or one or more parameters of
    /// a specific command or query. Setting this bit is a warning to the application that the resultant
    /// instrument state or action is probably what was expected but may deviate in some manner.
    ///
    /// For example, the Command Warning bit is set whenever a parameter in one of the
    /// Measurement Instruction commands or queries is ignored during execution. Such a
    /// parameter may be ignored because it cannot be specified by a particular instrument.
    CommandWarning = 14,
}

impl BitFlags<u16> for QuestionableBits {
    fn get_mask(self) -> u16 {
        1 << (self as u16)
    }

    fn get_pos(self) -> u16 {
        self as u16
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
pub type StatOperNtrCommand = NtrCommand<Operation>;

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
pub type StatOperPtrCommand = PtrCommand<Operation>;

///# 20.3.4 \[:EVENt\]?
///> `STATus:QUEStionable:EVENt?`
///> Defined the same as STATus:OPERation:EVENt. See Section 20.1.4 for details.
pub type StatQuesEvenCommand = EventCommand<Questionable>;

///# 20.3.2 :CONDition?
///> `STATus:QUEStionable:CONDition?`
///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
pub type StatQuesCondCommand = CondCommand<Questionable>;

///# 20.3.3 :ENABle \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:ENABle`
///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
pub type StatQuesEnabCommand = EnabCommand<Questionable>;

///# 20.3.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:NTRansition`
///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
pub type StatQuesNtrCommand = NtrCommand<Questionable>;

///# 20.3.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:PTRansition`
///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
pub type StatQuesPtrCommand = PtrCommand<Questionable>;

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
        device.exec_preset()
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
    qonly!();

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response
            .data(core::mem::replace(&mut device.register_mut().event, 0) & 0x7FFFu16)
            .finish()
    }
}

///> `CONDition?`
///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
pub struct CondCommand<T>(PhantomData<T>);

impl<T> CondCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for CondCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
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
            .data(device.register().condition & 0x7FFFu16)
            .finish()
    }
}

///> `ENABle`
///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
pub struct EnabCommand<T>(PhantomData<T>);

impl<T> EnabCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for EnabCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.register_mut().enable = args.next()?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.register().enable & 0x7FFFu16).finish()
    }
}

///> `NTRansition`
///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
pub struct NtrCommand<T>(PhantomData<T>);

impl<T> NtrCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for NtrCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.register_mut().ntr_filter = args.next()?;
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
            .data(device.register().ntr_filter & 0x7FFFu16)
            .finish()
    }
}

///> `PTRansition`
///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
pub struct PtrCommand<T>(PhantomData<T>);

impl<T> PtrCommand<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<D, T> Command<D> for PtrCommand<T>
where
    T: EventRegisterName,
    D: Device + GetEventRegister<T>,
{
    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        device.register_mut().ptr_filter = args.next()?;
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
            .data(device.register().ptr_filter & 0x7FFFu16)
            .finish()
    }
}

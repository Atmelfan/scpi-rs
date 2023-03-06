//! # TRIGger Subsystem
//! The trigger subsystem is used to synchronize device action(s) with events. A device action
//! might be the acquisition of a measurement or the application of a stimulus. The trigger
//! subsystem consists of the expanded capability model which is capable of describing very
//! complex device trigger systems. It also makes provision, through the ARM-TRIGger model,
//! for simple descriptions of less complicated trigger systems. These two models are consistent
//! and compatible with each other. The ARM-TRIGger model represents a subset of the
//! capability available in the expanded capability model.
//!
//! The figures in this section that represent the trigger model adhere to the following
//! nomenclature. A box identifies a state of a transition diagram and is referred to as a layer. A
//! sequence is a set of vertically connected layers. A solid line defines flow of control between
//! states. A dashed line defines how an event is propagated to other parts of the trigger model
//! and the instrument. Events generated by control flowing from one part of the trigger model
//! to another part are called “sequence events.”
use scpi::{cmd_both, cmd_nquery, error::Result, option::ScpiEnum, tree::prelude::*, units};

use self::{abort::Abort, initiate::Initiate};

use super::{numeric::NumericValue, prelude::*};

pub mod abort;
pub mod initiate;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum TriggerState {
    #[default]
    Idle,
    Initated,
    WaitingForArm,
    WaitingForTrigger,
    Triggered,
}

/// A subset of trigger sources specified in  
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, scpi_derive::ScpiEnum)]
#[non_exhaustive]
pub enum TriggerSource {
    /// `BUS` — The source is signal specific to the control interface. For IEEE 488.1 the
    /// group execute trigger, GET, would satisfy this condition. In VXI the word serial
    /// command TRIGger performs this function. The event detector is also satisfied,
    /// independent of the interface, when a *TRG command is received. Note that
    /// neither GET nor *TRG can be sent without having an effect on the message
    /// exchange protocol described in IEEE 488.2.
    #[scpi(mnemonic = b"BUS")]
    Bus,
    /// `EXTernal` — An external signal jack is selected as the source. If no suffix is
    /// specified, EXTernal1 is assumed.
    #[scpi(mnemonic = b"EXTernal")]
    External,
    /// `IMMediate` — No waiting for an event occurs.
    #[default]
    #[scpi(mnemonic = b"IMMediate")]
    Immediate,
    /// `INTernal` — An internal channel is selected as the source. An INTernal event is
    /// derived from a measurement function and/or sensor capability in the signal
    /// conditioning block. If no suffix is specified, INTernal1 is assumed
    #[scpi(mnemonic = b"INTernal")]
    Internal,
    /// `OUTPut` — The signal source is taken from an output channel. If no channel is
    /// specified, OUTPut1 is assumed.
    #[scpi(mnemonic = b"OUTPut")]
    Output,
}

///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, scpi_derive::ScpiEnum)]
#[non_exhaustive]
pub enum TriggerSlope {
    /// `POSitive` — Risng edge
    #[default]
    #[scpi(mnemonic = b"POSitive")]
    Positive,
    /// `NEGative` — Falling edge.
    #[scpi(mnemonic = b"NEGative")]
    Negative,
    /// `EITHer` — Either falling or positive edge.
    #[scpi(mnemonic = b"EITHer")]
    Either,
}

///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, scpi_derive::ScpiEnum)]
#[non_exhaustive]
pub enum TriggerCoupling {
    /// `AC` — AC coupled trigger
    #[scpi(mnemonic = b"AC")]
    Ac,
    /// `DC` — DC coupled trigger.
    #[default]
    #[scpi(mnemonic = b"DC")]
    Dc,
}

pub trait Trigger<const SEQ: usize = 1>: Abort + Initiate {
    type Source: ScpiEnum + for<'a> TryFrom<Token<'a>, Error = Error>;

    fn count(&mut self, count: NumericValue<usize>) -> Result<()>;
    fn get_count(&self) -> usize;

    fn delay(&mut self, delay: NumericValue<units::Time>) -> Result<()>;
    fn get_delay(&self) -> units::Time;

    /// Set source of trigger
    fn source(&mut self, source: Self::Source) -> Result<()>;
    fn get_source(&self) -> Self::Source;
}

pub struct TrigSeqSourceCommand<const SEQ: usize = 1>;

impl<D, const SEQ: usize> Command<D> for TrigSeqSourceCommand<SEQ>
where
    D: ScpiDevice + Trigger<SEQ>,
{
    cmd_both!();

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let src: D::Source = args.data()?;
        device.source(src)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.get_source()).finish()
    }
}

pub struct TrigSeqDelayCommand<const SEQ: usize = 1>;

impl<D, const SEQ: usize> Command<D> for TrigSeqDelayCommand<SEQ>
where
    D: ScpiDevice + Trigger<SEQ>,
{
    cmd_both!();

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let delay: NumericValue<units::Time> = args.data()?;
        device.delay(delay)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.get_delay()).finish()
    }
}

pub struct TrigSeqCountCommand<const SEQ: usize = 1>;

impl<D, const SEQ: usize> Command<D> for TrigSeqCountCommand<SEQ>
where
    D: ScpiDevice + Trigger<SEQ>,
{
    cmd_both!();

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let delay: NumericValue<usize> = args.data()?;
        device.count(delay)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.get_count()).finish()
    }
}
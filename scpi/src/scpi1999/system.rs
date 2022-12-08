//! # SYSTem Subsystem
//! The SYSTem subsystem collects the functions that are not related to instrument
//! performance. Examples include functions for performing general housekeeping and
//! functions related to setting global configurations, such as TIME or SECurity

use crate::error::Result;
use crate::prelude::*;
use crate::qonly;

use super::util::Auto;
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
        response
            .data(device.pop_front_error().unwrap_or_default())
            .finish()
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
            while let Some(err) = device.pop_front_error() {
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

impl ResponseData for &SystVersCommand {
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

#[cfg(feature = "unit-frequency")]
pub trait LineFrequency {
    fn line_frequency(&mut self, freq: NumericValue<uom::si::f32::Frequency>) -> Result<()>;
    fn get_line_frequency(&self) -> uom::si::f32::Frequency;

    fn auto(&mut self, auto: Auto) -> Result<()>;
    fn get_auto(&self) -> Auto;
}
#[cfg(feature = "unit-frequency")]
pub struct SystLfrCommand;

#[cfg(feature = "unit-frequency")]
impl<D> Command<D> for SystLfrCommand
where
    D: ScpiDevice + LineFrequency,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let freq: NumericValue<uom::si::f32::Frequency> = args.data()?;
        match freq {
            NumericValue::Auto => device.auto(Auto::Bool(true)),
            freq => device.line_frequency(freq),
        }
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let freq: uom::si::f32::Frequency = device.get_line_frequency();
        response.data(freq).finish()
    }
}

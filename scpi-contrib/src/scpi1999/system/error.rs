use scpi::{cmd_qonly, error::Result, tree::prelude::*};

use crate::ScpiDevice;

///## 21.8.8 \[NEXT\]?
///> `SYSTem:ERRor:NEXT?` queries the error/event queue for the next item and removes it
///> from the queue. The response returns the full queue item consisting of an integer and a string
///> as described in the introduction to the SYSTem:ERRor subsystem.
pub struct SystErrNextCommand;

impl<D> Command<D> for SystErrNextCommand
where
    D: ScpiDevice,
{
    cmd_qonly!();

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
pub struct SystErrCountCommand;

impl<D> Command<D> for SystErrCountCommand
where
    D: ScpiDevice,
{
    cmd_qonly!();

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
    cmd_qonly!();

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

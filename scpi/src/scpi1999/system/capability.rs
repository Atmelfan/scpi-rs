use crate::{cmd_qonly, error::Result, scpi1999::ScpiDevice, tree::prelude::*};

pub trait Capability: ScpiDevice {
    fn capability() -> &'static [u8];
}

/// ## 21.3 :CAPability?
///> `SYSTem:CAPability?`
///> This query returns an <instrument_specifier>. See the Compliance section in the
///> introduction to Instrument Class Applications.
pub struct SystCapCommand;

impl<D> Command<D> for SystCapCommand
where
    D: ScpiDevice + Capability,
{
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        //Always return first error (NoError if empty)
        response.data(D::capability()).finish()
    }
}

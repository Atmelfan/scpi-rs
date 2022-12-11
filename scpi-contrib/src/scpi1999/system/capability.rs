use scpi::{cmd_qonly, error::Result, tree::prelude::*};

use crate::ScpiDevice;

/// ## 21.3 :CAPability?
///> `SYSTem:CAPability?`
///> This query returns an <instrument_specifier>. See the Compliance section in the
///> introduction to Instrument Class Applications.
pub struct SystCapCommand {
    instrument_specifier: &'static [u8]
}

impl<D> Command<D> for SystCapCommand
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
        //Always return first error (NoError if empty)
        response.data(self.instrument_specifier).finish()
    }
}

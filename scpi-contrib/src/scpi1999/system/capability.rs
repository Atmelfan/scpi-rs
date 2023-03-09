use scpi::{cmd_qonly, error::Result, tree::prelude::*};

use crate::{classes::InstrumentClass, ScpiDevice};

/// ## 21.3 :CAPability?
///> `SYSTem:CAPability?`
///> This query returns an <instrument_specifier>. See the Compliance section in the
///> introduction to Instrument Class Applications.
pub struct SystCapCommand;

impl<D> Command<D> for SystCapCommand
where
    D: ScpiDevice + InstrumentClass,
{
    cmd_qonly!();

    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(D::instrument_specifier()).finish()
    }
}

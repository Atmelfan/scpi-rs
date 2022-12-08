use crate::error::Result;
use crate::nquery;
use crate::parameters::Arguments;
use crate::prelude::*;

use super::IEEE488Device;

/// *TRG
trait IEEE488Trg {
    fn exec_trg(&mut self) -> Result<()>;
}

pub struct TrgCommand;

impl<D> Command<D> for TrgCommand
where
    D: IEEE488Device + IEEE488Trg,
{
    nquery!();

    fn event(&self, device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
        // Clear any device specific status
        device.exec_trg()
    }
}

#[macro_export]
macro_rules! ieee488_trg {
    () => {
        $crate::prelude::Leaf {
            name: b"*TRG",
            default: false,
            handler: &TrgCommand,
        }
    };
}

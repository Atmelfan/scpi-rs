use scpi::{cmd_nquery, error::Result, tree::prelude::*};

use crate::ScpiDevice;

pub trait Initiate {
    /// Initiate trigger
    fn initiate(&mut self);
}

pub struct InitImmAllCommand;

impl<D> Command<D> for InitImmAllCommand
where
    D: ScpiDevice + Initiate,
{
    cmd_nquery!();

    fn event(&self, device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
        device.initiate();
        Ok(())
    }
}

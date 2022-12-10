//! IEEE488.2 Trigger command
//!
use crate::{cmd_nquery, error::Result, tree::prelude::*};

use super::IEEE488Device;

/// Implements trigger logic for the `*TRG` command
pub trait IEEE488Trg {
    /// Called when `*TRG` is executed.
    fn exec_trg(&mut self) -> Result<()>;
}

///## 10.37 *TRG, Trigger Command
///> The Trigger command is the device-specific analog of the IEEE 488.1 defined Group Execute Trigger (GET) interface
///> message, and has exactly the same effect as a GET when received, parsed, and executed by the device. GET operation
///> is discussed in detail in 6.1.4.2.5.
pub struct TrgCommand;

impl<D> Command<D> for TrgCommand
where
    D: IEEE488Device + IEEE488Trg,
{
    cmd_nquery!();

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

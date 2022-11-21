//! Contains IEEE 488.2 parser and mandatory commands
//!

use crate::error::{Error, Result};
use crate::Device;
use crate::prelude::ErrorCode;

pub mod commands;

pub trait IEEE488Device: Device {
    fn read_stb(&self) -> u8 {
        let mut reg = self.status();
        //Set ESB bit
        if self.esr() & self.ese() != 0 {
            reg |= 0x20;
        }
        //Set MSS bit
        if reg & self.sre() != 0 {
            reg |= 0x40;
        }
        // MAV bit is left to user
        reg
    }

    /// Current status of device
    /// Note
    fn status(&self) -> u8 {
        0x00
    }

    /// Service Request Enable register
    fn sre(&self) -> u8;

    /// Set the SRE register
    fn set_sre(&mut self, value: u8);

    /// Event Status Register
    fn esr(&self) -> u8;
    /// Set the ESR register
    fn set_esr(&mut self, value: u8);

    /// Event Status Enable Register
    fn ese(&self) -> u8;

    /// Set the ESE register
    fn set_ese(&mut self, value: u8);

    /// # *TST
    /// Executed when a `*TST` command is issued.
    /// See [crate::ieee488::commands::TstCommand] for details.
    ///
    /// Return Ok(()) on successfull self-test or
    /// some kind of standard or device-specific error on self-test-fault
    fn exec_tst(&mut self) -> Result<()> {
        Ok(())
    }

    /// # *RST
    /// Executed when a `*RST` command is issued.
    /// See [crate::ieee488::commands::RstCommand] for details.
    fn exec_rst(&mut self) -> Result<()> {
        Ok(())
    }

    /// # *CLS
    /// Executed when a `*CLS` command is issued.
    /// See [crate::ieee488::commands::ClsCommand] for details.
    fn exec_cls(&mut self) -> Result<()> {
        // Clear ESR
        self.set_esr(0);
        Ok(())
    }

    /// # *OPC
    /// Executed when a `*OPC` command is issued.
    /// See [crate::ieee488::commands::OpcCommand] for details.
    fn exec_opc(&mut self) -> Result<()> {
        let esr = self.esr() | ErrorCode::OperationComplete.esr_mask();
        self.set_esr(esr);
        Ok(())
    }

    /// Add a error
    fn _handle_error(&mut self, err: Error) {
        let esr = self.esr() | err.esr_mask();
        self.set_esr(esr);
    }
}


impl<T> Device for T where T: IEEE488Device  {
    fn handle_error(&mut self, err: Error) {
        <Self as IEEE488Device>::_handle_error(self, err)
    }
}
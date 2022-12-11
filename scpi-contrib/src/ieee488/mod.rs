//! Contains IEEE 488.2 parser and mandatory commands
//!

use scpi::error::Result;

pub mod common;
pub mod trg;

/// Event status/enable register bits
#[derive(Debug, Clone, Copy)]
pub enum EventStatusBit {
    /// Operation complete
    OperationComplete = 0,
    /// Request control
    RequestControl = 1,
    /// Query error
    QueryError = 2,
    /// Device dependant error
    DeviceDependantError = 3,
    /// Execution error
    ExecutionError = 4,
    /// Command error
    CommandError = 5,
    /// User request
    UserRequest = 6,
    /// Power on
    PowerOn = 7
}

impl EventStatusBit {
    pub fn mask(&self) -> u8 {
        (0x01 << *self as usize) as u8 
    }
}

/// Status byte bits
#[derive(Debug, Clone, Copy)]
pub enum StatusBit {
    /// Designer bit 0
    Designer0 = 0,
    /// Designer bit 1
    Designer1 = 1,
    /// Error/Event queue bit or designer bit 2
    ErrorEventQueue = 2,
    /// Questionable summary bit or designer bit 3
    Questionable = 3,
    /// Message available bit
    Mav = 4,
    /// Event status bit
    Esb = 5,
    /// RQS or MSS bit
    RqsMss = 6,
    /// Operation summary bit or designer bit 7
    Operation = 7
}

impl StatusBit {
    pub fn mask(&self) -> u8 {
        (0x01 << *self as usize) as u8 
    }
}

pub trait IEEE4882 {
    // Status byte register
    fn stb(&self) -> u8 {
        let mut stb = 0x00;
        // ESB
        if self.esr() &  self.ese() != 0 {
            stb |= StatusBit::Esb.mask();
        }
        // MSS
        if stb &  self.sre() != 0 {
            stb |= StatusBit::RqsMss.mask();
        }
        stb
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
    /// See [crate::ieee488::common::TstCommand] for details.
    ///
    /// Return Ok(()) on successfull self-test or
    /// some kind of standard or device-specific error on self-test-fault
    fn tst(&mut self) -> Result<()>;

    /// # *RST
    /// Executed when a `*RST` command is issued.
    /// See [crate::ieee488::common::RstCommand] for details.
    fn rst(&mut self) -> Result<()>;

    /// # *CLS
    /// Executed when a `*CLS` command is issued.
    /// See [crate::ieee488::common::ClsCommand] for details.
    fn cls(&mut self) -> Result<()>;

    /// # *OPC
    /// Executed when a `*OPC` command is issued.
    /// See [crate::ieee488::common::OpcCommand] for details.
    fn opc(&mut self) -> Result<()>;
}
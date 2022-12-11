use scpi::Device;
use scpi_contrib::{scpi1999::prelude::*, IEEE4882};

use std::collections::VecDeque;

mod measure;
mod trigger;

pub(crate) struct Voltmeter {
    /// # Mandatory IEEE488/SCPI registers
    /// Event Status Register
    pub esr: u8,
    /// Event Status Enable register
    pub ese: u8,
    /// Service Request Enable register
    pub sre: u8,
    /// OPERation:ENABle register
    pub operation: EventRegister,
    /// QUEStionable:ENABle register
    pub questionable: EventRegister,
    /// Error queue
    pub errors: VecDeque<Error>,
}

impl Voltmeter {
    pub(crate) fn new() -> Self {
        Voltmeter {
            esr: 0,
            ese: 0,
            sre: 0,
            operation: Default::default(),
            questionable: Default::default(),
            errors: Default::default(),
        }
    }
}

impl ScpiDevice for Voltmeter {

}

impl Device for Voltmeter {
    fn handle_error(&mut self, err: Error) {
        self.push_error(err);
    }
}

impl IEEE4882 for Voltmeter {
    fn stb(&self) -> u8 {
        0x00
    }

    fn sre(&self) -> u8 {
        self.sre
    }

    fn set_sre(&mut self, value: u8) {
        self.sre = value;
    }

    fn esr(&self) -> u8 {
        self.esr
    }

    fn set_esr(&mut self, value: u8) {
        self.esr = value;
    }

    fn ese(&self) -> u8 {
        self.ese
    }

    fn set_ese(&mut self, value: u8) {
        self.ese = value;
    }

    fn tst(&mut self) -> scpi::error::Result<()> {
        Ok(())
    }

    fn rst(&mut self) -> scpi::error::Result<()> {
        Ok(())
    }

    fn cls(&mut self) -> scpi::error::Result<()> {
        self.cls_standard()
    }

    fn opc(&mut self) -> scpi::error::Result<()> {
        self.opc_standard()
    }
}

impl ErrorQueue for Voltmeter {
    fn push_back_error(&mut self, err: Error) {
        self.errors.push_back(err);
    }

    fn pop_front_error(&mut self) -> Option<Error> {
        self.errors.pop_front()
    }

    fn num_errors(&self) -> usize {
        self.errors.len()
    }

    fn clear_errors(&mut self) {
        self.errors.clear()
    }
}

impl GetEventRegister<Questionable> for Voltmeter {
    fn register(&self) -> &EventRegister {
        &self.questionable
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.questionable
    }
}

impl GetEventRegister<Operation> for Voltmeter {
    fn register(&self) -> &EventRegister {
        &self.operation
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.operation
    }
}

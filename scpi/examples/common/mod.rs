use std::collections::VecDeque;

use scpi::{
    prelude::*,
    scpi1999::{
        status::{Operation, Questionable},
        EventRegister, GetEventRegister, ScpiDevice,
    },
};

pub mod completion;

pub(crate) struct TestDevice {
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

impl TestDevice {
    pub(crate) fn new() -> Self {
        TestDevice {
            esr: 0,
            ese: 0,
            sre: 0,
            operation: Default::default(),
            questionable: Default::default(),
            errors: Default::default(),
        }
    }
}

impl ScpiDevice for TestDevice {
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
}

impl ErrorQueue for TestDevice {
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

impl GetEventRegister<Questionable> for TestDevice {
    fn register(&self) -> &EventRegister {
        &self.questionable
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.questionable
    }
}

impl GetEventRegister<Operation> for TestDevice {
    fn register(&self) -> &EventRegister {
        &self.operation
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.operation
    }
}

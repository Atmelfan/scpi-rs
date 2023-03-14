//! This is an example of a minimal SCPI complient device.
//! It supports all required status registers and commands as specified in SCPI-99 standard.
//!
//!

use std::{
    collections::VecDeque,
    io::{self, BufRead, Write},
};

use scpi::{tree::Node, Context, Device, Root};
use scpi_contrib::{ieee488::prelude::*, scpi1999::prelude::*};
use scpi_contrib::{
    ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst, ieee488_sre,
    ieee488_stb, ieee488_tst, ieee488_wai, scpi_status, scpi_system,
};

struct MinimalScpiDevice {
    /// Event Status Register
    pub esr: u8,
    /// Event Status Enable register
    pub ese: u8,
    /// Service Request Enable register
    pub sre: u8,
    /// OPERation register
    pub operation: EventRegister,
    /// QUEStionable register
    pub questionable: EventRegister,
    /// Error queue
    pub errors: VecDeque<Error>,
}

impl MinimalScpiDevice {
    fn new() -> Self {
        Self {
            esr: 0,
            ese: 0,
            sre: 0,
            operation: EventRegister::default(),
            questionable: EventRegister::default(),
            errors: VecDeque::new(),
        }
    }
}

/// Handle parsing errors
impl Device for MinimalScpiDevice {
    fn handle_error(&mut self, err: Error) {
        self.push_error(err)
    }
}

// Handle common IEEE488.2 commands
impl IEEE4882 for MinimalScpiDevice {
    fn stb(&self) -> u8 {
        self.scpi_stb()
    }

    fn sre(&self) -> u8 {
        self.sre
    }

    fn set_sre(&mut self, value: u8) {
        self.sre = value
    }

    fn esr(&self) -> u8 {
        self.esr
    }

    fn set_esr(&mut self, value: u8) {
        self.esr = value
    }

    fn ese(&self) -> u8 {
        self.ese
    }

    fn set_ese(&mut self, value: u8) {
        self.ese = value
    }

    // `*TST` command
    fn tst(&mut self) -> scpi::error::Result<()> {
        Ok(())
    }

    // `*RST` command
    fn rst(&mut self) -> scpi::error::Result<()> {
        Ok(())
    }

    fn cls(&mut self) -> scpi::error::Result<()> {
        self.scpi_cls()
    }

    fn opc(&mut self) -> scpi::error::Result<()> {
        self.scpi_opc()
    }
}

// Implement Operation event register
impl GetEventRegister<Operation> for MinimalScpiDevice {
    fn register(&self) -> &EventRegister {
        &self.operation
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.operation
    }
}

// Implement Questionable event register
impl GetEventRegister<Questionable> for MinimalScpiDevice {
    fn register(&self) -> &EventRegister {
        &self.questionable
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.questionable
    }
}

// Implement an error queue
impl ErrorQueue for MinimalScpiDevice {
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

// Implement SCPI device
// Mostly done by requiring the traits above to be implmented
impl ScpiDevice for MinimalScpiDevice {}

// Create a minimal command tree
const MINIMAL_TREE: Node<MinimalScpiDevice> = Root![
    // Create default IEEE488 mandated commands
    ieee488_cls!(),
    ieee488_ese!(),
    ieee488_esr!(),
    ieee488_idn!(b"Example Inc", b"T800-101", b"0", b"0"),
    ieee488_opc!(),
    ieee488_rst!(),
    ieee488_sre!(),
    ieee488_stb!(),
    ieee488_tst!(),
    ieee488_wai!(),
    // Create default SCPI mandated STATus commands
    scpi_status!(),
    // Create default SCPI mandated SYSTem commands
    scpi_system!()
];

fn main() {
    let mut device = MinimalScpiDevice::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for command in stdin.lock().lines() {
        let command = command.unwrap().into_bytes();

        // Prepare a context and buffer to put a response into
        let mut context = Context::default();
        let mut response = Vec::new();

        // Execute command
        let res = MINIMAL_TREE.run(&command, &mut device, &mut context, &mut response);

        // Print response
        if let Ok(_) = res {
            stdout.write(&response).unwrap();
        }
    }
}

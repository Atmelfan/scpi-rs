use common::completion::ScpiHelper;
use rustyline::{error::ReadlineError, CompletionType, Config, Editor};
use scpi::{self, scpi1999::prelude::*, Context};

use std::collections::VecDeque;

// mod common doesn't work with mlti-file examples
#[path = "../common/mod.rs"]
mod common;
mod tree;

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

//
fn main() {
    let mut device = Voltmeter::new();

    let config = Config::builder()
        .history_ignore_space(true)
        .auto_add_history(true)
        .completion_type(CompletionType::List)
        .build();
    let h = ScpiHelper::new(&tree::TREE);
    let mut rl = Editor::with_config(config).expect("Failed to connect to terminal");
    rl.set_helper(Some(h));
    loop {
        let readline = rl.readline("SCPI> ");
        match readline {
            Ok(line) => {
                #[cfg(feature = "alloc")]
                let mut response = Vec::<u8>::new();

                #[cfg(not(feature = "alloc"))]
                let mut response = arrayvec::ArrayVec::<u8, 1024>::new();

                let mut context = Context::default();
                let res = tree::TREE.run(line.as_bytes(), &mut device, &mut context, &mut response);
                match res {
                    Ok(_) => {
                        if !response.is_empty() {
                            let x = String::from_utf8_lossy(response.as_slice());
                            print!("{x}");
                        }
                    }
                    Err(err) => println!("{err}"),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(_) => println!("No input"),
        }
    }
}

#[cfg(test)]
mod tests {}

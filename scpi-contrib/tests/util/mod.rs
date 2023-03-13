use std::{collections::VecDeque, path::Path};

use scpi::{error::Result, tree::prelude::*};
use serde::Deserialize;

use scpi_contrib::{ieee488::prelude::*, scpi1999::prelude::*};

// #[macro_export]
// macro_rules! check_esr {
//     ($context:ident == $esr:literal) => {
//     execute_str!($context, b"*esr?" => result, response {
//         assert_eq!(result, Ok(()));
//         assert_eq!(response, $esr);
//     });
//     };
//     ($context:ident) => {
//     check_esr!($context == b"0\n");
//     };
// }

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

impl Device for TestDevice {
    fn handle_error(&mut self, err: Error) {
        self.push_error(err)
    }
}

impl ScpiDevice for TestDevice {}

impl IEEE4882 for TestDevice {
    fn stb(&self) -> u8 {
        let mut stb = 0x00;
        if !self.is_empty() {
            stb |= scpi_contrib::ieee488::StatusBit::ErrorEventQueue.mask();
        }
        if self.get_register_summary::<Questionable>() {
            stb |= scpi_contrib::ieee488::StatusBit::Questionable.mask();
        }
        if self.get_register_summary::<Operation>() {
            stb |= scpi_contrib::ieee488::StatusBit::Operation.mask();
        }
        // ESB
        if self.esr() & self.ese() != 0 {
            stb |= scpi_contrib::ieee488::StatusBit::Esb.mask();
        }
        // MSS
        if stb & self.sre() != 0 {
            stb |= scpi_contrib::ieee488::StatusBit::RqsMss.mask();
        }
        stb
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

    fn tst(&mut self) -> Result<()> {
        Ok(())
    }

    fn rst(&mut self) -> Result<()> {
        Ok(())
    }

    fn cls(&mut self) -> Result<()> {
        self.scpi_cls()
    }

    fn opc(&mut self) -> Result<()> {
        self.scpi_opc()
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Record {
    command: String,
    error: i16,
    response: String,
}

#[allow(dead_code)]
pub fn test_file<P, D: Device>(dev: &mut D, tree: &Node<D>, path: P)
where
    P: AsRef<Path>,
{
    let mut rdr = csv::ReaderBuilder::default()
        .has_headers(true)
        .comment(Some(b'#'))
        .from_path(path)
        .unwrap();

    for result in rdr.deserialize() {
        // Get test case
        let mut record: Record = result.unwrap();
        record.response = record.response.replace("\\n", "\n");

        // Execute command
        let res = test_execute_str(tree, record.command.as_bytes(), dev);

        // Compare result
        match res {
            Ok(buf) => {
                assert_eq!(record.error, 0, "Expected command error {}", record.command);
                assert!(
                    record
                        .response
                        .as_bytes()
                        .eq_ignore_ascii_case(buf.as_slice()),
                    "Unexpected response {}\n\tExpected: {:?}\n\t  Actual: {:?}",
                    record.command,
                    record.response,
                    std::str::from_utf8(buf.as_slice()).unwrap_or("<Invalid utf8>")
                );
            }
            Err(err) => {
                assert_eq!(
                    record.error,
                    err.get_code(),
                    "Unexpected command error {:?}, {}",
                    err,
                    record.command
                );
            }
        }
        println!("OK {:?}", record);
    }
}

pub fn test_execute_str<D: Device>(tree: &Node<D>, s: &[u8], dev: &mut D) -> Result<Vec<u8>> {
    let mut context = Context::default();
    let mut buf = Vec::new();
    //Result
    tree.run(s, dev, &mut context, &mut buf)?;
    Ok(buf)
}

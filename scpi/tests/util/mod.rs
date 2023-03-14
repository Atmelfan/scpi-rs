use std::path::Path;

use scpi::{error::Result, tree::prelude::*};
use serde::Deserialize;

pub(crate) struct TestDevice;

impl TestDevice {
    pub(crate) fn new() -> Self {
        TestDevice
    }
}

impl Device for TestDevice {
    fn handle_error(&mut self, _err: Error) {}
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

#[cfg(feature = "alloc")]
pub fn test_execute_str<D: Device>(tree: &Node<D>, s: &[u8], dev: &mut D) -> Result<Vec<u8>> {
    let mut context = Context::default();
    let mut buf = Vec::new();
    //Result
    tree.run(s, dev, &mut context, &mut buf)?;
    Ok(buf)
}

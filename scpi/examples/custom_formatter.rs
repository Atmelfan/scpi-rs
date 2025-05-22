use scpi::{
    error::{self, Result},
    tree::prelude::*,
};

use std::io::{self, BufRead, Write};

struct MyDevice;
impl Device for MyDevice {
    fn handle_error(&mut self, err: Error) {
        eprintln!("Error: {err}")
    }
}

struct HelloWorldCommand;
impl Command<MyDevice> for HelloWorldCommand {
    // Allow only queries
    cmd_qonly!();

    // Called when a query is made
    fn query(
        &self,
        _device: &mut MyDevice,
        _context: &mut Context,
        mut params: Parameters,
        mut resp: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let target: Option<&str> = params.next_optional_data()?;
        if let Some(target) = target {
            let greeting = format!("Hello {target}");
            resp.data(greeting.as_bytes()).finish()
        } else {
            resp.data(b"Hello world".as_slice()).finish()
        }
    }
}

struct MyCustomFormatter;

impl Formatter for MyCustomFormatter {
    fn push_str(&mut self, s: &[u8]) -> Result<()> {
        std::io::stdout()
            .write(s)
            .map_err(|_| error::ErrorCode::ExecutionError)?;
        Ok(())
    }

    fn push_byte(&mut self, b: u8) -> Result<()> {
        std::io::stdout()
            .write(&[b])
            .map_err(|_| error::ErrorCode::ExecutionError)?;
        Ok(())
    }
}

const MYTREE: Node<MyDevice> = Root![
    Leaf!(b"*COM" => &HelloWorldCommand),
    Branch![ b"HELLo";
        Leaf!(default b"WORLd" => &HelloWorldCommand)
    ]
];

fn main() {
    let mut device = MyDevice;

    let stdin = io::stdin();
    for command in stdin.lock().lines() {
        let command = command.unwrap().into_bytes();

        // Prepare a context and buffer to put a response into
        let mut context = Context::default();

        // Execute command
        let _res = MYTREE.run(&command, &mut device, &mut context, &mut MyCustomFormatter);
    }
}

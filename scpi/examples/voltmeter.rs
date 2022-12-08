use std::io::Read;

use common::{completion::ScpiHelper, TestDevice};
use rustyline::{error::ReadlineError, CompletionType, Config, Editor};
use scpi::{
    self, ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc, ieee488_rst,
    ieee488_sre, ieee488_stb, ieee488_tst, ieee488_wai, scpi_status, scpi_system,
    tree::Node::{self, Branch},
    Context,
};

mod common;

const TREE: Node<TestDevice> = Branch {
    name: b"",
    sub: &[
        ieee488_cls!(),
        ieee488_ese!(),
        ieee488_esr!(),
        ieee488_idn!(b"GPA-Robotics", b"T800-101", b"0", b"0"),
        ieee488_opc!(),
        ieee488_rst!(),
        ieee488_sre!(),
        ieee488_stb!(),
        ieee488_tst!(),
        ieee488_wai!(),
        scpi_status!(),
        scpi_system!(),
    ],
};

fn main() {
    let mut device = TestDevice::new();

    let config = Config::builder()
        .history_ignore_space(true)
        .auto_add_history(true)
        .completion_type(CompletionType::List)
        .build();
    let h = ScpiHelper::new(&TREE);
    let mut rl = Editor::with_config(config).expect("Failed to connect to terminal");
    rl.set_helper(Some(h));
    loop {
        let readline = rl.readline("SCPI> ");
        match readline {
            Ok(line) => {
                let mut response = Vec::<u8>::new();
                let mut context = Context::new();
                let res = TREE.run(line.as_bytes(), &mut device, &mut context, &mut response);
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

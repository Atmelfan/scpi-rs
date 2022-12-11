use common::completion::ScpiHelper;
use device::Voltmeter;
use rustyline::{error::ReadlineError, CompletionType, Config, Editor};
use scpi::{self, prelude::*};

// mod common doesn't work with mlti-file examples
#[path = "../common/mod.rs"]
mod common;
mod tree;
mod device;

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
                let mut response = scpi::arrayvec::ArrayVec::<u8, 1024>::new();

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

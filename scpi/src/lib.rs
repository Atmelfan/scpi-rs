#![no_std]

#![feature(external_doc)]
#![doc(include = "../README.md")]


#[macro_use]
extern crate scpi_derive;

/* Used to create responses */
extern crate arrayvec;
extern crate lexical_core;
extern crate arraydeque;

pub mod error;
pub mod command;
pub mod suffix;
pub mod tokenizer;
pub mod ieee488;
pub mod tree;
pub mod response;
pub mod scpi;
pub mod expression;

/// Prelude containing the most useful stuff
///
pub mod prelude {
    pub use crate::error::{Error, ErrorQueue, ArrayErrorQueue};
    pub use crate::tokenizer::{Tokenizer, Token};
    pub use crate::command::Command;
    pub use crate::tree::Node;
    pub use crate::{Context, Device};
}

use crate::error::{Error, ErrorQueue};
use crate::tokenizer::{Tokenizer, Token};
use crate::response::Formatter;
use crate::tree::Node;
use crate::scpi::{EventRegister};

/// A SCPI device
///
/// Use this trait and provided mandatory commands to implement the mandatory parts of SCPI for your device.
///
///
pub trait Device {

    /// Called by *CLS
    fn cls(&mut self) -> Result<(), Error>;

    /// Called by *RST
    fn rst(&mut self) -> Result<(), Error>;

    /// Called by *TST?
    /// Return zero if self-test is successful or a positive user-error code or a
    /// standard negative error code (as a Error enum variant).
    fn tst(&mut self) -> Result<i16, Error> {
        Ok(0)
    }

}


/// Context in which to execute a message.
///
/// Contains registers related to the context and reference to the writer to respond with.
/// Also contains a reference to the Device (may be shared by multiple contexts (**Note! If threadsafe**)).
pub struct Context<'a> {
    /// SCPI command tree root
    root: &'a Node<'a>,
    /// Device executed upon
    pub device: &'a mut dyn Device,
    /// Error queue
    pub errors: &'a mut dyn ErrorQueue,
    /// Event Status Register
    pub esr: u8,
    /// Event Status Enable register
    pub ese: u8,
    /// Service Request Enable register
    pub sre: u8,

    /// OPERation:ENABle register
    pub operation: EventRegister,
    ///QUEStionable:ENABle register
    pub questionable: EventRegister
}

impl<'a> Context<'a> {

    /// Create a new context
    ///
    /// # Arguments
    ///  * `device` - Device to act upon
    ///  * `writer` - Writer used to write back response messages
    ///  * `root` - SCPI command tree to use
    pub fn new(device: &'a mut dyn Device, errorqueue: &'a mut dyn ErrorQueue, root: &'a Node<'a>) -> Self{
        Context {
            device,
            errors: errorqueue,
            root,
            esr: 0,
            ese: 0,
            sre: 0,
            operation: EventRegister::new(),
            questionable: EventRegister::new()
        }
    }

    /// Executes one SCPI message (terminated by `\n`) and queue any errors.
    ///
    /// # Arguments
    ///  * `tokenizer` - A tokenizer created from `Tokenizer::from_str(...)`. May be re-used
    ///  if still has valid tokens and Ok() was returned.
    ///
    /// # Returns
    ///  * `Ok(())` - If message (and all units within) was executed successfully
    ///  * `Err(error)` - If parser detected or a command returned an error
    ///
    pub fn exec(&mut self, tokenstream: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error>{
        response.clear();
        self.execute(tokenstream, response).map_err(|err| {
            //Set appropriate bits in ESR
            self.esr |= err.clone().esr_mask();

            //Queue error
            self.errors.push_back_error(err);

            //Original error
            err
        })
    }

    fn execute(&mut self, tokenstream: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error>{
        // Point the current branch to root
        let mut is_query = false;
        let mut is_common = false;

        let mut branch = self.root;//Node parent
        let mut node: Option<&Node> = None;//Current active node

        //Start response message
        response.message_start()?;
        'outer: while let Some(token) = tokenstream.next() {
            let tok = token?;
            //println!(">>> {:?}", tok);
            match tok {
                Token::ProgramMnemonic(_) => {
                    //Common nodes always use ROOT as base node
                    let subcommands = if is_common {
                        self.root.sub
                    }else{
                        branch.sub
                    }.ok_or(Error::UndefinedHeader)?;

                    for sub in subcommands {

                        if is_common {
                            //Common nodes must match mnemonic and start with '*'
                            if sub.name.starts_with(b"*") && tok.eq_mnemonic(&sub.name[1..]) {
                                node = Some(sub);
                                continue 'outer;
                            }
                        } else if tok.eq_mnemonic(sub.name) {
                            //Normal node must match mnemonic
                            node = Some(sub);
                            continue 'outer;
                        } else if sub.optional && sub.sub.is_some(){
                            //A optional node may have matching children
                            for subsub in sub.sub.unwrap() {
                                if tok.eq_mnemonic(subsub.name) {
                                    //Normal node must match mnemonic
                                    node = Some(subsub);
                                    continue 'outer;
                                }
                            }
                        }
                    }

                    return Err(Error::UndefinedHeader);
                }
                Token::HeaderMnemonicSeparator => {
                    //This node will be used as branch
                    if let Some(p) = node {
                        branch = p;
                    }else{
                        branch = self.root;
                    }
                    //println!("branch={}", String::from_utf8_lossy(branch.name));
                }
                Token::HeaderCommonPrefix => {
                    is_common = true;
                }
                Token::HeaderQuerySuffix => {
                    is_query = true;
                }
                Token::ProgramMessageTerminator => {
                    //
                    break 'outer;
                }
                Token::ProgramHeaderSeparator | Token::ProgramMessageUnitSeparator => {
                    // Execute header if available
                    if let Some(n) = node {

                        if tok == Token::ProgramHeaderSeparator {
                            //If a header separator was found, pass tokenizer for arguments and
                            // check that the command consumes them all
                            n.exec(self, tokenstream, response, is_query)?;

                            // Should have a terminator, unit terminator or END after arguments
                            // If, not, the handler has not consumed all arguments (error) or an unexpected token appeared.'
                            // TODO: This should abort above command!
                            if let Some(t) = tokenstream.next() {
                                match t? {
                                    Token::ProgramMessageTerminator | Token::ProgramMessageUnitSeparator => (),
                                    /* Leftover data objects */
                                    Token::CharacterProgramData(_) | Token::DecimalNumericProgramData(_) | Token::SuffixProgramData(_) |
                                    Token::NonDecimalNumericProgramData(_) | Token::StringProgramData(_) | Token::ArbitraryBlockData(_) |
                                    Token::ProgramDataSeparator => {
                                        return Err(Error::ParameterNotAllowed)
                                    },
                                    /* Shouldn't happen? */
                                    _ => {
                                        return Err(Error::SyntaxError)
                                    },
                                }
                            }
                        }else{
                            //No header separator was found = no arguments, pass an empty tokenizer
                            n.exec(self, &mut Tokenizer::empty(), response, is_query)?;
                        }

                    }else{
                        //return Err(Error::CommandHeaderError);
                    }

                    // Reset unit state
                    node = None;
                    is_query = false;
                    is_common = false;
                }
                _ => ()
            }
        }

        //Execute last command if any (never has arguments or would've been executed earlier)
        if let Some(n) = node {
            n.exec(self, &mut Tokenizer::empty(), response, is_query)?;
        }

        //End response message if anything has written something
        if !response.is_empty() {
            response.message_end()?;
        }


        Ok(())
        //branch.exec(self, tokenstream.clone().borrow_mut(), is_query)?;

    }

    fn get_stb(&self) -> u8 {
        let mut reg = 0u8;
        //Set OPERation status bits
        if self.operation.get_summary() { reg |= 0x80; }
        //Set QUEStionable status bit
        if self.questionable.get_summary() { reg |= 0x08; }
        //Set error queue empty bit
        if self.errors.not_empty() { reg |= 0x10; }
        //Set event bit
        if self.esr & self.ese != 0 { reg |= 0x20; }
        //Set MSS bit
        if reg & self.sre != 0{ reg |= 0x40; }

        reg
    }

    fn set_ese(&mut self, ese: u8) {
        self.ese = ese;
    }

    fn get_ese(&mut self) -> u8 {
        self.ese
    }
}
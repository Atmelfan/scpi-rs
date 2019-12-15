//! Contains IEEE 488.2 parser and mandatory commands
//!


use crate::error::Error;
use crate::tree::Node;
use crate::tokenizer::{Tokenizer, Token};
use crate::Device;

use core::fmt::Write;

/// Context in which to execute a message.
///
/// Contains registers related to the context and reference to the writer to respond with.
/// Also contains a reference to the Device (may be shared by multiple contexts (**Note! If threadsafe**)).
pub struct Context<'a> {
    /// SCPI command tree root
    root: &'a Node<'a>,
    /// Device executed upon
    pub device: &'a mut dyn Device,
    /// Writer to respond with
    pub writer: &'a mut dyn Write,
    /// Event Status Register
    pub esr: u8,
    /// Event Status Enable register
    pub ese: u8,
    /// Service Request Enable register
    pub sre: u8
}

impl<'a> Context<'a> {

    /// Create a new context
    ///
    /// # Arguments
    ///  * `device` - Device to act upon
    ///  * `writer` - Writer used to write back response messages
    ///  * `tree` - SCPI command tree to use
    pub fn new(device: &'a mut dyn Device, writer: &'a mut dyn Write, tree: &'a Node<'a>) -> Self{
        Context {
            device,
            writer,
            root: tree,
            esr: 0,
            ese: 0,
            sre: 0
        }
    }

    /// Executes one SCPI message (terminated by `\n`) and queue any errors.
    ///
    /// # Arguments
    ///  * `tokenizer` - A tokenizer created from `Tokenizer::from_str(...)`. May be re-used
    ///  if still has valid tokens and Ok() was returned.
    ///
    /// # Returns
    ///  * `Ok(())` - If successful
    ///  * `Err(error)` - If parser detected or a command returned an error
    ///
    pub fn exec(&mut self, tokenstream: &mut Tokenizer) -> Result<(), Error>{
        self.execute(tokenstream).map_err(|err| {
            //Queue error and set appropriate bits in esr
            self.esr |= err.clone().esr_mask();
            let _ = self.device.error_enqueue(err.clone());
            err
        })
    }

    fn execute(&mut self, tokenstream: &mut Tokenizer) -> Result<(), Error>{
        // Point the current branch to root
        let mut is_query = false;
        let mut is_common = false;

        let mut branch = self.root;//Node parent
        let mut node: Option<&Node> = None;//Current active node
        'outer: while let Some(token) = tokenstream.next() {
            let tok = token?;
            //println!(">>> {:?}", tok);
            match tok {
                Token::ProgramMnemonic(_) => {
                    //println!(":{}", String::from_utf8_lossy(mnemonic));

                    let subcommands = if is_common {
                        self.root.sub
                    }else{
                        branch.sub
                    }.ok_or(Error::UndefinedHeader)?;

                    for sub in subcommands {

                        //println!("{} = {}", String::from_utf8_lossy(mnemonic), String::from_utf8_lossy(sub.name));
                        //Continue to look for match
                        if is_common {
                            if sub.name.starts_with(b"*") && tok.eq_mnemonic(&sub.name[1..]) {
                                node = Some(sub);
                                continue 'outer;
                            }
                        } else if tok.eq_mnemonic(sub.name) {
                            node = Some(sub);
                            continue 'outer;
                        }
                    }
                    return Err(Error::UndefinedHeader);
                }
                Token::HeaderMnemonicSeparator => {
                    //Leading ':'

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
                    if let Some(n) = node {
                        if tok == Token::ProgramHeaderSeparator {
                            n.exec(self, tokenstream, is_query)?;
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
                            n.exec(self, &mut Tokenizer::empty(), is_query)?;
                        }

                    }else{
                        return Err(Error::CommandHeaderError);
                    }

                    node = None;

                    is_query = false;
                    is_common = false;
                }
                _ => ()
            }
        }

        if let Some(n) = node {
            n.exec(self, &mut Tokenizer::empty(), is_query)?;
        }
        Ok(())
        //branch.exec(self, tokenstream.clone().borrow_mut(), is_query)?;

    }

    fn get_stb(&self) -> u8 {
        let mut reg = 0u8;
        //Set OPERation status bits
        if self.device.oper_status() != 0 { reg |= 0x80; }
        //Set QUEStionable status bit
        if self.device.ques_status() != 0 { reg |= 0x08; }
        //Set error queue empty bit
        if self.device.error_len() == 0 { reg |= 0x10; }
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

/// Contains basic implementations of mandated IEEE 488.2 commands
pub mod commands {
    use crate::command::Command;
    use crate::Context;
    use crate::error::Error;
    use crate::tokenizer::Tokenizer;

    /// *CLS
    ///> This command clears all status data structures in a device. For a device which minimally
    ///> complies with SCPI, these registers are:
    ///>  * SESR
    ///>  * OPERation Status Register
    ///>  * QUEStionable Status Register
    ///>  * Error/Event Queue
    ///> Execution of *CLS shall also clear any additional status data structures implemented in the
    ///> device. The corresponding enable registers are unaffected. See the table in Command
    ///> Reference, 20.7.
    pub struct ClsCommand{}

    impl Command for ClsCommand {
        fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            Err(Error::UndefinedHeader)
        }

        fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            context.device.cls()
        }
    }

    pub struct EseCommand{}

    impl Command for EseCommand {
        fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }

        fn query(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }
    }

    ///
    ///
    pub struct IdnCommand<'a> {
        pub manufacturer: &'a [u8],
        pub model: &'a [u8],
        pub serial: &'a [u8],
        pub firmware: &'a [u8]
    }

    impl<'a> Command for IdnCommand<'a> {
        fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            Err(Error::UndefinedHeader)
        }

        fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            writeln!(context.writer, "*IDN? -> {:?},{:?},{:?},{:?}", self.manufacturer, self.model, self.serial, self.firmware).unwrap();
            Ok(())
        }
    }
}


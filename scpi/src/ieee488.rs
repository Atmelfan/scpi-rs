//! Contains IEEE 488.2 parser and mandatory commands
//!


use crate::error::Error;
use crate::tree::Node;
use crate::tokenizer::{Tokenizer, Token};
use crate::Device;
use crate::response::{ArrayVecFormatter, Formatter};

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
    pub response: &'a mut dyn Formatter,
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
    pub fn new(device: &'a mut dyn Device, response: &'a mut dyn Formatter, tree: &'a Node<'a>) -> Self{
        Context {
            device,
            response,
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
        self.response.clear();
        self.execute(tokenstream).map_err(|err| {
            //Set appropriate bits in ESR
            self.esr |= err.clone().esr_mask();

            //Try to queue the error...
            if let Err(queue_err) = self.device.error_enqueue(err) {
                self.esr |= queue_err.esr_mask();
            }

            //Try to push error in response
            self.response.clear();
            if let Err(new_err) = self.response.error(err) {
                self.esr |= err.esr_mask();
            }

            err
        })
    }

    fn execute(&mut self, tokenstream: &mut Tokenizer) -> Result<(), Error>{
        // Point the current branch to root
        let mut is_query = false;
        let mut is_common = false;

        let mut branch = self.root;//Node parent
        let mut node: Option<&Node> = None;//Current active node

        //Start response message
        self.response.message_start()?;
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
                            //No header separator was found = no arguments, pass an empty tokenizer
                            n.exec(self, &mut Tokenizer::empty(), is_query)?;
                        }

                    }else{
                        return Err(Error::CommandHeaderError);
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
            n.exec(self, &mut Tokenizer::empty(), is_query)?;
        }

        //End response message
        self.response.message_end()?;

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

/// Contains basic implementations of mandated IEEE 488.2 commands.
///
/// Mandatory IEEE488.2 commands:
///
/// | Mnemonic | Name                                 | 488.2 Section |
/// |----------|--------------------------------------|---------------|
/// | *CLS     | Clear Status Command                 | 10.3          |
/// | *ESE     | Standard Event Status Enable Command | 10.10         |
/// | *ESE?    | Standard Event Status Enable Query   | 10.11         |
/// | *ESR?    | Standard Event Status Register Query | 10.12         |
/// | *IDN?    | Identification Query                 | 10.14         |
/// | *OPC     | Operation Complete Command           | 10.18         |
/// | *OPC?    | Operation Complete Query             | 10.19         |
/// | *RST     | Reset Command                        | 10.32         |
/// | *SRE     | Service Request Enable Command       | 10.34         |
/// | *SRE?    | Service Request Enable Query         | 10.35         |
/// | *STB     | Read Status Byte Query               | 10.36         |
/// | *TST     | Self-Test Query                      | 10.38         |
/// | *WAI     | Wait-To-Continue                     | 10.39         |
///
/// Note that the comments about the default mandatory commands below are from the IEEE 488.2-1992 document and explain their purpose, not my implementation.
pub mod commands {
    use crate::command::Command;
    use crate::Context;
    use crate::error::Error;
    use crate::tokenizer::Tokenizer;

    /// Creates a stub for event()
    ///
    macro_rules! qonly {
        () => {
            fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
                Err(Error::UndefinedHeader)
            }
        };
    }

    /// Creates a stub for query()
    ///
    macro_rules! nquery {
        () => {
            fn query(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
                Err(Error::UndefinedHeader)
            }
        };
    }

    ///## 10.3 *CLS, Clear Status Command
    ///> The Clear Status command clears status data structures, see 11.1.2, and forces the device to the Operation Complete
    ///> Command Idle State and the Operation Complete Query Idle State, see 12.5.2 and 12.5.3.
    ///>
    ///> If the Clear Status command immediately follows a <PROGRAM MESSAGE TERMINATOR>, the Output Queue
    ///> and the MAV bit will be cleared because any new <PROGRAM MESSAGE> after a <PROGRAM MESSAGE
    ///> TERMINATOR> clears the Output Queue, see 6.3.2.3.
    pub struct ClsCommand;

    impl Command for ClsCommand { nquery!();

        fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            context.device.cls()
        }
    }

    ///## 10.10 *ESE, Standard Event Status Enable Command
    ///> The Standard Event Status Enable command sets the Standard Event Status Enable Register bits as defined in 11.5.1.3.
    ///## 10.11 *ESE?, Standard Event Status Enable Query
    ///> The Standard Event Status Enable query allows the programmer to determine the current contents of the Standard
    ///> Event Status Enable Register. See 11.5.1.3.
    pub struct EseCommand;

    impl Command for EseCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            let ese = (args.next_f32(false)?.unwrap() + 0.5f32) as i32;
            if ese >= 0i32 && ese < 256i32 {
                context.ese = ese as u8;
                Ok(())
            }else{
                Err(Error::DataOutOfRange)
            }
        }

        fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            context.response.u8_data(context.ese)
        }
    }

    ///## 10.12 *ESR?, Standard Event Status Register Query
    ///> The Standard Event Status Register query allows the programmer to determine the current contents of the Standard
    ///> Event Status Register. Reading the Standard Event Status Register clears it. See 11.5.1.2.
    pub struct EsrCommand;

    impl Command for EsrCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            context.response.u8_data(context.esr)
        }
    }

    ///## 10.14 *IDN?, Identification Query
    ///> The intent of the Identification query is for the unique identification of devices over the system interface.
    ///
    ///### 4.1.3.6 SCPI-99 Comments:
    ///> IEEE 488.2 is purposefully vague about the content of each of the four fields in the response
    ///> syntax. SCPI adds no further requirement, but here are some suggestions:
    ///>
    ///> All devices produced by a company should implement the *IDN? response consistently.
    ///>  * Field 1, the Manufacturer field, should be identical for all devices produced by a single company.
    ///>  * Field 2, the Model field, should NOT contain the word “MODEL”.
    ///>  * Field 4, the Firmware level field, should contain information about all separately revisable subsystems.
    ///> This information can be contained in single or multiple revision codes.
    pub struct IdnCommand<'a> {
        pub manufacturer: &'a [u8],
        pub model: &'a [u8],
        pub serial: &'a [u8],
        pub firmware: &'a [u8]
    }

    impl<'a> Command for IdnCommand<'a> { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {

            //TODO: Make this easier
            context.response.ascii_data(self.manufacturer)?;
            context.response.separator()?;
            context.response.ascii_data(self.model)?;
            context.response.separator()?;
            context.response.ascii_data(self.serial)?;
            context.response.separator()?;
            context.response.ascii_data(self.firmware)?;

            Ok(())
        }
    }

    ///## 10.18 *OPC, Operation Complete Command
    ///> The Operation Complete command causes the device to generate the operation complete message in the Standard
    ///> Event Status Register when all pending selected device operations have been finished. See 12.5.2.2 for details of
    ///> operation.
    ///## 10.19 *OPC?, Operation Complete Query
    ///> The Operation Complete query places an ASCII character "1" into the device's Output Queue when all pending
    ///> selected device operations have been finished. See 12.5.3 for details of operation.
    ///
    pub struct OpcCommand;
    impl Command for OpcCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }

        fn query(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            context.response.ascii_data(b"1")
        }
    }

    ///## 10.32 *RST, Reset Command
    ///> The Reset command performs a device reset. The Reset command is the third level of reset in a three-level reset
    ///> strategy, see 17.1.2 and Appendix D. The Reset command shall do the following:
    ///>  * Except as explicitly excluded below, set the device-specific functions to a known state that is independent of
    ///> the past-use history of the device. Device-specific commands may be provided to program a different reset
    ///>  * state than the original factory-supplied one.
    ///>  * Set the macro defined by *DDT to a device-defined state, see 10.4.
    ///>  * Disable macros, see 10.8.
    ///>  * Force the device into the OCIS state, see 12.5.2.
    ///>  * Force the device into the OQIS state, see 12.5.3.
    ///> The reset command explicitly shall NOT affect the following:
    ///>  * The state of the IEEE 488.1 interface.
    ///>  * The selected IEEE 488.1 address of the device.
    ///>  * The Output Queue.
    ///>  * Any Event Enable Register setting, including the Standard Event Status Enable Register settings, see
    ///> 11.4.2.3.4 and 11.5.1.3.4.
    ///>  * Any Event Register setting, including the Standard Event Status Register settings, see 11.4.2.2.4 and
    ///> 11.5.1.2.4.
    ///>  * The power-on-status-clear flag setting.
    ///>  * Macros defined with the DeÞne Macro Contents command.
    ///>  * Calibration data that affects device specifications.
    ///>  * The Protected User Data query response.
    ///>  * The Resource Description Transfer query response.
    ///>  * The Service Request Enable Register setting, see 11.3.2.4.
    ///>  * The Parallel Poll Enable Register setting, see 11.6.1.4.
    ///>  * The memory register(s) associated with *SAV.
    ///> The scope of the *LRN? response and *RCL (if implemented) is the same as *RST. See 10.17.3 and 10.29.3.
    pub struct RstCommand;
    impl Command for RstCommand { nquery!();

        fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
            context.device.rst()
        }
    }

    ///## 10.34 *SRE, Service Request Enable Command
    ///> The Service Request Enable command sets the Service Request Enable Register bits as defined in 11.3.2.
    ///## 10.35 *SRE?, Service Request Enable Query
    ///> The Service Request Enable query allows the programmer to determine the current contents of the Service Request
    ///> Enable Register, see 11.3.2.
    pub struct SreCommand;
    impl Command for SreCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }

        fn query(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }
    }

    ///## 10.36 *STB?, Read Status Byte Query
    ///> The Read Status Byte query allows the programmer to read the status byte and Master Summary Status bit.
    pub struct StbCommand;
    impl Command for StbCommand { qonly!();

        fn query(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            context.response.u8_data(context.get_stb())
        }
    }

    ///## 10.38 *TST?, Self-Test Query
    ///> The self-test query causes an internal self-test and places a response into the Output Queue indicating whether or not
    ///> the device completed the self-test without any detected errors. Optionally, information on why the self-test was not
    ///> completed may be contained in the response. The scope of the internal self-test shall appear in the device
    ///> documentation, see 4.9.
    ///>
    ///> The *TST? query shall not require any local operator interaction. It shall not create bus conditions that are violations
    ///> to the IEEE Std 488.1-1987 [4] or IEEE Std 488.2-1992 standards. Otherwise, the scope of the self-test is completely
    ///> at the discretion of the device designer.
    ///>
    ///> Upon successful completion of *TST?, the device settings shall be restored to their values prior to the *TST?; set to
    ///> fixed, known values that are stated in the device documentation; or set to values deÞned by the user and stored in local
    ///> memory.
    pub struct TstCommand;
    impl Command for TstCommand { qonly!();

        fn query(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }
    }

    ///## 10.39 *WAI, Wait-to-Continue Command
    ///> The Wait-to-Continue command shall prevent the device from executing any further commands or queries until the no-
    ///> operation-pending flag is TRUE. See 12.5.1.
    ///>
    ///> NOTE - In a device that implements only sequential commands, the no-operation-pending flag is always TRUE
    pub struct WaiCommand;
    impl Command for WaiCommand { nquery!();
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            unimplemented!()
        }
    }
}


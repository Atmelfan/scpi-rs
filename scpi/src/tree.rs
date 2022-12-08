//! Used to build a SCPI command tree

use core::iter::Peekable;
//extern crate std;

use crate::command::Command;
use crate::error::{Error, ErrorCode, Result};
use crate::parameters::Arguments;
use crate::response::Formatter;
use crate::tokenizer::{Token, Tokenizer};
use crate::{Context, Device};

/// A SCPI command node
/// These nodes are structured as a command tree where each node represent a SCPI header mnemonic.
///
/// # Example
///
/// ```
/// //TODO
/// ```
/// Note that all strings are ascii-/bytestrings, this is because only ASCII is defined in SCPI thus
/// the normal UTF8 &str in rust would be improper. To send a unicode string you can use Arbitrary Block Data
/// (or, this parser has an alternative arbitrary data header `#s"..."` which allows and checks UTF8 data inside the quotes.
///
pub enum Node<'a, D> {
    Leaf {
        /// Name of this leaf node
        name: &'static [u8],
        /// Default node, will be executed if the branch immediately below is executed.
        /// Only one default node is allowed in each branch.
        default: bool,
        /// Command handler
        handler: &'a dyn Command<D>,
    },
    Branch {
        /// Name of this branch node
        name: &'static [u8],
        /// Child nodes
        /// **Note:** Default child must be first!
        sub: &'a [Node<'a, D>],
    },
}

impl<'a, D> Node<'a, D> {
    pub fn name(&self) -> &'static [u8] {
        match self {
            Self::Leaf { name, .. } => name,
            Self::Branch { name, .. } => name,
        }
    }
}

#[macro_export]
macro_rules! tree {
    ($tree:expr) => {
        $crate::tree::Node::Branch {
            name: b"",
            sub: $tree,
        }
    };
}

impl<'a, D> Node<'a, D>
where
    D: Device,
{
    pub fn run<FMT>(
        &self,
        s: &[u8],
        device: &mut D,
        context: &mut Context,
        response: &mut FMT,
    ) -> Result<()>
    where
        FMT: Formatter,
    {
        let mut tokenizer = Tokenizer::new(s).peekable();
        let res = self.run_tokens(device, context, &mut tokenizer, response);
        if let Err(err) = &res {
            device.handle_error(*err);
        }
        res
    }

    pub(crate) fn run_tokens<FMT>(
        &self,
        device: &mut D,
        context: &mut Context,
        tokens: &mut Peekable<Tokenizer>,
        response: &mut FMT,
    ) -> Result<()>
    where
        FMT: Formatter,
    {
        let mut leaf = self;

        //Start response message
        response.message_start()?;
        loop {
            // Execute header
            match tokens.peek() {
                // :header..
                Some(Ok(Token::HeaderMnemonicSeparator)) => {
                    leaf = self;
                    // Consume seperator
                    tokens.next();
                    self.exec(&mut leaf, device, context, tokens, response)?;
                }
                // header.. | *header
                Some(Ok(Token::ProgramMnemonic(s))) => {
                    if s.starts_with(b"*") {
                        let mut _x = self;
                        self.exec(&mut _x, device, context, tokens, response)?;
                    } else {
                        leaf.exec(&mut leaf, device, context, tokens, response)?;
                    }
                }
                // Empty input
                None => break Ok(()),
                //
                Some(Err(err)) => break Err(Error::new(*err)),
                // idk?
                Some(_) => break Err(ErrorCode::SyntaxError.into()),
            }
            // Should've consumed up to unit seperator

            // What's next?
            match tokens.next() {
                // EOM
                None => {
                    if !response.is_empty() {
                        response.message_end()?;
                    }
                    break Ok(());
                }
                // New unit
                Some(Ok(Token::ProgramMessageUnitSeparator)) => {
                    continue;
                }
                // More tokens...
                Some(Ok(tok)) => {
                    if tok.is_data() || tok == Token::ProgramDataSeparator {
                        break Err(ErrorCode::ParameterNotAllowed.into());
                    } else {
                        break Err(ErrorCode::SyntaxError.into());
                    }
                }
                // Error
                Some(Err(err)) => break Err(Error::new(err)),
            }
        }
    }

    pub(crate) fn exec<FMT>(
        &'a self,
        leaf: &mut &'a Node<'a, D>,
        device: &mut D,
        context: &mut Context,
        tokens: &mut Peekable<Tokenizer>,
        response: &mut FMT,
    ) -> Result<()>
    where
        FMT: Formatter,
    {
        let next = match tokens.peek() {
            Some(Ok(tok)) => Some(tok),
            Some(Err(err)) => return Err(Error::new(*err)),
            None => None,
        };

        //extern crate std;

        match self {
            Node::Leaf { handler, .. } => {
                //std::println!("Leaf {}", std::str::from_utf8(name).unwrap());
                match next {
                    // "Leaf .." | "Leaf\EOM"
                    Some(Token::ProgramHeaderSeparator | Token::ProgramMessageUnitSeparator)
                    | None => {
                        // Consume the header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Execute handler
                        handler.event(device, context, Arguments::with(tokens))
                    }
                    // Branch?..
                    Some(Token::HeaderQuerySuffix) => {
                        // Consume query suffix
                        tokens.next();

                        // Consume header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Execute handler
                        let response_unit = response.response_unit()?;
                        handler.query(device, context, Arguments::with(tokens), response_unit)
                    }
                    // Tokenizer shouldn't emit anything else...
                    Some(_) => Err(ErrorCode::SyntaxError.into()),
                }
            }
            Node::Branch { sub, .. } => {
                //std::println!("Branch {}", std::str::from_utf8(name).unwrap());
                match next {
                    // Branch[:]<mnemonic>..
                    Some(Token::HeaderMnemonicSeparator | Token::ProgramMnemonic(..)) => {
                        // Consume seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::HeaderMnemonicSeparator)));

                        // Get mnemonic
                        let mnemonic = match tokens.next() {
                            Some(Ok(mnemonic @ Token::ProgramMnemonic(..))) => mnemonic,
                            Some(Err(err)) => return Err(err.into()),
                            _ => return Err(ErrorCode::CommandHeaderError.into()),
                        };

                        //std::println!("Branch:{mnemonic:?}");

                        *leaf = self;
                        for child in *sub {
                            if mnemonic.match_program_header(child.name()) {
                                return child.exec(leaf, device, context, tokens, response);
                            }
                        }
                        Err(ErrorCode::UndefinedHeader.into())
                    }
                    // Branch .. | Branch\EOM | Branch;
                    Some(Token::ProgramHeaderSeparator | Token::ProgramMessageUnitSeparator)
                    | None => {
                        // Consume header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Check if the first child is default and execute
                        match sub.first() {
                            Some(Node::Leaf {
                                default: true,
                                handler,
                                ..
                            }) => handler.event(device, context, Arguments::with(tokens)),
                            _ => Err(ErrorCode::CommandHeaderError.into()),
                        }
                    }
                    // Branch?..
                    Some(Token::HeaderQuerySuffix) => {
                        tokens.next();

                        // Consume header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Check if the first child is default and execute
                        match sub.first() {
                            Some(Node::Leaf {
                                default: true,
                                handler,
                                ..
                            }) => {
                                let response_unit = response.response_unit()?;
                                handler.query(
                                    device,
                                    context,
                                    Arguments::with(tokens),
                                    response_unit,
                                )
                            }
                            _ => Err(ErrorCode::CommandHeaderError.into()),
                        }
                    }
                    // Tokenizer shouldn't emit anything else...
                    Some(_) => Err(ErrorCode::SyntaxError.into()),
                }
            }
        }
    }
}

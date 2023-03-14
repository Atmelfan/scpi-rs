//! A SCPI command tree consisting of command nodes and handlers.
//!
//! Se chapter 6 of SCPI standard for guidelines for header and command tree guidelines.
//!
//! # Example
//! Take the following command tree.
//! ```text
//! *COM
//! :TODO
//! :TODO?
//! :BRANch
//!     :CHILd <args>
//!     [:DEFault?] <args>
//! ```
//! This would be written like below.
//! Note that a node handles both the query and event form of commands and it's up to the handler to reject a query on an event only node or vice-versa.
//! ```
//! # struct MyDevice;
//! # impl scpi::Device for MyDevice {
//! #     fn handle_error(&mut self, err: Error) {}
//! # }
//! use scpi::tree::{prelude::*, command::Todo};
//! const ROOT: Node<MyDevice> = Branch {
//!     name: b"",
//!     default: false,
//!     sub: &[
//!         Leaf {
//!             name: b"*COM",
//!             default: false,
//!             handler: &Todo,
//!         },
//!         Leaf {
//!             name: b"TODO",
//!             default: false,
//!             handler: &Todo,//Handles both TODO and TODO?
//!         },
//!         Branch {
//!             name: b"BRANch",
//!             default: false,
//!             sub: &[
//!                 // Default leaves must be first!
//!                 Leaf {
//!                     name: b"DEFault",
//!                     default: true,
//!                     handler: &Todo,
//!                 },
//!                 Leaf {
//!                     name: b"CHILd",
//!                     default: false,
//!                     handler: &Todo,
//!                 },
//!             ],
//!         },
//!     ],
//! };
//! ```

use core::iter::Peekable;
//extern crate std;

pub mod command;

use command::Command;

use crate::error::{Error, ErrorCode, Result};
use crate::parser::parameters::Parameters;
use crate::parser::response::Formatter;
use crate::parser::tokenizer::{Token, Tokenizer};
use crate::{Context, Device};

/// Everything needed when creating command trees or command handlers
pub mod prelude {
    pub use super::{
        command::{Command, CommandTypeMeta},
        Node::{self, Branch, Leaf},
    };
    pub use crate::{
        error::{Error, ErrorCode},
        parser::{
            format::*,
            parameters::Parameters,
            response::{Formatter, ResponseData, ResponseUnit},
            tokenizer::{Token, Tokenizer},
        },
        Context, Device,
    };
}

/// A SCPI command node
/// These nodes are structured as a command tree where each node represent a SCPI header mnemonic.
///
pub enum Node<'a, D> {
    /// A leaf node which can be called or queried.
    Leaf {
        /// Mnemonic of this leaf
        name: &'static [u8],
        /// Default node, will be executed if the branch immediately below is executed.
        /// Only one default node is allowed in each branch.
        default: bool,
        /// Command handler
        handler: &'a dyn Command<D>,
    },
    /// A branch which contains one or more leaves.
    Branch {
        /// Mnemonic of this branch
        name: &'static [u8],
        /// Default node.
        default: bool,
        /// Child nodes
        /// **Note:** Default node must be first!
        sub: &'a [Node<'a, D>],
    },
}

impl<'a, D> Node<'a, D> {
    /// Create a leaf node
    ///
    /// Alternatively use [crate::Leaf!]
    pub const fn leaf(name: &'static [u8], handler: &'a dyn Command<D>) -> Self {
        Self::Leaf {
            name,
            default: false,
            handler,
        }
    }

    /// Create a default leaf node
    ///
    /// Alternatively use [crate::Leaf!]
    pub const fn default_leaf(name: &'static [u8], handler: &'a dyn Command<D>) -> Self {
        Self::Leaf {
            name,
            default: true,
            handler,
        }
    }

    /// Create a branch node
    ///
    /// Alternatively use [crate::Branch!]
    pub const fn branch(name: &'static [u8], sub: &'a [Node<'a, D>]) -> Self {
        Self::Branch {
            name,
            default: false,
            sub,
        }
    }

    /// Create a default branch node
    ///
    /// Alternatively use [crate::Branch!]
    pub const fn default_branch(name: &'static [u8], sub: &'a [Node<'a, D>]) -> Self {
        Self::Branch {
            name,
            default: true,
            sub,
        }
    }

    /// Create a root node
    ///
    /// Alternatively use [crate::Root!]
    pub const fn root(sub: &'a [Node<'a, D>]) -> Self {
        Self::Branch {
            name: b"",
            default: false,
            sub,
        }
    }
}

/// A utility to create a [Node::Leaf].
#[macro_export]
macro_rules! Leaf {
    ($name:literal => $handler:expr) => {
        $crate::tree::Node::Leaf {
            name: $name,
            default: false,
            handler: $handler,
        }
    };
    (default $name:literal => $handler:expr) => {
        $crate::tree::Node::Leaf {
            name: $name,
            default: true,
            handler: $handler,
        }
    };
}

/// A utility to create a [Node::Branch].
#[macro_export]
macro_rules! Branch {
    ($name:literal; $($child:expr),+) => {
        $crate::tree::Node::Branch {
            name: $name,
            default: false,
            sub: &[
                $($child),+
            ],
        }
    };
    ($name:literal => $handler:expr; $($child:expr),+) => {
        $crate::tree::Node::Branch {
            name: $name,
            default: false,
            sub: &[
                Leaf!{default b"" => $handler },
                $($child),+
            ],
        }
    };
    (default $name:literal; $($child:expr),+) => {
        $crate::tree::Node::Branch {
            name: $name,
            default: true,
            sub: &[
                $($child),+
            ],
        }
    };
}

/// A utility to create the root [Node] of a command tree.
#[macro_export]
macro_rules! Root {
    ($($child:expr),+) => {
        $crate::tree::Node::Branch {
            name: b"",
            default: false,
            sub: &[
                $($child),+
            ],
        }
    };
}

impl<'a, D> Node<'a, D> {
    pub fn name(&self) -> &'static [u8] {
        match self {
            Self::Leaf { name, .. } => name,
            Self::Branch { name, .. } => name,
        }
    }
}

impl<'a, D> Node<'a, D>
where
    D: Device,
{
    /// Execute a command against a given device.
    ///
    /// # Arguments:
    /// * command - To be executed
    /// * device - To execute against
    /// * context - Context for this command
    /// * response - A formatter to write a response into.
    ///
    pub fn run<FMT>(
        &self,
        command: &[u8],
        device: &mut D,
        context: &mut Context,
        response: &mut FMT,
    ) -> Result<()>
    where
        FMT: Formatter,
    {
        let mut tokenizer = Tokenizer::new(command).peekable();
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
                        handler.event(device, context, Parameters::with(tokens))
                    }
                    // Branch?..
                    Some(Token::HeaderQuerySuffix) => {
                        // Consume query suffix
                        tokens.next();

                        // Consume header seperator
                        tokens.next_if(|t| matches!(t, Ok(Token::ProgramHeaderSeparator)));

                        // Execute handler
                        let response_unit = response.response_unit()?;
                        handler.query(device, context, Parameters::with(tokens), response_unit)
                    }
                    // This is a leaf node, cannot traverse further
                    Some(Token::HeaderMnemonicSeparator | Token::ProgramMnemonic(..)) => {
                        Err(ErrorCode::UndefinedHeader.into())
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
                        let mnemonic = match tokens.peek() {
                            Some(Ok(mnemonic @ Token::ProgramMnemonic(..))) => mnemonic,
                            Some(Err(err)) => return Err((*err).into()),
                            _ => return Err(ErrorCode::CommandHeaderError.into()),
                        };

                        //std::println!("Branch:{mnemonic:?}");

                        // Try to match a child with mnemonic
                        *leaf = self;
                        for child in *sub {
                            if mnemonic.match_program_header(child.name()) {
                                tokens.next(); // Consume mnemonic
                                return child.exec(leaf, device, context, tokens, response);
                            }
                        }

                        // Check if there's a default child branch
                        if let Some(child) = sub
                            .iter()
                            .find(|child| matches!(child, Node::Branch { default: true, .. }))
                        {
                            child.exec(leaf, device, context, tokens, response)
                        } else {
                            Err(ErrorCode::UndefinedHeader.into())
                        }
                    }
                    // Branch .. | Branch\EOM | Branch;
                    Some(
                        Token::ProgramHeaderSeparator
                        | Token::ProgramMessageUnitSeparator
                        | Token::HeaderQuerySuffix,
                    )
                    | None => {
                        // Try to find a default leaf or branch execute
                        if let Some(default_leaf) = sub
                            .iter()
                            .find(|child| matches!(child, Node::Leaf { default: true, .. }))
                        {
                            default_leaf.exec(leaf, device, context, tokens, response)
                        } else if let Some(default_branch) = sub
                            .iter()
                            .find(|child| matches!(child, Node::Branch { default: true, .. }))
                        {
                            default_branch.exec(leaf, device, context, tokens, response)
                        } else {
                            Err(ErrorCode::UndefinedHeader.into())
                        }
                    }
                    // Tokenizer shouldn't emit anything else...
                    Some(_) => Err(ErrorCode::SyntaxError.into()),
                }
            }
        }
    }
}

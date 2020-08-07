//! Used to build a SCPI command tree

use crate::command::Command;
use crate::error::{ErrorCode, Result};
use crate::response::Formatter;
use crate::tokenizer::Tokenizer;
use crate::Context;

#[macro_export]
macro_rules! scpi_tree {
    ($($node:expr),*) => {
    &Node{name: b"ROOT", optional: false, handler: None, sub: &[
        $(
            $node
        ),*
    ]}
    };
}

/// A SCPI command node
/// These nodes are structured as a command tree where each node represent a SCPI header mnemonic.
///
/// # Example
///
/// ```
/// use scpi::tree::Node;
/// use scpi::scpi_tree;
/// use scpi::ieee488::commands::*;
///
/// let root = scpi_tree![
///     Node{name: b"*IDN?", optional: false,  handler: Some(&IdnCommand{
///         manufacturer: b"GPA-Robotics",
///         model: b"Potato",
///         serial: b"42",
///         firmware: b"0"
///     }), sub: &[]}
///     //...
/// ];
/// ```
/// Note that all strings are ascii-/bytestrings, this is because only ASCII is defined in SCPI thus
/// the normal UTF8 &str in rust would be improper. To send a unicode string you can use Arbitrary Block Data
/// (or, this parser has an alternative arbitrary data header `#s"..."` which allows and checks UTF8 data inside the quotes.
///
pub struct Node<'a> {
    /// Mnemonic of this node, must follow the form SCPI notation (eg `LARGEsmall[<index>]` etc)
    pub name: &'static [u8],
    /// Command handler. If None, the parser will return a UndefinedHeader error if the node is called (may still be traversed)
    pub handler: Option<&'a dyn Command>,
    /// Subnodes. The node may contain None or an array of subcommands. If a message attempts to traverse
    /// this node and it does not have any subnodes (eg `IMhelping:THISnode:DONTexist), a UndefinedHeaderError will be returned.
    pub sub: &'a [Node<'a>],
    ///Marks the node as being optional (called default with inverse behaviour in IEE488)
    pub optional: bool,
}

impl<'a> Node<'a> {
    pub(crate) fn exec<FMT>(
        &self,
        context: &mut Context,
        args: &mut Tokenizer,
        response: &mut FMT,
        query: bool,
    ) -> Result<()>
    where
        FMT: Formatter,
    {
        if let Some(handler) = self.handler {
            //Execute self
            if query {
                handler.query(context, args, &mut response.response_unit()?)
            } else {
                handler.event(context, args)
            }
        } else if !self.sub.is_empty() {
            //No handler, check for a default child
            for child in self.sub {
                if child.optional {
                    return child.exec(context, args, response, query);
                }
            }
            //No optional child
            Err(ErrorCode::CommandHeaderError.into())
        } else {
            Err(ErrorCode::CommandHeaderError.into())
        }
    }
}

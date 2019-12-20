//! Used to build a SCPI command tree

use crate::command::Command;
use crate::tokenizer::Tokenizer;
use crate::error::Error;
use crate::ieee488::Context;
use crate::response::Formatter;

/// A SCPI command node
/// These nodes are structured as a command tree where each node represent a SCPI header mnemonic.
///
/// # Example
///
/// ```
/// use scpi::tree::Node;
/// use scpi::commands::IdnCommand;
///
/// let root = &Node{name: b"ROOT", optional: false, handler: None, sub: Some(&[
///     Node{name: b"*IDN?", optional: false,  handler: Some(&IdnCommand{
///            manufacturer: b"GPA-Robotics",
///            model: b"Potato",
///            serial: b"42",
///            firmware: b"0"
///        }), sub: None},
///     //...
/// ])};
/// ```
/// Note that all strings are ascii-/bytestrings, this is because only ASCII is defined in SCPI thus
/// the normal UTF8 &str in rust would be improper. To send a unicode string you can use Arbitrary Block Data
/// (or, this parser has an alternative arbitrary data header `#s"..."` which allows and checks UTF8 data inside the quotes.
///
pub struct Node<'a> {
    /// Mnemonic of this node, must follow the form SCPI notation (eg `LARGEsmall[<index>]` etc)
    pub name: &'static [u8],
    /// Command handler
    /// If None, the parser will return a UndefinedHeader error if the node is called (may still be traversed)
    pub handler: Option<&'a dyn Command>,
    /// Subcommands
    /// The node may contain None or an array of subcommands. If a message attempts to traverse
    /// this node and it does not have any subnodes (eg `IMhelping:THISnode:DONTexist), a UndefinedHeaderError will be returned.
    pub sub: Option<&'a [Node<'a>]>,
    ///Marks the node as being optional
    pub optional: bool
}

impl<'a> Node<'a> {

    pub(crate) fn exec(&self, context: &mut Context, args: &mut Tokenizer, response: &mut dyn Formatter, query: bool) -> Result<(), Error>{
        if let Some(handler) = self.handler {
            //Execute self
            if query {
                response.unit_start()?;
                handler.query(context, args, response)?;
                response.unit_end()
            }else{
                handler.event(context, args)
            }
        }else if self.sub.is_some() {
            //No handler, check for a default child
            for child in self.sub.unwrap() {
                if child.optional {
                    return child.exec(context, args, response, query);
                }
            }
            //No optional child
            Err(Error::CommandHeaderError)
        }else{
            Err(Error::CommandHeaderError)
        }
    }
}
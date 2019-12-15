//! Command trait and helper functions.
//!
//!

use crate::error::Error;
use crate::tokenizer::Tokenizer;
use crate::ieee488::Context;

/// This trait implements a command with optional event/query operations.
///
///
/// # Example
///
/// ```
/// use scpi::command::Command;
/// use scpi::Context;
/// use scpi::error::Error;
/// use scpi::tokenizer::Tokenizer;
///
/// struct MyCommand {
///    //...
/// }
///
/// // Implement Command for MyCommand
/// impl Command for MyCommand {
///     fn event(&self,context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
///         //Read a optional argument x
///         if let Some(x) = args.next_data(true)? {
///             // Non-optional argument y if x is present
///             let y = args.next_data(false)?.unwrap();
///
///             // Do stuff with x and y...
///         }
///
///         // Do stuff without x or y...
///
///         //I'm good thank you
///         Ok(())
///     }
///
///     fn query(&self,context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
///         Err(Error::UndefinedHeader)//Query not allowed
///     }
///
/// }
///
/// ```
///
pub trait Command {

    /// Called when the event form is used
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error>;

    ///Called when the query form is used
    fn query(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error>;
}
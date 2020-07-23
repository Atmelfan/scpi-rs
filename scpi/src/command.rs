//! Command trait and helper functions.
//!
//!

use crate::error::Result;
use crate::response::Formatter;
use crate::tokenizer::Tokenizer;
use crate::Context;

/// This trait implements a command with optional event/query operations.
///
///
/// # Example
///
/// ```rust
/// use scpi::prelude::*;
/// use scpi::error::Result;
///
/// struct MyCommand {
///    //...
/// }
///
/// // Implement Command for MyCommand
/// impl Command for MyCommand {
///     fn event(&self,context: &mut Context, args: &mut Tokenizer) -> Result<()> {
///         //Read a optional argument x
///         if let Some(x) = args.next_data(true)? {
///             // Non-optional argument y if x is present
///             let y = args.next_data(false)?.unwrap();
///
///             // Do stuff with x and y...
///         }else{
///             // Do stuff with neither x or y...
///         }
///
///         //I'm good thank you
///         Ok(())
///     }
///
///     fn query(&self,context: &mut Context, args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<()> {
///         Err(ErrorCode::UndefinedHeader.into())//Query not allowed
///     }
///
/// }
///
/// ```
///
pub trait Command {
    fn help(&self, _response: &mut dyn Formatter) {}

    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Unknown
    }

    /// Called when the event form is used
    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()>;

    ///Called when the query form is used
    fn query(
        &self,
        context: &mut Context,
        args: &mut Tokenizer,
        response: &mut dyn Formatter,
    ) -> Result<()>;
}

pub enum CommandTypeMeta {
    Unknown,
    NoQuery,
    QueryOnly,
    None,
}

/// Creates a stub for event()
///
#[macro_export]
macro_rules! qonly {
    () => {
        fn meta(&self) -> CommandTypeMeta {
            CommandTypeMeta::QueryOnly
        }

        fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
            Err(ErrorCode::UndefinedHeader.into())
        }
    };
}

/// Creates a stub for query()
///
#[macro_export]
macro_rules! nquery {
    () => {
        fn meta(&self) -> CommandTypeMeta {
            CommandTypeMeta::NoQuery
        }

        fn query(
            &self,
            _context: &mut Context,
            _args: &mut Tokenizer,
            _response: &mut dyn Formatter,
        ) -> Result<()> {
            Err(ErrorCode::UndefinedHeader.into())
        }
    };
}

//! Command trait and helper functions.
//!
//!

use crate::error::{Error, Result};
use crate::prelude::ErrorCode;
use crate::response::ResponseUnit;
use crate::tokenizer::Arguments;
use crate::{Context, Device};

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
///     fn query(&self,context: &mut Context, args: &mut Tokenizer, response: &mut ResponseUnit) -> Result<()> {
///         Err(ErrorCode::UndefinedHeader.into())//Query not allowed
///     }
///
/// }
///
/// ```
///
pub trait Command<D: Device> {
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Unknown
    }

    /// Called when the event form is used
    fn event(&self, _device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }

    ///Called when the query form is used
    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        _response: ResponseUnit,
    ) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }
}

pub struct Todo;
impl<D> Command<D> for Todo
where
    D: Device,
{
    fn event(&self, _device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
        todo!()
    }

    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        _response: ResponseUnit,
    ) -> Result<()> {
        todo!()
    }
}

/// Hint about the command forms
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CommandTypeMeta {
    // Not known
    Unknown,
    // Query not allowed
    NoQuery,
    /// Only query allowed
    QueryOnly,
    /// Both query and event are allowed
    Both,
}

/// Marks the command as query only
#[macro_export]
macro_rules! qonly {
    () => {
        fn meta(&self) -> CommandTypeMeta {
            CommandTypeMeta::QueryOnly
        }
    };
}

/// Marks the command as no query
#[macro_export]
macro_rules! nquery {
    () => {
        fn meta(&self) -> CommandTypeMeta {
            CommandTypeMeta::NoQuery
        }
    };
}

#[cfg(test)]
mod test_command {
    use crate::error::Result;
    use crate::prelude::*;
    use crate::tokenizer::Arguments;

    impl Device for () {
        fn handle_error(&mut self, _err: Error) {}
    }

    struct Query;
    impl Command<()> for Query {
        qonly!();

        fn query(
            &self,
            _device: &mut (),
            _context: &mut Context,
            _args: Arguments,
            _response: ResponseUnit,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_query() {
        assert_eq!(Query {}.meta(), CommandTypeMeta::QueryOnly);
    }

    struct Event;
    impl Command<()> for Event {
        nquery!();

        fn event(&self, device: &mut (), _context: &mut Context, _args: Arguments) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_event() {
        assert_eq!(Event {}.meta(), CommandTypeMeta::NoQuery);
    }

    struct Default;
    impl Command<()> for Default {
        fn event(&self, device: &mut (), _context: &mut Context, _args: Arguments) -> Result<()> {
            Ok(())
        }

        fn query(
            &self,
            device: &mut (),
            _context: &mut Context,
            _args: Arguments,
            _response: ResponseUnit,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_default() {
        assert_eq!(Default {}.meta(), CommandTypeMeta::Unknown);
    }
}

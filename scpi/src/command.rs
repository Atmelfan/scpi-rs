//! Command trait and helper functions.
//!
//!

use crate::error::Result;
use crate::response::ResponseUnit;
use crate::tokenizer::Tokenizer;
use crate::Context;

#[cfg(feature = "async")]
use async_trait::async_trait;

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
pub trait Command {
    fn help(&self, _response: &mut ResponseUnit) {}

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
        response: &mut ResponseUnit,
    ) -> Result<()>;
}

/// Async version of [Command]
#[cfg(feature = "async")]
#[async_trait]
trait AsyncCommand {
    async fn async_event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()>;

    async fn async_query(
        &self,
        context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncCommand for T where T: Command {
    async fn async_event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
        self.event(context, args)
    }

    async fn async_query(
        &self,
        context: &mut Context,
        args: &mut Tokenizer,
        response: &mut ResponseUnit,
    ) -> Result<()> {
        self.query(context, args, response)
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
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
            _response: &mut ResponseUnit,
        ) -> Result<()> {
            Err(ErrorCode::UndefinedHeader.into())
        }
    };
}

/// Stub command which defines no event or query
struct Stub;

impl Command for Stub {
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::None
    }
    
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }

    fn query(
        &self,
        _context: &mut Context,
        _args: &mut Tokenizer,
        _response: &mut ResponseUnit,
    ) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }
}

#[cfg(test)]
mod test_command {
    use crate::error::Result;
    use crate::prelude::*;

    struct Query {}
    impl Command for Query {
        qonly!();

        fn query(
            &self,
            _context: &mut Context,
            _args: &mut Tokenizer,
            _response: &mut ResponseUnit,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_query() {
        assert_eq!(Query {}.meta(), CommandTypeMeta::QueryOnly);
    }

    struct Event {}
    impl Command for Event {
        nquery!();

        fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_event() {
        assert_eq!(Event {}.meta(), CommandTypeMeta::NoQuery);
    }

    struct Default {}
    impl Command for Default {
        fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
            Ok(())
        }

        fn query(
            &self,
            _context: &mut Context,
            _args: &mut Tokenizer,
            _response: &mut ResponseUnit,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_default() {
        assert_eq!(Default {}.meta(), CommandTypeMeta::Unknown);
    }
}

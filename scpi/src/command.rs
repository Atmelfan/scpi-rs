//! Command trait and helper functions.
//!
//!

use crate::error::Result;
use crate::prelude::*;

/// This trait implements a command with optional event/query operations.
///
///
/// # Example
///
/// ```rust
/// //TODO
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
        fn meta(&self) -> $crate::prelude::CommandTypeMeta {
            $crate::prelude::CommandTypeMeta::QueryOnly
        }
    };
}

/// Marks the command as no query
#[macro_export]
macro_rules! nquery {
    () => {
        fn meta(&self) -> $crate::prelude::CommandTypeMeta {
            $crate::prelude::CommandTypeMeta::NoQuery
        }
    };
}

/// Marks the command as no query
#[macro_export]
macro_rules! both {
    () => {
        fn meta(&self) -> $crate::prelude::CommandTypeMeta {
            $crate::prelude::CommandTypeMeta::Both
        }
    };
}

#[cfg(test)]
mod test_command {
    use crate::error::Result;
    use crate::prelude::*;

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

        fn event(&self, _device: &mut (), _context: &mut Context, _args: Arguments) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_event() {
        assert_eq!(Event {}.meta(), CommandTypeMeta::NoQuery);
    }

    struct Default;
    impl Command<()> for Default {
        fn event(&self, _device: &mut (), _context: &mut Context, _args: Arguments) -> Result<()> {
            Ok(())
        }

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
    fn test_default() {
        assert_eq!(Default {}.meta(), CommandTypeMeta::Unknown);
    }
}

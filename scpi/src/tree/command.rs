//! Command trait and helper functions.
//!
//!

use crate::{
    error::{ErrorCode, Result},
    parser::{parameters::Arguments, response::ResponseUnit},
    Context, Device,
};

/// Marks the command as query only by creating a stub for [Command::meta].
///
/// ```
/// # use scpi::{qonly, tree::prelude::*};
/// # struct MyDevice;
/// # impl Device for MyDevice {
/// #     fn handle_error(&mut self, _: scpi::error::Error) { todo!() }
/// # }
///
/// struct MyQuery;
/// impl Command<MyDevice> for MyQuery {
///     qonly!();
///
///     fn query(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _args: Arguments,
///         _response: ResponseUnit,
///     ) -> Result<(), Error> {
///         // Do stuff
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! cmd_qonly {
    () => {
        fn meta(&self) -> $crate::tree::prelude::CommandTypeMeta {
            $crate::tree::prelude::CommandTypeMeta::QueryOnly
        }
    };
}

/// Marks the command as not queryable by creating a stub for [Command::meta].
///
/// ```
/// # use scpi::{nquery, tree::prelude::*};
/// # struct MyDevice;
/// # impl Device for MyDevice {
/// #     fn handle_error(&mut self, _: scpi::error::Error) { todo!() }
/// # }
///
/// struct MyQuery;
/// impl Command<MyDevice> for MyQuery {
///     nquery!();
///
///     fn event(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _args: Arguments,
///     ) -> Result<(), Error> {
///         // Do stuff
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! cmd_nquery {
    () => {
        fn meta(&self) -> $crate::tree::prelude::CommandTypeMeta {
            $crate::tree::prelude::CommandTypeMeta::NoQuery
        }
    };
}

/// Marks the command as both query and event by creating a stub for [Command::meta].
///
/// ```
/// # use scpi::{both, tree::prelude::*};
/// # struct MyDevice;
/// # impl Device for MyDevice {
/// #     fn handle_error(&mut self, _: scpi::error::Error) { todo!() }
/// # }
///
/// struct MyQuery;
/// impl Command<MyDevice> for MyQuery {
///     both!();
///
///     fn query(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _args: Arguments,
///         _response: ResponseUnit,
///     ) -> Result<(), Error> {
///         // Do stuff
///         Ok(())
///     }
///
///     fn event(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _args: Arguments,
///     ) -> Result<(), Error> {
///         // Do stuff
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! cmd_both {
    () => {
        fn meta(&self) -> $crate::tree::prelude::CommandTypeMeta {
            $crate::tree::prelude::CommandTypeMeta::Both
        }
    };
}

/// This trait implements a command with event/query operations.
/// # Example
/// ```
/// use scpi::{error::Result, tree::prelude::*, both};
///
/// struct MyCommand;
/// impl<D> Command<D> for MyCommand
/// where
///     D: Device,// or MyDevice or Device + AdditionalTrait or ...
/// {
///     // Create a stub for Command::meta()
///     cmd_both!();
///
///     // `HELLo:WORLd`
///     fn event(&self, _device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
///         //  Do stuff
///         println!("Hello world");
///         Ok(())
///     }
///
///     // `HELLo:WORLd?`
///     fn query(
///         &self,
///         _device: &mut D,
///         _context: &mut Context,
///         _args: Arguments,
///         mut response: ResponseUnit,
///     ) -> Result<()> {
///         response.data(&b"Hello world"[..]).finish()
///     }
/// }
/// ```
///
/// The default stubs for [Command::event] and [Command::query] returns an [ErrorCode::UndefinedHeader] error.
pub trait Command<D: Device> {
    /// Hint about the allowed forms this command allows.
    ///
    /// Not actually binding in any way but can be used to provide autocompletion and help info.
    /// Use [cmd_nquery!], [cmd_qonly!], or [cmd_both!] to automatically create the corresponding stub.
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Unknown
    }

    /// Called when the event form `COMmand` is used.
    fn event(&self, _device: &mut D, _context: &mut Context, _args: Arguments) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }

    ///Called when the query form `COMmand?` is used
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

/// Dummy node which calls [todo!] on event and query.
///
/// Indicates an unfinished command similar to the [todo!] macro.
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

/// Hint about the command forms.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CommandTypeMeta {
    /// Not known
    Unknown,
    /// Query not allowed
    NoQuery,
    /// Only query allowed
    QueryOnly,
    /// Both query and event are allowed
    Both,
}

#[cfg(test)]
mod test_command {
    use crate::tests::fixture_device;

    use super::*;

    struct TestCommandDevice;
    fixture_device!(TestCommandDevice);

    struct Query;
    impl Command<TestCommandDevice> for Query {
        cmd_qonly!();

        fn query(
            &self,
            _device: &mut TestCommandDevice,
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
    impl Command<TestCommandDevice> for Event {
        cmd_nquery!();

        fn event(
            &self,
            _device: &mut TestCommandDevice,
            _context: &mut Context,
            _args: Arguments,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_event() {
        assert_eq!(Event {}.meta(), CommandTypeMeta::NoQuery);
    }

    struct Default;
    impl Command<TestCommandDevice> for Default {
        fn event(
            &self,
            _device: &mut TestCommandDevice,
            _context: &mut Context,
            _args: Arguments,
        ) -> Result<()> {
            Ok(())
        }

        fn query(
            &self,
            _device: &mut TestCommandDevice,
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

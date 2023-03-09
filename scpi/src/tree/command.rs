//! Command trait and helper functions.
//!
//!

use crate::{
    error::{ErrorCode, Result},
    parser::{parameters::Parameters, response::ResponseUnit},
    Context, Device,
};

/// Marks the command as query only by creating a stub for [Command::meta].
///
/// ```
/// # use scpi::{cmd_qonly, tree::prelude::*};
/// # struct MyDevice;
/// # impl Device for MyDevice {
/// #     fn handle_error(&mut self, _: scpi::error::Error) { todo!() }
/// # }
///
/// struct MyQuery;
/// impl Command<MyDevice> for MyQuery {
///     cmd_qonly!();
///
///     fn query(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _params: Parameters,
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
/// # use scpi::{cmd_nquery, tree::prelude::*};
/// # struct MyDevice;
/// # impl Device for MyDevice {
/// #     fn handle_error(&mut self, _: scpi::error::Error) { todo!() }
/// # }
///
/// struct MyQuery;
/// impl Command<MyDevice> for MyQuery {
///     cmd_nquery!();
///
///     fn event(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _params: Parameters,
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
/// # use scpi::{cmd_both, tree::prelude::*};
/// # struct MyDevice;
/// # impl Device for MyDevice {
/// #     fn handle_error(&mut self, _: scpi::error::Error) { todo!() }
/// # }
///
/// struct MyQuery;
/// impl Command<MyDevice> for MyQuery {
///     cmd_both!();
///
///     fn query(
///         &self,
///         _device: &mut MyDevice,
///         _context: &mut Context,
///         _params: Parameters,
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
///         _params: Parameters,
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
/// use scpi::{error::Result, tree::prelude::*, cmd_both};
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
///     fn event(&self, _device: &mut D, _context: &mut Context, _params: Parameters) -> Result<()> {
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
///         _params: Parameters,
///         mut response: ResponseUnit,
///     ) -> Result<()> {
///         response.data(&b"Hello world"[..]).finish()
///     }
/// }
/// ```
///
/// The default stubs for [Command::event] and [Command::query] returns an [ErrorCode::UndefinedHeader] error.
///
/// ## Naming convention
/// A command impl should be named in PascalCase after the shortform header mnemonics upp to the last which should be in longform.
/// Common commands should be named as-is without '*' obv.
///
/// Examples:
/// * `INITiate[:IMMediate][:ALL]` => `InitImmAllCommand`
/// * `SYSTem:ERRor[:NEXT]` => `SystErrNextCommand`
/// * `SENSe:VOLTage[:DC]:NPLCycles` => `SensVoltDcNPLCyclesCommand`
/// * `*TRG` => `TrgCommand`
///
/// Sometimes a command is re-used in multiple branches, in that case one might skip the changing branches in the name.
/// Generics may be used to specialize the command.
/// * `SENSe(:VOLTage|:CURRent|..)([:DC]|:AC):RESolution` => `SensResolutionCommand` or `SensResolutionCommand<Func>`
///
/// When instantaiting more than one command it is recommended to use a command constant/member or const generics, i.e. like this:
/// * `OUTPut[<n>]:ATTenuation` => `OutpAttenuationCommand<const N: usize = 1>`
/// * `ARM:SEQuence[<n1>]:LAYer[<n2>][:IMMediate]` => `ArmSeqLayImmediateCommand { sequence: n1, layer: n2 }`
///
/// All of these forms may also be combined for extra headache.
///
/// In the end readability overrules the above conventions if the name gets too verbose...
pub trait Command<D: Device> {
    /// Hint about the allowed forms this command allows.
    ///
    /// Not actually binding in any way but can be used to provide autocompletion and help info.
    /// Use [cmd_nquery!], [cmd_qonly!], or [cmd_both!] to automatically create the corresponding stub.
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Unknown
    }

    /// Called when the event form `COMmand` is used.
    ///
    /// Default behaviour returns a [ErrorCode::UndefinedHeader] error.
    fn event(&self, _device: &mut D, _context: &mut Context, _params: Parameters) -> Result<()> {
        Err(ErrorCode::UndefinedHeader.into())
    }

    ///Called when the query form `COMmand?` is used
    ///
    /// Default behaviour returns a [ErrorCode::UndefinedHeader] error.
    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _params: Parameters,
        _resp: ResponseUnit,
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
    fn event(&self, _device: &mut D, _context: &mut Context, _params: Parameters) -> Result<()> {
        todo!()
    }

    fn query(
        &self,
        _device: &mut D,
        _context: &mut Context,
        _params: Parameters,
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
            _params: Parameters,
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
            _params: Parameters,
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
            _params: Parameters,
        ) -> Result<()> {
            Ok(())
        }

        fn query(
            &self,
            _device: &mut TestCommandDevice,
            _context: &mut Context,
            _params: Parameters,
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

#![no_std]

use scpi::command::{Command, CommandTypeMeta};
use scpi::error::Error;
use scpi::{qonly, nquery};
use scpi::Context;
use scpi::tokenizer::Tokenizer;
use scpi::response::Formatter;
use core::cell::{RefCell, RefMut, Cell};
use core::borrow::BorrowMut;

pub trait Trigger {
    fn abort(&self) -> Result<(), Error>;

    fn initiate_immediate(&self) -> Result<(), Error>;
}

pub struct AbortCommand<'a, T: Trigger>{
    trigger: &'a RefCell<T>
}
impl<'a, T: Trigger> Command for AbortCommand<'a, T> { nquery!();
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        self.trigger.borrow_mut().abort()
    }
}

pub struct TrigImmCommand<'a, T: Trigger>{
    trigger: &'a RefCell<T>
}
impl<'a, T: Trigger> Command for TrigImmCommand<'a, T> { nquery!();
    fn event(&self, _context: &mut Context, _args: &mut Tokenizer) -> Result<(), Error> {
        self.trigger.borrow_mut().initiate_immediate()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Trigger, AbortCommand, TrigImmCommand};
    use scpi::error::{Error, ArrayErrorQueue};
    use scpi::tree::Node;
    use core::cell::RefCell;
    use scpi::response::ArrayVecFormatter;
    use scpi::{Context, Device};
    use scpi::tokenizer::Tokenizer;
    use core::borrow::Borrow;

    extern crate std;

    struct MyDevice;

    impl Device for MyDevice {
        fn cls(&mut self) -> Result<(), Error> {
            unimplemented!()
        }

        fn rst(&mut self) -> Result<(), Error> {
            unimplemented!()
        }
    }

    struct TestTrigger;
    impl Trigger for TestTrigger {
        fn abort(&self) -> Result<(), Error> {
            std::println!("ABORt");
            Ok(())
        }

        fn initiate_immediate(&self) -> Result<(), Error> {
            std::println!("INITiate:IMMediate");
            Ok(())
        }
    }

    #[test]
    fn test_trigger() {
        let mut my_device = MyDevice;
        let ref trigger = RefCell::new(TestTrigger);

        let abort_command = AbortCommand{trigger};
        let initiate_command = TrigImmCommand{trigger};

        let init = [
            Node{
                name: b"IMMediate",
                handler: Some(&initiate_command),
                optional: true,
                sub: None
            },

        ];

        let tree = [
            Node{
                name: b"ABORt",
                handler: Some(&abort_command),
                optional: false,
                sub: None
            },
            Node{
                name: b"INITiate",
                handler: None,
                optional: false,
                sub: Some(&init)
            }

        ];

        let root = Node{ name: b"ROOT", handler: None, optional: false, sub: Some(&tree)};

        let mut errors = ArrayErrorQueue::<[Error; 1]>::new();

        let mut context = Context::new(&mut my_device, &mut errors, &root);

        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();

        let mut tokenizer = Tokenizer::from_str(b"init");

        let result = context.exec(&mut tokenizer, &mut buf);

        if let Err(error) = result {
            assert_eq!(error as isize, 0);
        }
    }
}

/// # 3.1 CONFigure:\<function> \<parameters>[,\<source list>]
/// Sets up the instrument in order to perform the measurement specified by the function.
/// Individual functions are described in \<functions> later in this chapter. The parameters of this
/// command are described in the individual function descriptions. The execution of the
/// CONFigure command may affect the value of any other setting in the instrument. The state
/// of the instrument is not specified after the execution of this command, except that a
/// subsequent READ? QUERY operation would perform the specified function.
pub trait Measure {

    fn rst() -> Result<(), Error>;

    /// Called when the event form is used
    fn configure(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error>;

    ///Called when the query form is used
    fn configure_query(&self, context: &mut Context, args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error>;

    ///Called when the query form is used
    fn fetch_query(&self, context: &mut Context, args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error>;
}

pub struct ConfCommand<'a, M: Measure>{
    measure: &'a RefCell<M>
}

impl<'a, M: Measure> Command for ConfCommand<'a, M> {

    fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
        self.measure.borrow_mut().configure(context, args)
    }

    fn query(&self, context: &mut Context, args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error> {
        self.measure.borrow_mut().configure_query(context, args, response)
    }
}

pub struct FetcCommand<'a, M: Measure>{
    measure: &'a RefCell<M>
}

impl<'a, M: Measure> Command for FetcCommand<'a, M> { qonly!();

    fn query(&self, context: &mut Context, args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error> {
        self.measure.borrow_mut().fetch_query(context, args, response)
    }
}

pub struct ReadCommand<'a, T: Trigger, M: Measure>{
    trigger: &'a RefCell<T>,
    measure: &'a RefCell<M>
}

impl<'a, T: Trigger, M: Measure> Command for ReadCommand<'a, T, M> { qonly!();

    fn query(&self, context: &mut Context, args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error> {
        //ABORt
        self.trigger.borrow_mut().abort()?;
        //INITiate
        self.trigger.borrow_mut().initiate_immediate()?;
        //FETCh?
        self.measure.borrow_mut().fetch_query(context, args, response)
    }
}

pub struct MeasCommand<'a, T: Trigger, M: Measure>{
    trigger: &'a RefCell<T>,
    measure: &'a RefCell<M>
}

impl<'a, T: Trigger, M: Measure> Command for MeasCommand<'a, T, M> { qonly!();

    fn query(&self, context: &mut Context, args: &mut Tokenizer, response: &mut dyn Formatter) -> Result<(), Error> {
        //ABORt
        self.trigger.borrow_mut().abort()?;
        //CONFigure
        self.measure.borrow_mut().configure(context, args)?;
        //READ?
        self.trigger.borrow_mut().abort()?;
        self.trigger.borrow_mut().initiate_immediate()?;
        self.measure.borrow_mut().fetch_query(context, args, response)
    }
}


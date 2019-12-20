//!Contains SCPI modules and mandatory commands
//!
//!

/// Contains basic implementations of SCPI mandated and optional commands.
///
///
///
pub mod commands {
    use crate::command::{Command, CommandTypeMeta};
    use crate::ieee488::Context;
    use crate::tokenizer::Tokenizer;
    use crate::response::Formatter;
    use crate::error::Error;
    use crate::{nquery, qonly};

    ///## 21.8.8 \[NEXT\]?
    ///> `SYSTem:ERRor:NEXT?` queries the error/event queue for the next item and removes it
    ///> from the queue. The response returns the full queue item consisting of an integer and a string
    ///> as described in the introduction to the SYSTem:ERRor subsystem.
    pub struct SystErrNextCommand;

    impl Command for SystErrNextCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Always return first error (NoError if empty)
            response.error(context.errors.pop_front_error())
        }

    }

    ///## 21.8.6 COUNt?
    ///> `SYSTem:ERRor:COUNt?` queries the error/event queue for the number of unread items. As
    ///> errors and events may occur at any time, more items may be present in the queue at the time
    ///> it is actually read.
    ///>
    ///> Note: If the queue is empty, the response is 0.
    pub struct SystErrCounCommand;

    impl Command for SystErrCounCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Always return first error (NoError if empty)
            response.usize_data(context.errors.len())
        }

    }

    ///## 21.8.5.1 ALL?
    ///> `SYSTem:ERRor:ALL?` queries the error/event queue for all the unread items and
    ///> removes them from the queue. The response returns a comma separated list of only the
    ///> error/event code numbers in FIFO order. If the queue is empty, the response is 0.
    pub struct SystErrAllCommand;

    impl Command for SystErrAllCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Always return first error (NoError if empty)
            let first = context.errors.pop_front_error();
            response.error(first)?;
            loop {
                let err = context.errors.pop_front_error();
                if err == Error::NoError {
                    break;
                }
                response.separator()?;
                response.error(err)?;
            }
            Ok(())
        }
    }

    ///## 21.21 :VERSion?
    ///> `SYSTem:VERSion?` query returns an <NR2> formatted numeric value corresponding to the SCPI version
    ///> number for which the instrument complies. The response shall have the form YYYY.V where
    ///> the Ys represent the year-version (i.e. 1990) and the V represents an approved revision
    ///> number for that year. If no approved revisions are claimed, then this extension shall be 0.
    pub struct SystVersCommand{
        pub year: u16,
        pub rev: u8
    }

    impl Command for SystVersCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Return {year}.{rev}
            response.u16_data(self.year)?;
            response.ascii_data(b".")?;
            response.u8_data(self.rev)
        }

    }

    ///## 20.1.4 \[:EVENt\]?
    ///> `STATus:OPERation:EVENt?`
    ///> This query returns the contents of the event register associated with the status structure
    ///> defined in the command.
    ///> The response is (NR1 NUMERIC RESPONSE DATA) (range: 0 through 32767) unless
    ///> changed by the :FORMat:SREGister command.
    ///>
    ///> Note that reading the event register clears it.
    pub struct StatOperEvenCommand;

    impl Command for StatOperEvenCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Always return first error (NoError if empty)
            response.u16_data(context.device.oper_event())
        }

    }

    ///## 20.1.2 :CONDition?
    ///> `STATus:OPERation:CONDition?`
    ///> Returns the contents of the condition register associated with the status structure defined in
    ///> the command. Reading the condition register is nondestructive.
    pub struct StatOperCondCommand;

    impl Command for StatOperCondCommand { qonly!();

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Always return first error (NoError if empty)
            response.u16_data(context.device.oper_condition())
        }

    }

    ///## 20.1.3 :ENABle \<NRf\> | \<non-decimal numeric\>
    ///> `STATus:OPERation:ENABle`
    ///> Sets the enable mask which allows true conditions in the event register to be reported in the
    ///> summary bit. If a bit is 1 in the enable register and its associated event bit transitions to true,
    ///> a positive transition will occur in the associated summary bit.
    ///> The command accepts parameter values of either format in the range 0 through 65535
    ///> (decimal) without error.
    ///>
    ///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
    ///> Note that 32767 is the maximum value returned as the most-significant bit of the register
    ///> cannot be set true.
    pub struct StatOperEnabCommand;

    impl Command for StatOperEnabCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            Ok(())
        }

        fn query(&self, context: &mut Context, _args: &mut Tokenizer, response: & mut dyn Formatter) -> Result<(), Error> {
            //Always return first error (NoError if empty)
            response.u16_data(context.oper_enable & 0x7FFFu16)
        }

    }

    ///## 20.2 :PRESet
    ///> `STATus:PRESet`
    ///> The PRESet command is an event that configures the SCPI and device-dependent status data
    ///> structures such that device-dependent events are reported at a higher level through the
    ///> mandatory part of the status-reporting mechanism. Device-dependent events are summarized
    ///> in the mandatory structures. The mandatory structure is defined in part by IEEE 488.2;
    ///> SCPI-required structures compose the rest. The mandatory part of the status-reporting
    ///> mechanism provides a device-independent interface for determining the gross status of a
    ///> device.
    pub struct StatPresCommand;

    impl Command for StatPresCommand { nquery!();
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<(), Error> {
            context.ques_enable = 0u16;
            context.oper_enable = 0u16;
            Ok(())
        }

    }


}
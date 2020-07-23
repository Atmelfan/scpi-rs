//!Contains SCPI modules and mandatory commands
//!
//!

/// This struct contains a register with event/enable functionality
/// (used in OPERation/QUEStionable registers)
///
///
#[derive(PartialEq, Copy, Clone)]
pub struct EventRegister {
    pub condition: u16,
    pub event: u16,
    pub enable: u16,
    pub ntr_filter: u16,
    pub ptr_filter: u16,
}

/// Utility trait
pub trait BitFlags<T> {
    /// Return a bitmask with the relevant bits set and others cleared
    ///
    fn get_mask(self) -> T;
    /// Return the position/offset of the relevant bit(s)
    fn get_pos(self) -> T;
}

/// The OPERation status register contains conditions which are part of the instrument’s normal
/// operation.
pub enum OperationBits {
    /// The instrument is currently performing a calibration.
    Calibrating = 0,
    /// The instrument is waiting for signals it controls to stabilize
    /// enough to begin measurements.
    Settling = 1,
    /// The instrument is currently changing its range.
    Ranging = 2,
    /// A sweep is in progress.
    Sweeping = 3,
    /// The instrument is actively measuring.
    Measuring = 4,
    /// The instrument is in a “wait for trigger” state of the
    /// trigger model.
    WaitingForTrig = 5,
    /// The instrument is in a “wait for arm” state of the trigger
    /// model.
    WaitingForArm = 6,
    /// The instrument is currently performing a correction.
    Correcting = 7,
    /// Available to designer.
    Designer1 = 8,
    /// Available to designer.
    Designer2 = 9,
    /// Available to designer.
    Designer3 = 10,
    /// Available to designer.
    Designer4 = 11,
    /// Available to designer.
    Designer5 = 12,
    /// One of n multiple logical instruments is
    /// reporting OPERational status.
    InstrumentSummary = 13,
    /// A user-defined programming is currently in the run
    /// state.
    ProgramRunning = 14,
}

impl BitFlags<u16> for OperationBits {
    fn get_mask(self) -> u16 {
        1 << (self as u16)
    }

    fn get_pos(self) -> u16 {
        self as u16
    }
}

/// The QUEStionable status register set contains bits which give an indication of the quality of
/// various aspects of the signal.
pub enum QuestionableBits {
    /// Indicates that the data is currently being acquired or generated
    SummaryVoltage = 0,
    SummaryCurrent = 1,
    SummaryTime = 2,
    SummaryPower = 3,
    SummaryTemperature = 4,
    SummaryFrequency = 5,
    SummaryPhase = 6,
    SummaryModulation = 7,
    SummaryCalibration = 8,
    Designer1 = 9,
    Designer2 = 10,
    Designer3 = 11,
    Designer4 = 12,
    InstrumentSummary = 13,
    /// Bit 14 is defined as the Command Warning bit. This bit indicates a non-fatal warning that
    /// relates to the instrument’s interpretation of a command, query, or one or more parameters of
    /// a specific command or query. Setting this bit is a warning to the application that the resultant
    /// instrument state or action is probably what was expected but may deviate in some manner.
    ///
    /// For example, the Command Warning bit is set whenever a parameter in one of the
    /// Measurement Instruction commands or queries is ignored during execution. Such a
    /// parameter may be ignored because it cannot be specified by a particular instrument.
    CommandWarning = 14,
}

impl BitFlags<u16> for QuestionableBits {
    fn get_mask(self) -> u16 {
        1 << (self as u16)
    }

    fn get_pos(self) -> u16 {
        self as u16
    }
}

impl Default for EventRegister {
    fn default() -> Self {
        EventRegister {
            condition: 0,
            event: 0,
            enable: 0,
            ntr_filter: 0,
            ptr_filter: 0xffff,
        }
    }
}

impl EventRegister {
    /// Create a new event register
    pub fn new() -> Self {
        EventRegister::default()
    }

    /// Preset the registers to default values
    pub fn preset(&mut self) {
        self.enable = 0u16;
        self.condition = 0u16;
        self.ptr_filter = 0xffffu16;
        self.ntr_filter = 0u16;
    }

    pub fn clear_event(&mut self) {
        self.event = 0;
    }

    /// Return the enabled operation bits summary.
    /// Returns true if any enabled condition bit is set, false otherwise.
    ///
    pub fn get_summary(&self) -> bool {
        (self.condition & self.enable) & 0x7fffu16 != 0u16
    }

    /// Get the state of relevant bit in status register. Returns true if bit is set, false otherwise.
    pub fn get_condition_bit(&self, bit: OperationBits) -> bool {
        self.condition & (bit as u16) != 0
    }

    /// Update condition register and event register based on pos-/neg-transition filters
    pub fn set_condition(&mut self, condition: u16) {
        let transitions = self.condition ^ condition;
        // Record pos-/negative-transitions to event register
        self.event |=
            transitions & ((condition & self.ptr_filter) | (!condition & self.ntr_filter));
        //Save new condition
        self.condition = condition;
    }

    /// Set relevant bit in condition register
    pub fn set_condition_bits(&mut self, bitmask: u16) {
        self.set_condition(self.condition | bitmask)
    }

    /// Clear relevant bit in condition register
    pub fn clear_condition_bits(&mut self, bitmask: u16) {
        self.set_condition(self.condition & !bitmask)
    }
}

/// Contains basic implementations of SCPI mandated and optional commands.
///
///
///
pub mod commands {
    use crate::command::{Command, CommandTypeMeta};
    use crate::error::{ErrorCode, Result};
    use crate::response::Formatter;
    use crate::tokenizer::Tokenizer;
    use crate::Context;
    use crate::{nquery, qonly};
    use core::convert::TryInto;

    ///## 21.8.8 \[NEXT\]?
    ///> `SYSTem:ERRor:NEXT?` queries the error/event queue for the next item and removes it
    ///> from the queue. The response returns the full queue item consisting of an integer and a string
    ///> as described in the introduction to the SYSTem:ERRor subsystem.
    pub struct SystErrNextCommand;

    impl Command for SystErrNextCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
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

    impl Command for SystErrCounCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            //Always return first error (NoError if empty)
            response.usize_data(context.errors.len())
        }
    }

    ///## 21.8.5.1 ALL?
    ///> `SYSTem:ERRor:ALL?` queries the error/event queue for all the unread items and
    ///> removes them from the queue. The response returns a comma separated list of only the
    ///> error/event code numbers in FIFO order. If the queue is empty, the response is 0.
    pub struct SystErrAllCommand;

    impl Command for SystErrAllCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            //Always return first error (NoError if empty)
            let first = context.errors.pop_front_error();
            response.error(first)?;
            loop {
                let err = context.errors.pop_front_error();
                if err == ErrorCode::NoError {
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
    pub struct SystVersCommand {
        pub year: u16,
        pub rev: u8,
    }

    impl Command for SystVersCommand {
        qonly!();

        fn query(
            &self,
            _context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
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

    impl Command for StatOperEvenCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            //Always return first error (NoError if empty)
            response.u16_data(context.operation.event & 0x7FFFu16)?;
            context.operation.event = 0;
            Ok(())
        }
    }

    ///## 20.1.2 :CONDition?
    ///> `STATus:OPERation:CONDition?`
    ///> Returns the contents of the condition register associated with the status structure defined in
    ///> the command. Reading the condition register is nondestructive.
    pub struct StatOperCondCommand;

    impl Command for StatOperCondCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            //Always return first error (NoError if empty)
            response.u16_data(context.operation.condition & 0x7FFFu16)
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
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
            context.operation.enable = args.next_data(false)?.unwrap().try_into()?;
            Ok(())
        }

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            response.u16_data(context.operation.enable & 0x7FFFu16)
        }
    }

    ///# 20.1.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
    ///> `STATus:OPERation:NTRansition`
    ///> Sets the negative transition filter. Setting a bit in the negative transition filter shall cause a 1
    ///> to 0 transition in the corresponding bit of the associated condition register to cause a 1 to be
    ///> written in the associated bit of the corresponding event register.
    ///> The command accepts parameter values of either format in the range 0 through 65535
    ///> (decimal) without error.
    ///>
    ///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
    ///> Note that 32767 is the maximum value returned as the most-significant bit of the register
    ///> cannot be set true.
    pub struct StatOperNtrCommand;

    impl Command for StatOperNtrCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
            context.operation.ntr_filter = args.next_data(false)?.unwrap().try_into()?;
            Ok(())
        }

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            response.u16_data(context.operation.ntr_filter & 0x7FFFu16)
        }
    }

    ///# 20.1.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
    ///> STATus:OPERation:PTRansition
    ///> Sets the positive transition filter. Setting a bit in the positive transition filter shall cause a 0 to
    ///> transition in the corresponding bit of the associated condition register to cause a 1 to be
    ///> written in the associated bit of the corresponding event register.
    ///> The command accepts parameter values of either format in the range 0 through 65535
    ///> (decimal) without error.
    ///>
    ///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
    ///> Note that 32767 is the maximum value returned as the most-significant bit of the register
    ///> cannot be set true.
    pub struct StatOperPtrCommand;

    impl Command for StatOperPtrCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
            context.operation.ptr_filter = args.next_data(false)?.unwrap().try_into()?;
            Ok(())
        }

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            response.u16_data(context.operation.ptr_filter & 0x7FFFu16)
        }
    }

    ///# 20.3.4 \[:EVENt\]?
    ///> `STATus:QUEStionable:EVENt?`
    ///> Defined the same as STATus:OPERation:EVENt. See Section 20.1.4 for details.
    pub struct StatQuesEvenCommand;

    impl Command for StatQuesEvenCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            //Always return first error (NoError if empty)
            response.u16_data(context.questionable.event & 0x7FFFu16)?;
            context.operation.event = 0;
            Ok(())
        }
    }

    ///# 20.3.2 :CONDition?
    ///> `STATus:QUEStionable:CONDition?`
    ///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
    pub struct StatQuesCondCommand;

    impl Command for StatQuesCondCommand {
        qonly!();

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            //Always return first error (NoError if empty)
            response.u16_data(context.questionable.condition & 0x7FFFu16)
        }
    }

    ///# 20.3.3 :ENABle \<NRf\> | \<non-decimal numeric\>
    ///> `STATus:QUEStionable:ENABle`
    ///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
    pub struct StatQuesEnabCommand;

    impl Command for StatQuesEnabCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
            context.questionable.enable = args.next_data(false)?.unwrap().try_into()?;
            Ok(())
        }

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            response.u16_data(context.questionable.enable & 0x7FFFu16)
        }
    }

    ///# 20.3.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
    ///> `STATus:QUEStionable:NTRansition`
    ///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
    pub struct StatQuesNtrCommand;

    impl Command for StatQuesNtrCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
            context.operation.ntr_filter = args.next_data(false)?.unwrap().try_into()?;
            Ok(())
        }

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            response.u16_data(context.operation.ntr_filter & 0x7FFFu16)
        }
    }

    ///# 20.3.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
    ///> `STATus:QUEStionable:PTRansition`
    ///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
    pub struct StatQuesPtrCommand;

    impl Command for StatQuesPtrCommand {
        fn event(&self, context: &mut Context, args: &mut Tokenizer) -> Result<()> {
            context.operation.ptr_filter = args.next_data(false)?.unwrap().try_into()?;
            Ok(())
        }

        fn query(
            &self,
            context: &mut Context,
            _args: &mut Tokenizer,
            response: &mut dyn Formatter,
        ) -> Result<()> {
            response.u16_data(context.operation.ptr_filter & 0x7FFFu16)
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

    impl Command for StatPresCommand {
        nquery!();
        fn event(&self, context: &mut Context, _args: &mut Tokenizer) -> Result<()> {
            context.questionable.preset();
            context.operation.preset();
            Ok(())
        }
    }

    /// Create a `STATus:` tree branch
    #[macro_export]
    macro_rules! scpi_status {
        () => {
            Node {
                name: b"STATus",
                optional: false,
                handler: None,
                sub: Some(&[
                    Node {
                        name: b"OPERation",
                        optional: false,
                        handler: None,
                        sub: Some(&[
                            Node {
                                name: b"CONDition",
                                optional: false,
                                handler: Some(&StatOperCondCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"ENABle",
                                optional: false,
                                handler: Some(&StatOperEnabCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"EVENt",
                                optional: true,
                                handler: Some(&StatOperEvenCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"NTRansition",
                                optional: false,
                                handler: Some(&StatOperNtrCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"PTRansition",
                                optional: false,
                                handler: Some(&StatOperPtrCommand {}),
                                sub: None,
                            },
                        ]),
                    },
                    Node {
                        name: b"QUEStionable",
                        optional: false,
                        handler: None,
                        sub: Some(&[
                            Node {
                                name: b"CONDition",
                                optional: false,
                                handler: Some(&StatQuesCondCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"ENABle",
                                optional: false,
                                handler: Some(&StatQuesEnabCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"EVENt",
                                optional: true,
                                handler: Some(&StatQuesEvenCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"NTRansition",
                                optional: false,
                                handler: Some(&StatQuesNtrCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"PTRansition",
                                optional: false,
                                handler: Some(&StatQuesPtrCommand {}),
                                sub: None,
                            },
                        ]),
                    },
                    Node {
                        name: b"PRESet",
                        optional: false,
                        handler: Some(&StatPresCommand {}),
                        sub: None,
                    },
                ]),
            }
        };
    }

    /// Create a `SYSTem:` tree branch
    #[macro_export]
    macro_rules! scpi_system {
        () => {
            Node {
                name: b"SYSTem",
                optional: false,
                handler: None,
                sub: Some(&[
                    Node {
                        name: b"ERRor",
                        optional: false,
                        handler: None,
                        sub: Some(&[
                            Node {
                                name: b"ALL",
                                optional: false,
                                handler: Some(&SystErrAllCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"NEXT",
                                optional: true,
                                handler: Some(&SystErrNextCommand {}),
                                sub: None,
                            },
                            Node {
                                name: b"COUNt",
                                optional: false,
                                handler: Some(&SystErrCounCommand {}),
                                sub: None,
                            },
                        ]),
                    },
                    Node {
                        name: b"VERSion",
                        optional: false,
                        handler: Some(&SystVersCommand { year: 1999, rev: 0 }),
                        sub: None,
                    },
                ]),
            }
        };
    }
}

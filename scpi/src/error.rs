//! This module contains standard SCPI errors in the form of the Error enum.
//!
//! Each error variant has a corresponding error/event number as the enum discriminant.
//!

use arraydeque::{ArrayDeque, Saturating, Array};

/// The Error type contains error definitions detected by the parser or commands
///
///> # 21.8.2 Error/Event numbers
///> The system-defined error/event numbers are chosen on an enumerated (“1 of N”) basis. The
///> SCPI-defined error/event numbers and the <error/event_description> portions of the full
///> queue item are listed here. The first error/event described in each class (for example, -100,
///> -200, -300, -400) is a “generic” error. In selecting the proper Error/event number to report,
///> more specific error/event codes are preferred, and the generic error/event is used only if the
///> others are inappropriate.
///
///> Note the organization of the following tables. A “simple-minded” parser might implement
///> only the XX0 errors, and a smarter one might implement all of them. A “smart and friendly”
///> parser might use the instrument-dependent part of the error/event message string to point out
///> the offending part of the command.
///
///
///

pub struct ExtendedError {
    error: Error,
    msg: Option<&'static [u8]>
}

impl ExtendedError {
    pub fn new(error: Error) -> Self {
        ExtendedError {
            error,
            msg: None
        }
    }

    pub fn extended(error: Error, msg: &'static [u8]) -> Self {
        ExtendedError {
            error,
            msg: Some(msg)
        }
    }

    pub fn get_code(&self) -> i16 {
        self.error.get_code()
    }

    pub fn get_message(&self) -> &'static [u8] {
        self.error.get_message()
    }

    pub fn get_extended(&self) -> Option<&'static [u8]> {
        self.msg
    }
}

impl Into<ExtendedError> for Error {
    fn into(self) -> ExtendedError {
        ExtendedError::new(self)
    }
}

#[derive(Debug, PartialEq, Copy, Clone, ScpiError)]
pub enum Error {
    #[error(custom)]
    Custom(i16, &'static [u8]),
    ///# 28.8.3 No error [-99, 0]
    ///> This message indicates that the device has no errors or events to report.
    /// ---
    /// `0, "No error"`
    ///
    /// The queue is completely empty. Every error/event in the queue has been read or
    /// the queue was purposely cleared by power-on, *CLS, etc
    #[error(code=0,message=b"No error")]
    NoError,

    ///# 28.9.10 Command Errors [-199, -100]
    ///> An <error/event number> in the range `[ -199 , -100 ]` indicates that an IEEE 488.2 syntax
    ///> error has been detected by the instrument’s parser. The occurrence of any error in this class
    ///> shall cause the command error bit (bit 5) in the event status register (IEEE 488.2, section
    ///> 11.5.1) to be set. One of the following events has occurred:
    ///>  * An IEEE 488.2 syntax error has been detected by the parser. That is, controller-to-device message was received which is in violation of the IEEE 488.2 standard. Possible violations include a data element which violates the devicelistening formats or whose type is unacceptable to the device.
    ///>  * An unrecognized header was received. Unrecognized headers include incorrect device-specific headers and incorrect or unimplemented IEEE 488.2 common commands.
    ///>  * A Group Execute Trigger (GET) was entered into the input buffer inside of an IEEE 488.2 <PROGRAM MESSAGE>.
    ///> Events that generate command errors shall not generate execution errors, device-specific
    ///> errors, or query errors; see the other error definitions in this chapter.
    /// ---
    /// `-100, "Command error"`
    ///
    ///This is the generic syntax error for devices that cannot detect more specific
    ///errors. This code indicates only that a Command Error as defined in IEEE 488.2,
    ///11.5.1.1.4 has occurred.
    #[error(code=-100,message=b"Command error")]
    CommandError,
    /// `-101, "Invalid character"`
    ///
    /// A syntactic element contains a character which is invalid for that type; for
    /// example, a header containing an ampersand, SETUP&. This error might be used
    /// in place of errors -114, -121, -141, and perhaps some others.
    #[error(code=-101,message=b"Invalid character")]
    InvalidCharacter,
    /// `-102, "Syntax error"`
    ///
    /// An unrecognized command or data type was encountered; for example, a string
    /// was received when the device does not accept strings.
    #[error(code=-102,message=b"Syntax error")]
    SyntaxError,
    ///The parser was expecting a separator and encountered an illegal character; for
    ///example, the semicolon was omitted after a program message unit,
    ///*EMC 1:CH1:VOLTS 5
    #[error(code=-103,message=b"Invalid separator")]
    InvalidSeparator,
    ///The parser recognized a data element different than one allowed; for example,
    ///numeric or string data was expected but block data was encountered
    #[error(code=-104,message=b"Data type error")]
    DataTypeError,
    ///A Group Execute Trigger was received within a program message (see IEEE
    ///488.2, 7.7)
    #[error(code=-105,message=b"GET not allowed")]
    GetNotAllowed,

    #[error(code=-108,message=b"Parameter not allowed")]
    ParameterNotAllowed,
    #[error(code=-109,message=b"Missing parameter")]
    MissingParameter,
    #[error(code=-110,message=b"Command header error")]
    CommandHeaderError,
    #[error(code=-111,message=b"Header separator error")]
    HeaderSeparatorError,
    #[error(code=-112,message=b"Program mnemonic too long")]
    ProgramMnemonicTooLong,
    #[error(code=-113,message=b"Undefined header")]
    UndefinedHeader,
    #[error(code=-114,message=b"Header suffix out of range")]
    HeaderSuffixOutOfRange,
    #[error(code=-115,message=b"Unexpected number of parameters")]
    UnexpectedNumberOfParameters,
    #[error(code=-120,message=b"Numeric data error")]
    NumericDataError,
    #[error(code=-121,message=b"Invalid character in number")]
    InvalidCharacterInNumber,
    #[error(code=-123,message=b"Exponent too large")]
    ExponentTooLarge,
    #[error(code=-124,message=b"Too many digits")]
    TooManyDigits,
    #[error(code=-128,message=b"Numeric data not allowed")]
    NumericDataNotAllowed,
    #[error(code=-130,message=b"Suffix error")]
    SuffixError,
    #[error(code=-131,message=b"Invalid suffix")]
    InvalidSuffix,
    #[error(code=-134,message=b"Suffix too long")]
    SuffixTooLong,
    #[error(code=-138,message=b"Suffix not allowed")]
    SuffixNotAllowed,
    #[error(code=-140,message=b"Character data error")]
    CharacterDataError,
    #[error(code=-141,message=b"Invalid character data")]
    InvalidCharacterData,
    #[error(code=-144,message=b"Character data too long")]
    CharacterDataTooLong,
    #[error(code=-148,message=b"Character data not allowed")]
    CharacterDataNotAllowed,
    #[error(code=-150,message=b"String data error")]
    StringDataError,
    #[error(code=-151,message=b"Invalid string data")]
    InvalidStringData,
    #[error(code=-158,message=b"String data not allowed")]
    StringDataNotAllowed,
    #[error(code=-160,message=b"Block data error")]
    BlockDataError,
    #[error(code=-161,message=b"Invalid block data")]
    InvalidBlockData,
    #[error(code=-168,message=b"Block data not allowed")]
    BlockDataNotAllowed,
    #[error(code=-170,message=b"Expression error")]
    ExpressionError,
    #[error(code=-171,message=b"Invalid expression")]
    InvalidExpression,
    #[error(code=-178,message=b"Expression data not allowed")]
    ExpressionDataNotAllowed,
    #[error(code=-180,message=b"Macro error")]
    MacroError,
    #[error(code=-181,message=b"Invalid outside macro definition")]
    InvalidOutsideMacroDefinition,
    #[error(code=-183,message=b"Invalid inside macro definition")]
    InvalidInsideMacroDefinition,
    #[error(code=-184,message=b"Macro parameter error")]
    MacroParameterError,

    ///# 21.8.10 Execution Errors [-299, -200]
    ///> An <error/event number> in the range `[ -299 , -200 ]` indicates that an error has been
    ///> detected by the instrument’s execution control block. The occurrence of any error in this
    ///> class shall cause the execution error bit (bit 4) in the event status register (IEEE 488.2,
    ///> section 11.5.1) to be set. One of the following events has occurred:
    ///>  * A <PROGRAM DATA> element following a header was evaluated by the device as outside of its legal input range or is otherwise inconsistent with the device’s capabilities.
    ///>  * A valid program message could not be properly executed due to some device condition.
    ///> Execution errors shall be reported by the device after rounding and expression evaluation
    ///> operations have taken place. Rounding a numeric data element, for example, shall not be
    ///> reported as an execution error. Events that generate execution errors shall not generate
    ///> Command Errors, device-specific errors, or Query Errors; see the other error definitions in
    ///> this section.
    /// ---
    ///This is the generic syntax error for devices that cannot detect more specific
    ///errors. This code indicates only that an Execution Error as defined in IEEE 488.2,
    ///11.5.1.1.5 has occurred.
    #[error(code=-200,message=b"Execution error")]
    ExecutionError,
    #[error(code=-201,message=b"Invalid while in local")]
    InvalidWhileInLocal,
    #[error(code=-202,message=b"Settings lost due to rtl")]
    SettingsLostDueToRTL,
    #[error(code=-203,message=b"Command protected")]
    CommandProtected,
    #[error(code=-210,message=b"Trigger error")]
    TriggerError,
    #[error(code=-211,message=b"Trigger ignored")]
    TriggerIgnored,
    #[error(code=-212,message=b"Arm ignored")]
    ArmIgnored,
    #[error(code=-213,message=b"Init ignored")]
    InitIgnored,
    #[error(code=-214,message=b"Trigger deadlock")]
    TriggerDeadlock,
    #[error(code=-215,message=b"Arm deadlock")]
    ArmDeadlock,
    #[error(code=-220,message=b"Parameter error")]
    ParameterError,
    #[error(code=-221,message=b"Settings conflict")]
    SettingsConflict,
    #[error(code=-222,message=b"Data out of range")]
    DataOutOfRange,
    #[error(code=-223,message=b"Too much data")]
    TooMuchData,
    #[error(code=-224,message=b"Illegal parameter value")]
    IllegalParameterValue,
    #[error(code=-225,message=b"Out of memory")]
    OutOfMemory,
    #[error(code=-226,message=b"Lists not same length")]
    ListsNotSameLength,
    #[error(code=-230,message=b"Data corrupt or stale")]
    DataCorruptOrStale,
    #[error(code=-231,message=b"Data questionable")]
    DataQuestionable,
    #[error(code=-232,message=b"Invalid format")]
    InvalidFormat,
    #[error(code=-233,message=b"Invalid version")]
    InvalidVersion,
    #[error(code=-240,message=b"Hardware error")]
    HardwareError,
    #[error(code=-241,message=b"Hardware missing")]
    HardwareMissing,
    #[error(code=-250,message=b"Mass storage error")]
    MassStorageError,
    #[error(code=-251,message=b"Missing mass storage")]
    MissingMassStorage,
    #[error(code=-252,message=b"Missing media")]
    MissingMedia,
    #[error(code=-253,message=b"Corrupt media")]
    CorruptMedia,
    #[error(code=-254,message=b"Media full")]
    MediaFull,
    #[error(code=-255,message=b"Directory full")]
    DirectoryFull,
    #[error(code=-256,message=b"Filename not found")]
    FileNameNotFound,
    #[error(code=-257,message=b"Filename error")]
    FileNameError,
    #[error(code=-258,message=b"Media protected")]
    MediaProtected,
    #[error(code=-260,message=b"Expression error")]
    ExecExpressionError,//Also declared in 170?
    #[error(code=-261,message=b"Math error in expression")]
    MathErrorInExpression,
    #[error(code=-270,message=b"Macro error")]
    ExecMacroError,
    #[error(code=-271,message=b"Macro syntax error")]
    MacroSyntaxError,
    #[error(code=-272,message=b"Macro execution error")]
    MacroExecutionError,
    #[error(code=-273,message=b"Illegal macro label")]
    IllegalMacroLabel,
    #[error(code=-274,message=b"Macro parameter error")]
    ExecMacroParameterError,
    #[error(code=-275,message=b"Macro definition too long")]
    MacroDefinitionTooLong,
    #[error(code=-276,message=b"Macro recursion error")]
    MacroRecursionError,
    #[error(code=-277,message=b"Macro redefinition not allowed")]
    MacroRedefinitionNotAllowed,
    #[error(code=-278,message=b"Macro header not found")]
    MacroHeaderNotFound,
    #[error(code=-280,message=b"Program error")]
    ProgramError,
    #[error(code=-281,message=b"Cannot create program")]
    CannotCreateProgram,
    #[error(code=-282,message=b"Illegal program name")]
    IllegalProgramName,
    #[error(code=-283,message=b"Illegal variable name")]
    IllegalVariableName,
    #[error(code=-284,message=b"Program currently running")]
    ProgramCurrentlyRunning,
    #[error(code=-285,message=b"Program syntax error")]
    ProgramSyntaxError,
    #[error(code=-286,message=b"Program runtime error")]
    ProgramRuntimeError,
    #[error(code=-290,message=b"Memory use error")]
    MemoryUseError,
    #[error(code=-291,message=b"Out of memory")]
    UseOutOfMemory,
    #[error(code=-292,message=b"Referenced name does not exist")]
    ReferencedNameDoesNotExist,
    #[error(code=-293,message=b"Referenced name already exists")]
    ReferencedNameAlreadyExists,
    #[error(code=-294,message=b"Incompatible type")]
    IncompatibleType,

    ///# Device-specific error `[-399, -300]`
    ///> An <error/event number> in the range `[ -399 , -300 ]` or `[ 1 , 32767 ]` indicates that the
    ///> instrument has detected an error which is not a command error, a query error, or an
    ///> execution error; some device operations did not properly complete, possibly due to an
    ///> abnormal hardware or firmware condition. These codes are also used for self-test response
    ///> errors. The occurrence of any error in this class should cause the device-specific error bit (bit
    ///> 3) in the event status register (IEEE 488.2, section 11.5.1) to be set. The meaning of positive
    ///> error codes is device-dependent and may be enumerated or bit mapped; the <error message>
    ///> string for positive error codes is not defined by SCPI and available to the device designer.
    ///> Note that the string is not optional; if the designer does not wish to implement a string for a
    ///> particular error, the null string should be sent (for example, 42,""). The occurrence of any
    ///> error in this class should cause the device-specific error bit (bit 3) in the event status register
    ///> (IEEE 488.2, section 11.5.1) to be set. Events that generate device-specific errors shall not
    ///> generate command errors, execution errors, or query errors; see the other error definitions in
    ///> this section.
    /// ---
    /// This is the generic device-dependent error for devices that cannot detect more
    /// specific errors. This code indicates only that a Device-Dependent Error as defined
    /// in IEEE 488.2, 11.5.1.1.6 has occurred.
    #[error(code=-300,message=b"Device-specific error")]
    DeviceSpecificError,
    #[error(code=-310,message=b"System error")]
    SystemError,
    #[error(code=-311,message=b"Memory error")]
    MemoryError,
    #[error(code=-312,message=b"PUD memory lost")]
    PudMemoryLost,
    #[error(code=-313,message=b"Calibration memory lost")]
    CalibrationMemoryLost,
    #[error(code=-314,message=b"Save/recall memory lost")]
    SaveRecallMemoryLost,
    #[error(code=-315,message=b"Configuration memory lost")]
    ConfigurationMemoryLost,
    #[error(code=-320,message=b"Storage fault")]
    StorageFault,
    #[error(code=-321,message=b"Out of memory")]
    StOutOfMemory,
    #[error(code=-330,message=b"Self-test failed")]
    SelfTestFailed,
    #[error(code=-340,message=b"Calibration failed")]
    CalibrationFailed,
    #[error(code=-350,message=b"Queue overflow")]
    QueueOverflow,
    #[error(code=-360,message=b"Communication error")]
    CommunicationError,
    #[error(code=-361,message=b"Parity error in program message")]
    ParityErrorInProgramMessage,
    #[error(code=-362,message=b"Framing error in program message")]
    FramingErrorInProgramMessage,
    #[error(code=-363,message=b"Input buffer overrun")]
    InputBufferOverrun,
    #[error(code=-365,message=b"Time out error")]
    TimeOutError,

    ///# Query error [-499, -400]
    ///> An <error/event number> in the range `[ -499 , -400 ]` indicates that the output queue control
    ///> of the instrument has detected a problem with the message exchange protocol described in
    ///> IEEE 488.2, chapter 6. The occurrence of any error in this class shall cause the query error
    ///> bit (bit 2) in the event status register (IEEE 488.2, section 11.5.1) to be set. These errors
    ///> correspond to message exchange protocol errors described in IEEE 488.2, section 6.5. One
    ///> of the following is true:
    ///>  * An attempt is being made to read data from the output queue when no output is either present or pending;
    ///>  * Data in the output queue has been lost.
    ///> Events that generate query errors shall not generate command errors, execution errors, or
    ///> device-specific errors; see the other error definitions in this section.
    /// ---
    /// This is the generic query error for devices that cannot detect more specific errors.
    /// This code indicates only that a Query Error as defined in IEEE 488.2, 11.5.1.1.7
    /// and 6.3 has occurred.
    #[error(code=-400,message=b"Query error")]
    QueryError,
    #[error(code=-410,message=b"Query INTERRUPTED")]
    QueryInterrupted,
    #[error(code=-420,message=b"Query UNTERMINATED")]
    QueryUnterminated,
    #[error(code=-430,message=b"Query DEADLOCKED")]
    QueryDeadlocked,
    #[error(code=-440,message=b"Query UNTERMINATED after indefinite response")]
    QueryUnterminatedAfterIndefiniteResponse,

    ///# Power on event [-599, -500]
    ///> An <error/event number> in the range `[-599:-500]` is used when the instrument wishes to
    ///> report a 488.2 power on event to the event/error queue. This event occurs when the
    ///> instrument detects an off to on transition in its power supply. This event also sets the power
    ///> on bit, (bit 7) of the Standard Event Status Register. See IEEE 488.2, section 11.5.1.
    /// ---
    /// The instrument has detected an off to on transition in its power supply.
    #[error(code=-500,message=b"Power on")]
    PowerOn,

    ///# User request event [-699, -600]
    ///> An <error/event number> in the range `[-699:-600]` is used when the instrument wishes to
    ///> report a 488.2 user request event. This event occurs when the instrument detects the
    ///> activation of a user request local control. This event also sets the user request bit (bit 6) of
    ///> the Standard Event Status Register. See IEEE 488.2, section 11.5.1.
    /// ---
    /// The instrument has detected the activation of a user request local control
    #[error(code=-600,message=b"User request")]
    UserRequest,

    ///# Request control event [-799, -700]
    ///> An <error/event number> in the range `[-799:-700]` is used when the instrument wishes to
    ///> report a 488.2 request control event to the error/event queue. This event occurs when the
    ///> instrument requests to become the active IEEE 488.1 controller-in-charge. This event also
    ///> sets request control bit (bit 1) of the Standard Event Status Register. See IEEE 488.2,
    ///> section 11.5.1.
    /// ---
    /// The instrument requested to become the active IEEE 488.1 controller-in-charge
    #[error(code=-700,message=b"Request control")]
    RequestControl,

    ///# Operation complete event [-899, -800]
    ///> An <error/event number> in the range `[-899:-800]` is used when the instrument wishes to
    ///> report a 488.2 operation complete event to the error/event queue. This event occurs when
    ///> instrument’s synchronization protocol, having been enabled by an *OPC command,
    ///> completes all selected pending operations. This protocol is described in IEEE 488.2, section
    ///> 12.5.2. This event also sets the operation complete bit (bit 0) of the Standard Event Status
    ///> Register. See IEEE 488.2, section 11.5.1. Note: *OPC? does not set bit 0 nor does it enter
    ///> any event in the error/event queue
    /// ---
    /// The instrument has completed all selected pending operations in accordance with
    /// the IEEE 488.2, 12.5.2 synchronization protocol
    #[error(code=-800,message=b"Operation complete")]
    OperationComplete,
}

impl<'a> Error {
    /**
     * Returns a bitmask for the appropriate bit in the ESR for this event/error.
     */
    pub fn esr_mask(self) -> u8 {
        match self.get_code() {
            -99..=0 => 0u8,//No bit
            -199..=-100 => 0x20u8,//bit 5
            -299..=-200 => 0x10u8,//bit 4
            -399..=-300 => 0x08u8,//bit 3
            -499..=-400 => 0x04u8,//bit 2
            -599..=-500 => 0x80u8,//bit 7
            -699..=-600 => 0x40u8,//bit 6
            -799..=-700 => 0x02u8,//bit 1
            -899..=-800 => 0x01u8,//bit 0
            _ => 0x08u8,//bit 3
        }
    }

}

pub trait ErrorQueue {
    fn push_back_error(&mut self, err: Error);

    fn pop_front_error(&mut self) -> Error;

    fn len(&self) -> usize;

    fn not_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct ArrayErrorQueue<T: Array<Item=Error>> {
    vec: ArrayDeque<T>,
}


impl<T: Array<Item=Error>> ArrayErrorQueue<T> {
    pub fn new() -> Self {
        ArrayErrorQueue {
            vec: ArrayDeque::<T, Saturating>::new()
        }
    }
}

impl<T: Array<Item=Error>> ErrorQueue for ArrayErrorQueue<T> {
    fn push_back_error(&mut self, err: Error) {
        //Try to queue an error, replace last with QueueOverflow if full
        if let Err(_) =  self.vec.push_back(err) {
            self.vec.pop_back();
            self.vec.push_back(Error::QueueOverflow).ok();
        }
    }

    fn pop_front_error(&mut self) -> Error {
        self.vec.pop_front().unwrap_or(Error::NoError)
    }

    fn len(&self) -> usize {
        self.vec.len()
    }
}
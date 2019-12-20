//! This module contains standard SCPI errors in the form of the Error enum.
//!
//! Each error variant has a corresponding error/event number as the enum discriminant.
//!

use arraydeque::{ArrayDeque, Saturating, Array, CapacityError};

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

#[derive(PartialEq, Copy, Clone, ScpiError)]
pub enum Error {
    ///# 28.8.3 No error [-99, 0]
    ///> This message indicates that the device has no errors or events to report.
    /// ---
    /// `0, "No error"`
    ///
    /// The queue is completely empty. Every error/event in the queue has been read or
    /// the queue was purposely cleared by power-on, *CLS, etc
    #[error(message=b"No error")]
    NoError = 0,

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
    #[error(message=b"Command error")]
    CommandError = -100,
    /// `-101, "Invalid character"`
    ///
    /// A syntactic element contains a character which is invalid for that type; for
    /// example, a header containing an ampersand, SETUP&. This error might be used
    /// in place of errors -114, -121, -141, and perhaps some others.
    #[error(message=b"Invalid character")]
    InvalidCharacter = -101,
    /// `-102, "Syntax error"`
    ///
    /// An unrecognized command or data type was encountered; for example, a string
    /// was received when the device does not accept strings.
    #[error(message=b"Syntax error")]
    SyntaxError = -102,
    ///The parser was expecting a separator and encountered an illegal character; for
    ///example, the semicolon was omitted after a program message unit,
    ///*EMC 1:CH1:VOLTS 5
    #[error(message=b"Invalid separator")]
    InvalidSeparator = -103,
    ///The parser recognized a data element different than one allowed; for example,
    ///numeric or string data was expected but block data was encountered
    #[error(message=b"Data type error")]
    DataTypeError = -104,
    ///A Group Execute Trigger was received within a program message (see IEEE
    ///488.2, 7.7)
    #[error(message=b"GET not allowed")]
    GetNotAllowed = -105,

    #[error(message=b"Parameter not allowed")]
    ParameterNotAllowed = -108,
    #[error(message=b"Missing parameter")]
    MissingParameter = -109,
    #[error(message=b"Command header error")]
    CommandHeaderError = -110,
    #[error(message=b"Header separator error")]
    HeaderSeparatorError = -111,
    #[error(message=b"Program mnemonic too long")]
    ProgramMnemonicTooLong = -112,
    #[error(message=b"Undefined header")]
    UndefinedHeader = -113,
    #[error(message=b"Header suffix out of range")]
    HeaderSuffixOutOfRange = -114,
    #[error(message=b"Unexpected number of parameters")]
    UnexpectedNumberOfParameters = -115,
    #[error(message=b"Numeric data error")]
    NumericDataError = -120,
    #[error(message=b"Invalid character in number")]
    InvalidCharacterInNumber = -121,
    #[error(message=b"Exponent too large")]
    ExponentTooLarge = -123,
    #[error(message=b"Too many digits")]
    TooManyDigits = -124,
    #[error(message=b"Numeric data not allowed")]
    NumericDataNotAllowed = -128,
    #[error(message=b"Suffix error")]
    SuffixError = -130,
    #[error(message=b"Invalid suffix")]
    InvalidSuffix = -131,
    #[error(message=b"Suffix too long")]
    SuffixTooLong = -134,
    #[error(message=b"Suffix not allowed")]
    SuffixNotAllowed = -138,
    #[error(message=b"Character data error")]
    CharacterDataError = -140,
    #[error(message=b"Invalid character data")]
    InvalidCharacterData = -141,
    #[error(message=b"Character data too long")]
    CharacterDataTooLong = -144,
    #[error(message=b"Character data not allowed")]
    CharacterDataNotAllowed = -148,
    #[error(message=b"String data error")]
    StringDataError = -150,
    #[error(message=b"Invalid string data")]
    InvalidStringData = -151,
    #[error(message=b"String data not allowed")]
    StringDataNotAllowed = -158,
    #[error(message=b"Block data error")]
    BlockDataError = -160,
    #[error(message=b"Invalid block data")]
    InvalidBlockData = -161,
    #[error(message=b"Block data not allowed")]
    BlockDataNotAllowed = -168,
    #[error(message=b"Expression error")]
    ExpressionError = -170,
    #[error(message=b"Invalid expression")]
    InvalidExpression = -171,
    #[error(message=b"Expression data not allowed")]
    ExpressionDataNotAllowed = -178,
    #[error(message=b"Macro error")]
    MacroError = -180,
    #[error(message=b"Invalid outside macro definition")]
    InvalidOutsideMacroDefinition = -181,
    #[error(message=b"Invalid inside macro definition")]
    InvalidInsideMacroDefinition = -183,
    #[error(message=b"Macro parameter error")]
    MacroParameterError = -184,

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
    #[error(message=b"Execution error")]
    ExecutionError = -200,
    #[error(message=b"Invalid while in local")]
    InvalidWhileInLocal = -201,
    #[error(message=b"Settings lost due to rtl")]
    SettingsLostDueToRTL = -202,
    #[error(message=b"Command protected")]
    CommandProtected = -203,
    #[error(message=b"Trigger error")]
    TriggerError = -210,
    #[error(message=b"Trigger ignored")]
    TriggerIgnored = -211,
    #[error(message=b"Arm ignored")]
    ArmIgnored = -212,
    #[error(message=b"Init ignored")]
    InitIgnored = -213,
    #[error(message=b"Trigger deadlock")]
    TriggerDeadlock = -214,
    #[error(message=b"Arm deadlock")]
    ArmDeadlock = -215,
    #[error(message=b"Parameter error")]
    ParameterError = -220,
    #[error(message=b"Settings conflict")]
    SettingsConflict = -221,
    #[error(message=b"Data out of range")]
    DataOutOfRange = -222,
    #[error(message=b"Too much data")]
    TooMuchData = -223,
    #[error(message=b"Illegal parameter value")]
    IllegalParameterValue = -224,
    #[error(message=b"Out of memory")]
    OutOfMemory = -225,
    #[error(message=b"Lists not same length")]
    ListsNotSameLength = -226,
    #[error(message=b"Data corrupt or stale")]
    DataCorruptOrStale = -230,
    #[error(message=b"Data questionable")]
    DataQuestionable = -231,
    #[error(message=b"Invalid format")]
    InvalidFormat = -232,
    #[error(message=b"Invalid version")]
    InvalidVersion = -233,
    #[error(message=b"Hardware error")]
    HardwareError = -240,
    #[error(message=b"Hardware missing")]
    HardwareMissing = -241,
    #[error(message=b"Mass storage error")]
    MassStorageError = -250,
    #[error(message=b"Missing mass storage")]
    MissingMassStorage = -251,
    #[error(message=b"Missing media")]
    MissingMedia = -252,
    #[error(message=b"Corrupt media")]
    CorruptMedia = -253,
    #[error(message=b"Media full")]
    MediaFull = -254,
    #[error(message=b"Directory full")]
    DirectoryFull = -255,
    #[error(message=b"Filename not found")]
    FileNameNotFound = -256,
    #[error(message=b"Filename error")]
    FileNameError = -257,
    #[error(message=b"Media protected")]
    MediaProtected = -258,
    #[error(message=b"Expression error")]
    ExecExpressionError = -260,//Also declared in 170?
    #[error(message=b"Math error in expression")]
    MathErrorInExpression = -261,
    #[error(message=b"Macro error")]
    ExecMacroError = -270,
    #[error(message=b"Macro syntax error")]
    MacroSyntaxError = -271,
    #[error(message=b"Macro execution error")]
    MacroExecutionError = -272,
    #[error(message=b"Illegal macro label")]
    IllegalMacroLabel = -273,
    #[error(message=b"Macro parameter error")]
    ExecMacroParameterError = -274,
    #[error(message=b"Macro definition too long")]
    MacroDefinitionTooLong = -275,
    #[error(message=b"Macro recursion error")]
    MacroRecursionError = -276,
    #[error(message=b"Macro redefinition not allowed")]
    MacroRedefinitionNotAllowed = -277,
    #[error(message=b"Macro header not found")]
    MacroHeaderNotFound = -278,
    #[error(message=b"Program error")]
    ProgramError = -280,
    #[error(message=b"Cannot create program")]
    CannotCreateProgram = -281,
    #[error(message=b"Illegal program name")]
    IllegalProgramName = -282,
    #[error(message=b"Illegal variable name")]
    IllegalVariableName = -283,
    #[error(message=b"Program currently running")]
    ProgramCurrentlyRunning = -284,
    #[error(message=b"Program syntax error")]
    ProgramSyntaxError = -285,
    #[error(message=b"Program runtime error")]
    ProgramRuntimeError = -286,
    #[error(message=b"Memory use error")]
    MemoryUseError = -290,
    #[error(message=b"Out of memory")]
    UseOutOfMemory = -291,
    #[error(message=b"Referenced name does not exist")]
    ReferencedNameDoesNotExist = -292,
    #[error(message=b"Referenced name already exists")]
    ReferencedNameAlreadyExists = -293,
    #[error(message=b"Incompatible type")]
    IncompatibleType = -294,

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
    #[error(message=b"Device-specific error")]
    DeviceSpecificError = -300,
    #[error(message=b"System error")]
    SystemError = -310,
    #[error(message=b"Memory error")]
    MemoryError = -311,
    #[error(message=b"PUD memory lost")]
    PudMemoryLost = -312,
    #[error(message=b"Calibration memory lost")]
    CalibrationMemoryLost = -313,
    #[error(message=b"Save/recall memory lost")]
    SaveRecallMemoryLost = -314,
    #[error(message=b"Configuration memory lost")]
    ConfigurationMemoryLost = -315,
    #[error(message=b"Storage fault")]
    StorageFault = -320,
    #[error(message=b"Out of memory")]
    StOutOfMemory = -321,
    #[error(message=b"Self-test failed")]
    SelfTestFailed = -330,
    #[error(message=b"Calibration failed")]
    CalibrationFailed = -340,
    #[error(message=b"Queue overflow")]
    QueueOverflow = -350,
    #[error(message=b"Communication error")]
    CommunicationError = -360,
    #[error(message=b"Parity error in program message")]
    ParityErrorInProgramMessage = -361,
    #[error(message=b"Framing error in program message")]
    FramingErrorInProgramMessage = -362,
    #[error(message=b"Input buffer overrun")]
    InputBufferOverrun = -363,
    #[error(message=b"Time out error")]
    TimeOutError = -365,

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
    #[error(message=b"Query error")]
    QueryError = -400,
    #[error(message=b"Query INTERRUPTED")]
    QueryInterrupted = -410,
    #[error(message=b"Query UNTERMINATED")]
    QueryUnterminated = -420,
    #[error(message=b"Query DEADLOCKED")]
    QueryDeadlocked = -430,
    #[error(message=b"Query UNTERMINATED after indefinite response")]
    QueryUnterminatedAfterIndefiniteResponse = -440,

    ///# Power on event [-599, -500]
    ///> An <error/event number> in the range `[-599:-500]` is used when the instrument wishes to
    ///> report a 488.2 power on event to the event/error queue. This event occurs when the
    ///> instrument detects an off to on transition in its power supply. This event also sets the power
    ///> on bit, (bit 7) of the Standard Event Status Register. See IEEE 488.2, section 11.5.1.
    /// ---
    /// The instrument has detected an off to on transition in its power supply.
    #[error(message=b"Power on")]
    PowerOn = -500,

    ///# User request event [-699, -600]
    ///> An <error/event number> in the range `[-699:-600]` is used when the instrument wishes to
    ///> report a 488.2 user request event. This event occurs when the instrument detects the
    ///> activation of a user request local control. This event also sets the user request bit (bit 6) of
    ///> the Standard Event Status Register. See IEEE 488.2, section 11.5.1.
    /// ---
    /// The instrument has detected the activation of a user request local control
    #[error(message=b"User request")]
    UserRequest = -600,

    ///# Request control event [-799, -700]
    ///> An <error/event number> in the range `[-799:-700]` is used when the instrument wishes to
    ///> report a 488.2 request control event to the error/event queue. This event occurs when the
    ///> instrument requests to become the active IEEE 488.1 controller-in-charge. This event also
    ///> sets request control bit (bit 1) of the Standard Event Status Register. See IEEE 488.2,
    ///> section 11.5.1.
    /// ---
    /// The instrument requested to become the active IEEE 488.1 controller-in-charge
    #[error(message=b"Request control")]
    RequestControl = -700,

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
    #[error(message=b"Operation complete")]
    OperationComplete = -800,
}

impl<'a> Error {
    /**
     * Returns a bitmask for the appropriate bit in the ESR for this event/error.
     */
    pub fn esr_mask(self) -> u8 {
        match self as i32 {
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
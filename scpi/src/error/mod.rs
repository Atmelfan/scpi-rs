//! This module contains standard SCPI errors in the form of the Error enum.
//!
//! Each error variant has a corresponding error/event number as the enum discriminant.
//!

mod arrayqueue;
use core::fmt::Display;

pub use arrayqueue::*;

/// A SCPI error
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Error(
    /// Error code
    ErrorCode,
    /// Optional additional error information
    Option<&'static [u8]>,
);

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = self.get_code();
        let msg = core::str::from_utf8(self.get_message()).unwrap_or("<invalid utf8>");

        if let Some(ext) = self.get_extended() {
            let ext = core::str::from_utf8(ext).unwrap_or("<invalid utf8>");
            write!(f, "{code},\"{msg};{ext}\"")
        } else {
            write!(f, "{code},\"{msg}\"")
        }
    }
}

impl Default for Error {
    fn default() -> Self {
        Self::new(ErrorCode::NoError)
    }
}

/// Useful alias of Result for SCPI operations
pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    /// Create new error with specified error code
    pub fn new(code: ErrorCode) -> Self {
        Self(code, None)
    }

    /// Create new error with a custom (unchecked) code
    pub fn custom(code: i16, desc: &'static [u8]) -> Self {
        Self(ErrorCode::Custom(code, desc), None)
    }

    /// Create new error with specified error code with an extended message
    pub fn extended(code: ErrorCode, msg: &'static [u8]) -> Self {
        Self(code, Some(msg))
    }

    /// Get numeric error code of error
    pub fn get_code(&self) -> i16 {
        self.0.get_code()
    }

    /// Get message of error
    pub fn get_message(&self) -> &'static [u8] {
        self.0.get_message()
    }

    /// Get extended message of error
    pub fn get_extended(&self) -> Option<&'static [u8]> {
        self.1
    }

    /**
     * Returns a bitmask for the appropriate bit in the ESR for this event/error.
     */
    pub fn esr_mask(&self) -> u8 {
        self.0.esr_mask()
    }
}

impl PartialEq<ErrorCode> for Error {
    fn eq(&self, other: &ErrorCode) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Error> for ErrorCode {
    fn eq(&self, other: &Error) -> bool {
        self == &other.0
    }
}

impl From<ErrorCode> for Error {
    fn from(err: ErrorCode) -> Self {
        Error::new(err)
    }
}

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
#[derive(Debug, PartialEq, Eq, Copy, Clone, ScpiError)]
pub enum ErrorCode {
    ///# Custom error
    /// Used for custom error in the range`[ -399 , -300 ]` or `[ 1 , 32767 ]`.
    /// See **Device-specific error** for closer description.
    ///
    #[error(custom)]
    Custom(i16, &'static [u8]),
    ///# 28.8.3 No error [-99, 0]
    ///> This message indicates that the device has no errors or events to report.
    /// ---
    /// `0, "No error"`
    ///
    /// The queue is completely empty. Every error/event in the queue has been read or
    /// the queue was purposely cleared by power-on, *CLS, etc
    #[error(code = 0, message = b"No error")]
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
    /// `-103, "Invalid Separator"`
    ///
    ///The parser was expecting a separator and encountered an illegal character; for
    ///example, the semicolon was omitted after a program message unit,
    ///*EMC 1:CH1:VOLTS 5
    #[error(code=-103,message=b"Invalid separator")]
    InvalidSeparator,
    /// `-104, "Data type error"`
    ///
    ///The parser recognized a data element different than one allowed; for example,
    ///numeric or string data was expected but block data was encountered
    #[error(code=-104,message=b"Data type error")]
    DataTypeError,
    /// `-105, "GET not allowed"`
    ///
    ///A Group Execute Trigger was received within a program message (see IEEE
    ///488.2, 7.7)
    #[error(code=-105,message=b"GET not allowed")]
    GetNotAllowed,
    /// `-108, "Parameter not allowed"`
    ///
    /// More parameters were received than expected for the header; for example, the
    /// *EMC common command only accepts one parameter, so receiving *EMC 0,1 is
    /// not allowed.
    #[error(code=-108,message=b"Parameter not allowed")]
    ParameterNotAllowed,
    /// `-109, "Missing parameter"`
    ///
    /// Fewer parameters were recieved than required for the header; for example, the
    /// *EMC common command requires one parameter, so receiving *EMC is not
    /// allowed.
    #[error(code=-109,message=b"Missing parameter")]
    MissingParameter,
    /// `-110, "Command header error"`
    ///
    /// An error was detected in the header. This error message should be used when the
    /// device cannot detect the more specific errors described for errors -111 through
    /// -119.
    #[error(code=-110,message=b"Command header error")]
    CommandHeaderError,
    /// `-111, "Header separator error"`
    ///
    /// A character which is not a legal header separator was encountered while parsing
    /// the header; for example, no white space followed the header, thus
    /// *GMC"MACRO" is an error.
    #[error(code=-111,message=b"Header separator error")]
    HeaderSeparatorError,
    /// `-112, "Program mnemonic too long"`
    ///
    /// The header contains more that twelve characters (see IEEE 488.2, 7.6.1.4.1).
    #[error(code=-112,message=b"Program mnemonic too long")]
    ProgramMnemonicTooLong,
    /// `-113, "Undefined header"`
    ///
    /// The header is syntactically correct, but it is undefined for this specific device; for
    /// example, *XYZ is not defined for any device.
    #[error(code=-113,message=b"Undefined header")]
    UndefinedHeader,
    /// `-114, "Header suffix out of range"`
    ///
    /// The value of a numeric suffix attached to a program mnemonic, see Syntax and
    /// Style section 6.2.5.2, makes the header invalid.
    #[error(code=-114,message=b"Header suffix out of range")]
    HeaderSuffixOutOfRange,
    /// `-115, "Unexpected number of parameters"`
    ///
    /// The number of parameters received does not correspond to the number of
    /// parameters expected. This is typically due an inconsistency with the number of
    /// instruments in the selected group (see section on INSTrument:DEFine:GROup).
    #[error(code=-115,message=b"Unexpected number of parameters")]
    UnexpectedNumberOfParameters,
    /// `-120, "Numeric data error"`
    ///
    /// This error, as well as errors -121 through -129, are generated when parsing a data
    /// element which apprears to be numeric, including the nondecimal numeric types.
    /// This particular error message should be used if the device cannot detect a more
    /// specific error.
    #[error(code=-120,message=b"Numeric data error")]
    NumericDataError,
    /// `-121, "Invalid character in number"`
    ///
    /// An invalid character for the data type being parsed was encountered; for
    /// example, an alpha in a decimal numeric or a “9" in octal data.
    #[error(code=-121,message=b"Invalid character in number")]
    InvalidCharacterInNumber,
    /// `-123, "Exponent too large"`
    ///
    /// The magnitude of the exponent was larger than 32000 (see IEEE 488.2,
    /// 7.7.2.4.1).
    #[error(code=-123,message=b"Exponent too large")]
    ExponentTooLarge,
    /// `-124, "Too many digits"`
    ///
    /// The mantissa of a decimal numeric data element contained more than 255 digits
    /// excluding leading zeros (see IEEE 488.2, 7.7.2.4.1).
    #[error(code=-124,message=b"Too many digits")]
    TooManyDigits,
    /// `-128, ""Numeric data not allowed`
    ///
    /// A legal numeric data element was received, but the device does not accept one in
    /// this position for the header.
    #[error(code=-128,message=b"Numeric data not allowed")]
    NumericDataNotAllowed,
    /// `-130, "Suffix error"`
    ///
    /// This error, as well as errors -131 through -139, are generated when parsing a
    /// suffix. This particular error message should be used if the device cannot detect a
    /// more specific error.
    #[error(code=-130,message=b"Suffix error")]
    SuffixError,
    /// `-131, "Invalid suffix"`
    ///
    /// The suffix does not follow the syntax described in IEEE 488.2, 7.7.3.2, or the
    /// suffix is inappropriate for this device.]
    #[error(code=-131,message=b"Invalid suffix")]
    InvalidSuffix,
    /// `-134, "Suffix too long"`
    ///
    /// The suffix contained more than 12 characters (see IEEE 488.2, 7.7.3.4).
    #[error(code=-134,message=b"Suffix too long")]
    SuffixTooLong,
    /// `-138, "Suffix not allowed"`
    ///
    /// A suffix was encountered after a numeric element which does not allow suffixes.
    #[error(code=-138,message=b"Suffix not allowed")]
    SuffixNotAllowed,
    /// `-140, "Character data error"`
    ///
    /// This error, as well as errors -141 through -149, are generated when parsing a
    /// character data element. This particular error message should be used if the device
    /// cannot detect a more specific error.
    #[error(code=-140,message=b"Character data error")]
    CharacterDataError,
    /// `-141, "Invalid character data"`
    ///
    /// Either the character data element contains an invalid character or the particular
    /// element received is not valid for the header.
    #[error(code=-141,message=b"Invalid character data")]
    InvalidCharacterData,
    /// `-144, "Character data too long"`
    ///
    /// The character data element contains more than twelve characters (see IEEE
    /// 488.2, 7.7.1.4).
    #[error(code=-144,message=b"Character data too long")]
    CharacterDataTooLong,
    /// `-148, "Character data not allowed"`
    ///
    /// A legal character data element was encountered where prohibited by the device.
    #[error(code=-148,message=b"Character data not allowed")]
    CharacterDataNotAllowed,
    /// `-150, "String data error"`
    ///
    /// This error, as well as errors -151 through -159, are generated when parsing a
    /// string data element. This particular error message should be used if the device
    /// cannot detect a more specific error.
    #[error(code=-150,message=b"String data error")]
    StringDataError,
    /// `-151, "Invalid string data"`
    ///
    /// A string data element was expected, but was invalid for some reason (see IEEE
    /// 488.2, 7.7.5.2); for example, an END message was received before the terminal
    /// quote character.
    #[error(code=-151,message=b"Invalid string data")]
    InvalidStringData,
    /// `-158, "String data not allowed"`
    ///
    /// A string data element was encountered but was not allowed by the device at this
    /// point in parsing.
    #[error(code=-158,message=b"String data not allowed")]
    StringDataNotAllowed,
    /// `-160, "Block data error"`
    ///
    /// This error, as well as errors -161 through -169, are generated when parsing a
    /// block data element. This particular error message should be used if the device
    /// cannot detect a more specific error.
    #[error(code=-160,message=b"Block data error")]
    BlockDataError,
    /// `-161, "Invalid block data"`
    ///
    /// A block data element was expected, but was invalid for some reason (see IEEE
    /// 488.2, 7.7.6.2); for example, an END message was received before the length was
    /// satisfied.
    #[error(code=-161,message=b"Invalid block data")]
    InvalidBlockData,
    /// `-168, "Block data not allowed"`
    ///
    /// A legal block data element was encountered but was not allowed by the device at
    /// this point in parsing.
    #[error(code=-168,message=b"Block data not allowed")]
    BlockDataNotAllowed,
    /// `-170, "Expression error"`
    ///
    /// This error, as well as errors -171 through -179, are generated when parsing an
    /// expression data element. This particular error message should be used if the
    /// device cannot detect a more specific error.
    #[error(code=-170,message=b"Expression error")]
    ExpressionError,
    /// `-171, "Invalid expression"`
    ///
    /// The expression data element was invalid (see IEEE 488.2, 7.7.7.2); for example,
    /// unmatched parentheses or an illegal character.
    #[error(code=-171,message=b"Invalid expression")]
    InvalidExpression,
    /// `-178, "Expression data not allowed"`
    ///
    /// A legal expression data was encountered but was not allowed by the device at
    /// this point in parsing.
    #[error(code=-178,message=b"Expression data not allowed")]
    ExpressionDataNotAllowed,
    /// `-180, "Macro error"`
    ///
    /// This error, as well as errors -181 through -189, are generated when defining a
    /// macro or executing a macro. This particular error message should be used if the
    /// device cannot detect a more specific error.
    #[error(code=-180,message=b"Macro error")]
    MacroError,
    /// `-181, "Invalid outside macro definition"`
    ///
    /// Indicates that a macro parameter placeholder ($<number) was encountered
    /// outside of a macro definition.
    #[error(code=-181,message=b"Invalid outside macro definition")]
    InvalidOutsideMacroDefinition,
    /// `-183, "Invalid inside macro definition"`
    ///
    /// Indicates that the program message unit sequence, sent with a *DDT or *DMC
    /// command, is syntactically invalid (see IEEE 488.2, 10.7.6.3).
    #[error(code=-183,message=b"Invalid inside macro definition")]
    InvalidInsideMacroDefinition,
    /// `-184, "Macro parameter error"`
    ///
    /// Indicates that a command inside the macro definition had the wrong number or
    /// type of parameters.
    #[error(code=-184,message=b"Macro parameter error")]
    MacroParameterError,

    ///# 21.8.10 Execution Errors [-299, -200]
    ///> An <error/event number> in the range `[ -299 , -200 ]` indicates that an error has been
    ///> detected by the instrument’s execution control block. The occurrence of any error in this
    ///> class shall cause the execution error bit (bit 4) in the event status register (IEEE 488.2,
    ///> section 11.5.1) to be set. One of the following events has occurred:
    ///>  * A \<PROGRAM DATA\> element following a header was evaluated by the device as outside of its legal input range or is otherwise inconsistent with the device’s capabilities.
    ///>  * A valid program message could not be properly executed due to some device condition.
    ///> Execution errors shall be reported by the device after rounding and expression evaluation
    ///> operations have taken place. Rounding a numeric data element, for example, shall not be
    ///> reported as an execution error. Events that generate execution errors shall not generate
    ///> Command Errors, device-specific errors, or Query Errors; see the other error definitions in
    ///> this section.
    /// ---
    /// `-200, "Execution error"`
    ///
    ///This is the generic syntax error for devices that cannot detect more specific
    ///errors. This code indicates only that an Execution Error as defined in IEEE 488.2,
    ///11.5.1.1.5 has occurred.
    #[error(code=-200,message=b"Execution error")]
    ExecutionError,
    /// `-201, "Invalid while in local"`
    ///
    /// Indicates that a command is not executable while the device is in local due to a
    /// hard local control (see IEEE 488.2, 5.6.1.5); for example, a device with a rotary
    /// switch receives a message which would change the switches state, but the device
    /// is in local so the message can not be executed.
    #[error(code=-201,message=b"Invalid while in local")]
    InvalidWhileInLocal,
    /// `-202, "Settings lost due to rtl"`
    ///
    /// Indicates that a setting associated with a hard local control (see IEEE 488.2,
    /// 5.6.1.5) was lost when the device changed to LOCS from REMS or to LWLS
    /// from RWLS.
    #[error(code=-202,message=b"Settings lost due to rtl")]
    SettingsLostDueToRtl,
    /// `-203, "Command protected"`
    ///
    /// Indicates that a legal password-protected program command or query could not
    /// be executed because the command was disabled.
    #[error(code=-203,message=b"Command protected")]
    CommandProtected,
    /// `-210, "Trigger error"`
    ///
    /// Indicates a trigger error.
    #[error(code=-210,message=b"Trigger error")]
    TriggerError,
    /// `-211, "Trigger ignored"`
    ///
    /// Indicates that a GET, *TRG, or triggering signal was received and recognized by
    /// the device but was ignored because of device timing considerations; for example,
    /// the device was not ready to respond. Note: a DT0 device always ignores GET and
    /// treats *TRG as a Command Error.
    #[error(code=-211,message=b"Trigger ignored")]
    TriggerIgnored,
    /// `-212, "Arm ignored"`
    ///
    /// Indicates that an arming signal was received and recognized by the device but
    /// was ignored.
    #[error(code=-212,message=b"Arm ignored")]
    ArmIgnored,
    /// `-213, "Init ignored"`
    ///
    /// Indicates that a request for a measurement initiation was ignored as another
    /// measurement was already in progress.
    #[error(code=-213,message=b"Init ignored")]
    InitIgnored,
    /// `-214, "Trigger deadlock"`
    ///
    /// Indicates that the trigger source for the initiation of a measurement is set to GET
    /// and subsequent measurement query is received. The measurement cannot be
    /// started until a GET is received, but the GET would cause an INTERRUPTED
    /// error.
    #[error(code=-214,message=b"Trigger deadlock")]
    TriggerDeadlock,
    /// `-215, "Arm deadlock"`
    ///
    /// Indicates that the arm source for the initiation of a measurement is set to GET
    /// and subsequent measurement query is received. The measurement cannot be
    /// started until a GET is received, but the GET would cause an INTERRUPTED
    /// error.
    #[error(code=-215,message=b"Arm deadlock")]
    ArmDeadlock,
    /// `-220, "Parameter error"`
    ///
    /// Indicates that a program data element related error occurred. This error message
    /// should be used when the device cannot detect the more specific errors described
    /// for errors -221 through -229.
    #[error(code=-220,message=b"Parameter error")]
    ParameterError,
    /// `-221, "Settings conflict"`
    ///
    /// Indicates that a legal program data element was parsed but could not be executed
    /// due to the current device state (see IEEE 488.2, 6.4.5.3 and 11.5.1.1.5.)
    #[error(code=-221,message=b"Settings conflict")]
    SettingsConflict,
    /// `-222, "Data out of range"`
    ///
    /// Indicates that a legal program data element was parsed but could not be executed
    /// because the interpreted value was outside the legal range as defined by the device
    /// (see IEEE 488.2, 11.5.1.1.5.)
    #[error(code=-222,message=b"Data out of range")]
    DataOutOfRange,
    /// `-223, "Too much data"`
    ///
    /// Indicates that a legal program data element of block, expression, or string type
    /// was received that contained more data than the device could handle due to
    /// memory or related device-specific requirements.
    #[error(code=-223,message=b"Too much data")]
    TooMuchData,
    /// `-224, "Illegal parameter value"`
    ///
    /// Used where exact value, from a list of possibles, was expected.]
    #[error(code=-224,message=b"Illegal parameter value")]
    IllegalParameterValue,
    /// `-225, "Out of memory"`
    ///
    /// The device has insufficent memory to perform the requested
    /// operation.
    #[error(code=-225,message=b"Out of memory")]
    OutOfMemory,
    /// `-226, "Lists not same length"`
    ///
    /// Attempted to use LIST structure having individual LIST’s
    /// of unequal lengths.
    #[error(code=-226,message=b"Lists not same length")]
    ListsNotSameLength,
    /// `-230, "Data corrupt or stale"`
    ///
    /// Possibly invalid data; new reading started but not completed since last access.
    #[error(code=-230,message=b"Data corrupt or stale")]
    DataCorruptOrStale,
    /// `-231, "Data questionable"`
    ///
    /// Indicates that measurement accuracy is suspect.
    #[error(code=-231,message=b"Data questionable")]
    DataQuestionable,
    /// `-232, "Invalid format"`
    ///
    /// Indicates that a legal program data element was parsed but could not be executed
    /// because the data format or structure is inappropriate. For example when loading
    /// memory tables or when sending a SYSTem:SET parameter from an unknown
    /// instrument.
    #[error(code=-232,message=b"Invalid format")]
    InvalidFormat,
    /// `-233, "Invalid version"`
    ///
    /// Indicates that a legal program data element was parsed but could not be executed
    /// because the version of the data is incorrect to the device. This particular error
    /// should be used when file or block data formats are recognized by the instrument
    /// but cannot be executed for reasons of version incompatibility. For example, a not
    /// supported file version, a not supported instrument version
    #[error(code=-233,message=b"Invalid version")]
    InvalidVersion,
    /// `-240, "Hardware error"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// of a hardware problem in the device. Definition of what constitutes a hardware
    /// problem is completely device-specific. This error message should be used when
    /// the device cannot detect the more specific errors described for errors -241 through
    /// -249.
    #[error(code=-240,message=b"Hardware error")]
    HardwareError,
    /// `-241, "Hardware missing"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// of missing device hardware; for example, an option was not installed. Definition
    /// of what constitutes missing hardware is completely device-specific.
    #[error(code=-241,message=b"Hardware missing")]
    HardwareMissing,
    /// `-250, "Mass storage error"`
    ///
    /// Indicates that a mass storage error occurred. This error message should be used
    /// when the device cannot detect the more specific errors described for errors -251
    /// through -259.
    #[error(code=-250,message=b"Mass storage error")]
    MassStorageError,
    /// `-251, "Missing mass storage"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// of missing mass storage; for example, an option that was not installed. Definition
    /// of what constitutes missing mass storage is device-specific.
    #[error(code=-251,message=b"Missing mass storage")]
    MissingMassStorage,
    /// `-252, "Missing media"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// of a missing media; for example, no disk. The definition of what constitutes
    /// missing media is device-specific.
    #[error(code=-252,message=b"Missing media")]
    MissingMedia,
    /// `-253, "Corrupt media"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// of corrupt media; for example, bad disk or wrong format. The definition of what
    /// constitutes corrupt media is device-specific.
    #[error(code=-253,message=b"Corrupt media")]
    CorruptMedia,
    /// `-254, "Media full"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// the media was full; for example, there is no room on the disk. The definition of
    /// what constitutes a full media is device-specific.
    #[error(code=-254,message=b"Media full")]
    MediaFull,
    /// `-255, "Directory full"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// the media directory was full. The definition of what constitutes a full media
    /// directory is device-specific.
    #[error(code=-255,message=b"Directory full")]
    DirectoryFull,
    /// `-256, "File name not found"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// the file name on the device media was not found; for example, an attempt was
    /// made to read or copy a nonexistent file. The definition of what constitutes a file
    /// not being found is device-specific.
    #[error(code=-256,message=b"Filename not found")]
    FileNameNotFound,
    /// `-257, "Filename error"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// the file name on the device media was in error; for example, an attempt was made
    /// to copy to a duplicate file name. The definition of what constitutes a file name
    /// error is device-specific.
    #[error(code=-257,message=b"Filename error")]
    FileNameError,
    /// `-258, "Media protected"`
    ///
    /// Indicates that a legal program command or query could not be executed because
    /// the media was protected; for example, the write-protect tab on a disk was present.
    /// The definition of what constitutes protected media is device-specific.
    #[error(code=-258,message=b"Media protected")]
    MediaProtected,
    /// `-260, "Expression error"`
    ///
    /// Indicates that a expression program data element related error occurred. This
    /// error message should be used when the device cannot detect the more specific
    /// errors described for errors -261 through -269.
    #[error(code=-260,message=b"Expression error")]
    ExecExpressionError, //Also declared in 170?
    /// `-261, "Math error in expression"`
    ///
    /// Indicates that a syntactically legal expression program data element could not be
    /// executed due to a math error; for example, a divide-by-zero was attempted. The
    /// definition of math error is device-specific.
    #[error(code=-261,message=b"Math error in expression")]
    MathErrorInExpression,
    /// `-270, "Macro error"`
    ///
    /// Indicates that a macro-related execution error occurred. This error message
    /// should be used when the device cannot detect the more specific errors described
    /// for errors -271 through -279.
    #[error(code=-270,message=b"Macro error")]
    ExecMacroError,
    /// `-271, "Macro syntax error"`
    ///
    /// Indicates that that a syntactically legal macro program data sequence, according
    /// to IEEE 488.2, 10.7.2, could not be executed due to a syntax error within the
    /// macro definition (see IEEE 488.2, 10.7.6.3.)
    #[error(code=-271,message=b"Macro syntax error")]
    MacroSyntaxError,
    /// `-272, "Macro execution error"`
    ///
    /// Indicates that a syntactically legal macro program data sequence could not be
    /// executed due to some error in the macro definition (see IEEE 488.2, 10.7.6.3.)
    #[error(code=-272,message=b"Macro execution error")]
    MacroExecutionError,
    /// `-273, "Illegal macro label"`
    ///
    /// Indicates that the macro label defined in the *DMC command was a legal string
    /// syntax, but could not be accepted by the device (see IEEE 488.2, 10.7.3 and
    /// 10.7.6.2); for example, the label was too long, the same as a common command
    /// header, or contained invalid header syntax.]
    #[error(code=-273,message=b"Illegal macro label")]
    IllegalMacroLabel,
    /// `-274, "Macro parameter error"`
    ///
    /// Indicates that the macro definition improperly used a macro parameter
    /// placeholder (see IEEE 488.2, 10.7.3).
    #[error(code=-274,message=b"Macro parameter error")]
    ExecMacroParameterError,
    /// `-275, "Macro definition too long"`
    ///
    /// Indicates that a syntactically legal macro program data sequence could not be
    /// executed because the string or block contents were too long for the device to
    /// handle (see IEEE 488.2, 10.7.6.1).
    #[error(code=-275,message=b"Macro definition too long")]
    MacroDefinitionTooLong,
    /// `-276, "Macro recursion error"`
    ///
    /// Indicates that a syntactically legal macro program data sequence could not be
    /// executed because the device found it to be recursive (see IEEE 488.2, 10.7.6.6).
    #[error(code=-276,message=b"Macro recursion error")]
    MacroRecursionError,
    /// `-277, "Macro redefinition not allowed"`
    ///
    /// Indicates that a syntactically legal macro label in the *DMC command could not
    /// be executed because the macro label was already defined (see IEEE 488.2,
    /// 10.7.6.4).
    #[error(code=-277,message=b"Macro redefinition not allowed")]
    MacroRedefinitionNotAllowed,
    /// `-278, "Macro header not found"`
    ///
    /// Indicates that a syntactically legal macro label in the *GMC? query could not be
    /// executed because the header was not previously defined.
    #[error(code=-278,message=b"Macro header not found")]
    MacroHeaderNotFound,
    /// `-280, "Program error"`
    ///
    /// Indicates that a downloaded program-related execution error occurred. This error
    /// message should be used when the device cannot detect the more specific errors
    /// described for errors -281 through -289. A downloaded program is used to add
    /// algorithmic capability to a device. The syntax used in the program and the
    /// mechanism for downloading a program is device-specific.
    #[error(code=-280,message=b"Program error")]
    ProgramError,
    /// `-281, "Cannot create program"`
    ///
    /// Indicates that an attempt to create a program was unsuccessful. A reason for the
    /// failure might include not enough memory.
    #[error(code=-281,message=b"Cannot create program")]
    CannotCreateProgram,
    /// `-282, "Illegal program name"`
    ///
    /// The name used to reference a program was invalid; for example, redefining an
    /// existing program, deleting a nonexistent program, or in general, referencing a
    /// nonexistent program.
    #[error(code=-282,message=b"Illegal program name")]
    IllegalProgramName,
    /// `-283, "Illegal variable name"`
    ///
    /// An attempt was made to reference a nonexistent variable in a program.
    #[error(code=-283,message=b"Illegal variable name")]
    IllegalVariableName,
    /// `-284, "Program currently running"`
    ///
    /// Certain operations dealing with programs may be illegal while the program is
    /// running; for example, deleting a running program might not be possible.
    #[error(code=-284,message=b"Program currently running")]
    ProgramCurrentlyRunning,
    /// `-285, "Program syntax error"`
    ///
    /// Indicates that a syntax error appears in a downloaded program. The syntax used
    /// when parsing the downloaded program is device-specific.
    #[error(code=-285,message=b"Program syntax error")]
    ProgramSyntaxError,
    /// `-286, "Program runtime error"`
    ///
    ///
    #[error(code=-286,message=b"Program runtime error")]
    ProgramRuntimeError,
    /// `-290, "Memory use error"`
    ///
    /// Indicates that a user request has directly or indirectly caused an error related to
    /// memory or <data_handle>s, this is not the same as “bad” memory.
    #[error(code=-290,message=b"Memory use error")]
    MemoryUseError,
    /// `-291, "Out of memory"`
    ///
    ///
    #[error(code=-291,message=b"Out of memory")]
    UseOutOfMemory,
    /// `-292, "Referenced name does not exist"`
    ///
    ///
    #[error(code=-292,message=b"Referenced name does not exist")]
    ReferencedNameDoesNotExist,
    /// `-293, "Referenced name already exists"`
    ///
    ///
    #[error(code=-293,message=b"Referenced name already exists")]
    ReferencedNameAlreadyExists,
    /// `-294, "Incompatible type"`
    ///
    /// Indicates that the type or structure of a memory item is inadequate.
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
    /// `-300, Device-specific error`
    ///
    /// This is the generic device-dependent error for devices that cannot detect more
    /// specific errors. This code indicates only that a Device-Dependent Error as defined
    /// in IEEE 488.2, 11.5.1.1.6 has occurred.
    #[error(code=-300,message=b"Device-specific error")]
    DeviceSpecificError,
    /// `-310, "System error"`
    ///
    /// Indicates that some error, termed “system error” by the device, has occurred.
    /// This code is device-dependent.
    #[error(code=-310,message=b"System error")]
    SystemError,
    /// `-311, "Memory error"`
    ///
    /// Indicates some physical fault in the device’s memory, such as parity error.
    #[error(code=-311,message=b"Memory error")]
    MemoryError,
    /// `-312, "PUD memory lost"`
    ///
    /// Indicates that the protected user data saved by the *PUD command has been lost.
    #[error(code=-312,message=b"PUD memory lost")]
    PudMemoryLost,
    /// `-313, "Calibration memory lost"`
    ///
    /// Indicates that nonvolatile calibration data used by the *CAL? command has been
    /// lost.
    #[error(code=-313,message=b"Calibration memory lost")]
    CalibrationMemoryLost,
    /// `-314, "Save/recall memory lost"`
    ///
    /// Indicates that the nonvolatile data saved by the *SAV? command has been lost.
    #[error(code=-314,message=b"Save/recall memory lost")]
    SaveRecallMemoryLost,
    /// `-315, "Configuration memory lost"`
    ///
    /// Indicates that nonvolatile configuration data saved by the device has been lost.
    /// The meaning of this error is device-specific.
    #[error(code=-315,message=b"Configuration memory lost")]
    ConfigurationMemoryLost,
    /// `-320, "Storage fault"`
    ///
    /// Indicates that the firmware detected a fault when using data storage. This error is
    /// not an indication of physical damage or failure of any mass storage element.
    #[error(code=-320,message=b"Storage fault")]
    StorageFault,
    /// `-321, "Out of memory"`
    ///
    /// An internal operation needed more memory than was available.
    #[error(code=-321,message=b"Out of memory")]
    StOutOfMemory,
    /// `-330, "Self-test failed"`
    ///
    ///
    #[error(code=-330,message=b"Self-test failed")]
    SelfTestFailed,
    /// `-340, "Calibration failed"`
    ///
    ///
    #[error(code=-340,message=b"Calibration failed")]
    CalibrationFailed,
    /// `-350, "Queue overflow"`
    ///
    /// A specific code entered into the queue in lieu of the code that caused the error.
    /// This code indicates that there is no room in the queue and an error occurred but
    /// was not recorded.
    #[error(code=-350,message=b"Queue overflow")]
    QueueOverflow,
    /// `-360, "Communication error"`
    ///
    /// This is the generic communication error for devices that cannot detect the more
    /// specific errors described for errors -361 through -363.
    #[error(code=-360,message=b"Communication error")]
    CommunicationError,
    /// `-361, "Parity error in program message"`
    ///
    /// Parity bit not correct when data received for example, on a serial port.
    #[error(code=-361,message=b"Parity error in program message")]
    ParityErrorInProgramMessage,
    /// `-362, "Framing error in program message"`
    ///
    /// A stop bit was not detected when data was received for example, on a serial port
    /// (for example, a baud rate mismatch).
    #[error(code=-362,message=b"Framing error in program message")]
    FramingErrorInProgramMessage,
    /// `-363, "Input buffer overrun"`
    ///
    /// Software or hardware input buffer on serial port overflows with data caused by
    /// improper or nonexistent pacing.
    #[error(code=-363,message=b"Input buffer overrun")]
    InputBufferOverrun,
    /// `-365, "Time out error"`
    ///
    /// This is a generic device-dependent error.
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
    /// `-400, "Query Error"`
    ///
    /// This is the generic query error for devices that cannot detect more specific errors.
    /// This code indicates only that a Query Error as defined in IEEE 488.2, 11.5.1.1.7
    /// and 6.3 has occurred.
    #[error(code=-400,message=b"Query error")]
    QueryError,
    /// `-410, "Query INTERRUPTED"`
    ///
    /// Indicates that a condition causing an INTERRUPTED Query error occurred (see
    /// IEEE 488.2, 6.3.2.3); for example, a query followed by DAB or GET before a
    /// response was completely sent.
    #[error(code=-410,message=b"Query INTERRUPTED")]
    QueryInterrupted,
    /// `-420, "Query UNTERMINATED"`
    ///
    /// Indicates that a condition causing an UNTERMINATED Query error occurred
    /// (see IEEE 488.2, 6.3.2.2); for example, the device was addressed to talk and an
    /// incomplete program message was received.
    #[error(code=-420,message=b"Query UNTERMINATED")]
    QueryUnterminated,
    /// `-430, "Query DEADLOCKED"`
    ///
    /// Indicates that a condition causing an DEADLOCKED Query error occurred (see
    /// IEEE 488.2, 6.3.1.7); for example, both input buffer and output buffer are full and
    /// the device cannot continue.
    #[error(code=-430,message=b"Query DEADLOCKED")]
    QueryDeadlocked,
    /// `-440, "Query UNTERMINATED after indefinite response"`
    ///
    /// Indicates that a query was received in the same program message after an query
    /// requesting an indefinite response was executed (see IEEE 488.2, 6.5.7.5).
    #[error(code=-440,message=b"Query UNTERMINATED after indefinite response")]
    QueryUnterminatedAfterIndefiniteResponse,

    ///# Power on event [-599, -500]
    ///> An <error/event number> in the range `[-599:-500]` is used when the instrument wishes to
    ///> report a 488.2 power on event to the event/error queue. This event occurs when the
    ///> instrument detects an off to on transition in its power supply. This event also sets the power
    ///> on bit, (bit 7) of the Standard Event Status Register. See IEEE 488.2, section 11.5.1.
    /// ---
    /// `-500, "Power on"`
    ///
    /// The instrument has detected an off to on transition in its power supply.
    #[error(code=-500,message=b"Power on")]
    PowerOn,

    ///# User request event [-699, -600]
    ///> An <error/event number> in the range `[-699:-600]` is used when the instrument wishes to
    ///> report a 488.2 user request event. This event occurs when the instrument detects the
    ///> activation of a user request local control. This event also sets the user request bit (bit 6) of
    ///> the Standard Event Status Register. See IEEE 488.2, section 11.5.1.
    /// ---
    /// `-600, "User request"`
    ///
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
    /// `-700, "Request Control"`
    ///
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
    /// `-800, "Operation Complete"`
    ///
    /// The instrument has completed all selected pending operations in accordance with
    /// the IEEE 488.2, 12.5.2 synchronization protocol
    #[error(code=-800,message=b"Operation complete")]
    OperationComplete,
}

impl ErrorCode {
    /**
     * Returns a bitmask for the appropriate bit in the ESR for this event/error.
     */
    pub fn esr_mask(&self) -> u8 {
        match self.get_code() {
            -99..=0 => 0u8,        //No bit
            -199..=-100 => 0x20u8, //bit 5
            -299..=-200 => 0x10u8, //bit 4
            -399..=-300 => 0x08u8, //bit 3
            -499..=-400 => 0x04u8, //bit 2
            -599..=-500 => 0x80u8, //bit 7
            -699..=-600 => 0x40u8, //bit 6
            -799..=-700 => 0x02u8, //bit 1
            -899..=-800 => 0x01u8, //bit 0
            _ => 0x08u8,           //bit 3
        }
    }
}

/// Generic error queue trait
pub trait ErrorQueue {
    /// Add a error to the queue.
    /// Shall replace last error with a QueueOverflow error if full.
    fn push_back_error(&mut self, err: Error);

    /// Get a error to the queue
    /// Shall return NoError if empty.
    fn pop_front_error(&mut self) -> Option<Error>;

    /// Current length of queue
    fn num_errors(&self) -> usize;

    /// Clear queue
    fn clear_errors(&mut self);

    /// Is queue empty?
    fn is_empty(&self) -> bool {
        self.num_errors() == 0
    }
}

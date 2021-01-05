use crate::hislip::{Message, MessageType};

#[derive(Debug, Copy, Clone)]
pub enum Error {
    None,
    Fatal(FatalErrorCode, &'static [u8]),
    NonFatal(NonFatalErrorCode, &'static [u8]),
}

impl Error {
    pub fn message(&self) -> Option<Message> {
        match self {
            Error::None => None,
            Error::Fatal(code, msg) => {
                Some(MessageType::FatalError.message_params(msg, code.error_code(), 0))
            }
            Error::NonFatal(code, msg) => {
                Some(MessageType::FatalError.message_params(msg, code.error_code(), 0))
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FatalErrorCode {
    UnidentifiedError,
    PoorlyFormattedMessageHeader,
    AttemptUseWithoutBothChannels,
    InvalidInitialization,
    MaximumClientsExceeded,
    SecureConnectionFailed,
    Extension(u8),
    DeviceDefined(u8),
}

impl FatalErrorCode {
    pub fn error_code(&self) -> u8 {
        match self {
            FatalErrorCode::UnidentifiedError => 0,
            FatalErrorCode::PoorlyFormattedMessageHeader => 1,
            FatalErrorCode::AttemptUseWithoutBothChannels => 2,
            FatalErrorCode::InvalidInitialization => 3,
            FatalErrorCode::MaximumClientsExceeded => 4,
            FatalErrorCode::SecureConnectionFailed => 5,
            FatalErrorCode::Extension(x) => *x,
            FatalErrorCode::DeviceDefined(x) => *x,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NonFatalErrorCode {
    UnidentifiedError,
    UnrecognizedMessageType,
    UnrecognizedControlCode,
    UnrecognizedVendorDefinedMessage,
    MessageTooLarge,
    AuthenticationFailed,
    Extension(u8),
    DeviceDefined(u8),
}

impl NonFatalErrorCode {
    pub fn error_code(&self) -> u8 {
        match self {
            NonFatalErrorCode::UnidentifiedError => 0,
            NonFatalErrorCode::UnrecognizedMessageType => 1,
            NonFatalErrorCode::UnrecognizedControlCode => 2,
            NonFatalErrorCode::UnrecognizedVendorDefinedMessage => 3,
            NonFatalErrorCode::MessageTooLarge => 4,
            NonFatalErrorCode::AuthenticationFailed => 5,
            NonFatalErrorCode::Extension(x) => *x,
            NonFatalErrorCode::DeviceDefined(x) => *x,
        }
    }
}

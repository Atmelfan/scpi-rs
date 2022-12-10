extern crate self as scpi;
use crate::{
    error::{Error, ErrorCode},
    parser::{response::ResponseData, tokenizer::Token},
};

use self::function::SensorFunction;

use super::ScpiDevice;

pub mod function;

// Sense functions
pub mod common;
#[cfg(feature = "unit-electric-current")]
pub mod current;
#[cfg(feature = "unit-electric-resistance")]
pub mod resistance;
#[cfg(feature = "unit-electric-potential")]
pub mod voltage;

pub trait SenseFunction {
    type Unit: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData;
}

pub trait Sense<const N: usize = 1> {
    fn function_on(&mut self, function: SensorFunction) -> Result<(), FunctionError>;
    fn get_function_on(&self) -> Result<SensorFunction, FunctionError>;
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionError {
    /// Specified funcion is not supported
    FunctionNotSupported,
    /// Specified suffix is not supported for this function or at all
    SuffixNotSupported,
    /// Specified presentation is not supported for this function or at all
    PresentationNotSupported,
    /// Some other error
    Other,
}

impl From<FunctionError> for Error {
    fn from(err: FunctionError) -> Self {
        match err {
            FunctionError::FunctionNotSupported => {
                Self::extended(ErrorCode::IllegalParameterValue, b"Function not supported")
            }
            FunctionError::SuffixNotSupported => Self::extended(
                ErrorCode::IllegalParameterValue,
                b"Sensor function suffix not supported",
            ),
            FunctionError::PresentationNotSupported => Self::extended(
                ErrorCode::IllegalParameterValue,
                b"Sensor function presentation not supported",
            ),
            FunctionError::Other => Self::new(ErrorCode::IllegalParameterValue),
        }
    }
}

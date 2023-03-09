use scpi::{
    error::{Error, ErrorCode},
    parser::{response::ResponseData, tokenizer::Token},
};

use core::fmt::Debug;

use super::ScpiDevice;

pub mod function;

// Sense functions
pub mod common;
#[cfg(feature = "unit-electric-current")]
pub mod current;
#[cfg(feature = "unit-electrical-resistance")]
pub mod resistance;
#[cfg(feature = "unit-electric-potential")]
pub mod voltage;

pub trait SenseFunction {
    type Unit: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData;
}

pub trait Sense<const N: usize = 1> {
    /// Sensor function type for the `SENSe:FUNCtion:ON[?] <sensor_function>` command.
    /// Should be convertable from a  string data token and returnable as a response by the query form.
    ///
    /// See [function::SensorFunction] for a simple function type or base for rollling your own.
    type Function: for<'a> TryFrom<Token<'a>, Error = Error> + ResponseData;

    fn function_on(&mut self, function: Self::Function) -> Result<(), FunctionError>;
    fn get_function_on(&self) -> Result<Self::Function, FunctionError>;
}

pub trait Sens<Func: SenseFunction, const N: usize = 1>: Sense<N> {}

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
                Self::new(ErrorCode::IllegalParameterValue).extended(b"Function not supported")
            }
            FunctionError::SuffixNotSupported => Self::new(ErrorCode::IllegalParameterValue)
                .extended(b"Sensor function suffix not supported"),
            FunctionError::PresentationNotSupported => Self::new(ErrorCode::IllegalParameterValue)
                .extended(b"Sensor function presentation not supported"),
            FunctionError::Other => Self::new(ErrorCode::IllegalParameterValue),
        }
    }
}

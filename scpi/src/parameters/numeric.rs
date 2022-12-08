use crate::{
    error::{Error, ErrorCode, Result},
    prelude::Token,
    util,
};

/// Numeric values that can be substituted for `<numeric>`
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NumericValue<T> {
    /// `MAXimum`
    Maximum,
    /// `MINimum`
    Minimum,
    /// `DEFault`
    Default,
    /// `UP`
    Up,
    /// `DOWN`
    Down,
    /// `AUTO`
    Auto,
    /// Number
    Value(T),
}

impl<T> NumericValue<T> {
    /// Return value if possible
    pub fn value(&self) -> Option<&T> {
        if let Self::Value(t) = self {
            Some(t)
        } else {
            None
        }
    }

    /// Return value if possible
    pub fn with(self, max: T, min: T) -> NumericBuilder<T>
    where
        T: PartialOrd,
    {
        NumericBuilder::new(self, max, min)
    }

    pub fn map<F, U>(self, f: F) -> NumericValue<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            NumericValue::Maximum => NumericValue::Maximum,
            NumericValue::Minimum => NumericValue::Minimum,
            NumericValue::Default => NumericValue::Default,
            NumericValue::Up => NumericValue::Up,
            NumericValue::Down => NumericValue::Down,
            NumericValue::Auto => NumericValue::Auto,
            NumericValue::Value(t) => NumericValue::Value(f(t)),
        }
    }
}

pub struct NumericBuilder<T> {
    value: NumericValue<T>,
    max: T,
    min: T,
    default: Option<T>,
}

impl<T> NumericBuilder<T>
where
    T: PartialOrd,
{
    fn new(value: NumericValue<T>, max: T, min: T) -> Self {
        Self {
            value,
            max,
            min,
            default: Default::default(),
        }
    }

    /// Set the default value
    pub fn default(self, value: T) -> Self {
        Self {
            default: Some(value),
            ..self
        }
    }

    pub fn finish(self) -> Result<T> {
        match self.value {
            NumericValue::Maximum => Ok(self.max),
            NumericValue::Minimum => Ok(self.min),
            NumericValue::Default => self.default.ok_or(ErrorCode::IllegalParameterValue.into()),
            NumericValue::Up => Err(ErrorCode::IllegalParameterValue.into()),
            NumericValue::Down => Err(ErrorCode::IllegalParameterValue.into()),
            NumericValue::Auto => Err(ErrorCode::IllegalParameterValue.into()),
            NumericValue::Value(t) => {
                if t <= self.max && t >= self.min {
                    Ok(t)
                } else {
                    Err(ErrorCode::DataOutOfRange.into())
                }
            }
        }
    }

    pub fn finish_auto<Auto>(self, auto: Auto) -> Result<T>
    where
        Auto: FnOnce() -> T,
    {
        match self.value {
            NumericValue::Auto => Ok(auto()),
            _ => self.finish(),
        }
    }
}

impl<'a, T> TryFrom<Token<'a>> for NumericValue<T>
where
    T: TryFrom<Token<'a>, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Self> {
        match value {
            Token::CharacterProgramData(s) => match s {
                //Check for special float values
                ref x if util::mnemonic_compare(b"MAXimum", x) => Ok(Self::Maximum),
                ref x if util::mnemonic_compare(b"MINimum", x) => Ok(Self::Minimum),
                ref x if util::mnemonic_compare(b"DEFault", x) => Ok(Self::Default),
                ref x if util::mnemonic_compare(b"UP", x) => Ok(Self::Up),
                ref x if util::mnemonic_compare(b"DOWN", x) => Ok(Self::Down),
                ref x if util::mnemonic_compare(b"AUTO", x) => Ok(Self::Auto),
                _ => Ok(Self::Value(T::try_from(value)?)),
            },
            t => Ok(Self::Value(T::try_from(t)?)),
        }
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    fn test_numeric() {}
}

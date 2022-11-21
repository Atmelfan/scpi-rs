use crate::{
    error::{Error, ErrorCode, Result},
    prelude::Token,
    util,
};

pub trait Num: Default + PartialOrd + Copy {
    fn up(&self) -> Self;
    fn down(&self) -> Self;
}

macro_rules! impl_num {
    (float $t:ident) => {
        impl Num for $t {
            fn up(&self) -> Self {
                self + 1.0
            }
        
            fn down(&self) -> Self {
                self - 1.0
            }
        }
    };
    (int $t:ident) => {
        impl Num for $t {
            fn up(&self) -> Self {
                self.saturating_add(1)
            }
        
            fn down(&self) -> Self {
                self.saturating_sub(1)
            }
        }
    };
}

impl_num!(float f32);
impl_num!(float f64);
impl_num!(int u8);
impl_num!(int i8);
impl_num!(int u16);
impl_num!(int i16);
impl_num!(int u32);
impl_num!(int i32);
impl_num!(int u64);
impl_num!(int i64);

/// Numeric values that can be substituted for `<numeric>`
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NumericValues<T> {
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

impl<T> NumericValues<T> {
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
        T: Num,
    {
        NumericBuilder::new(self, max, min)
    }
}

pub struct NumericBuilder<T> {
    value: NumericValues<T>,
    max: T,
    min: T,
    default: Option<T>,
    current: Option<T>,
}

impl<T> NumericBuilder<T>
where
    T: Num,
{
    fn new(value: NumericValues<T>, max: T, min: T) -> Self {
        Self {
            value,
            max,
            min,
            default: Default::default(),
            current: None,
        }
    }

    /// Set the default value
    pub fn default(self, value: T) -> Self {
        Self {
            default: Some(value),
            ..self
        }
    }

    /// Set the current value, used by `UP` and `DOWN` parameters
    pub fn current(self, value: T) -> Self {
        Self {
            current: Some(value),
            ..self
        }
    }

    pub fn check(&self, t: T) -> Result<T> {
        if t > self.max {
            Err(ErrorCode::DataOutOfRange.into())
        } else if t < self.min {
            Err(ErrorCode::DataOutOfRange.into())
        } else {
            Ok(t)
        }
    }

    pub fn finish(&self) -> Result<T> {
        match self.value {
            NumericValues::Maximum => Ok(self.max),
            NumericValues::Minimum => Ok(self.min),
            NumericValues::Default => self.default.ok_or(ErrorCode::IllegalParameterValue.into()),
            NumericValues::Up => self
                .current
                .ok_or(ErrorCode::IllegalParameterValue.into())
                .map(|c| c.up())
                .and_then(|t| self.check(t)),
            NumericValues::Down => self
                .current
                .ok_or(ErrorCode::IllegalParameterValue.into())
                .map(|c| c.down())
                .and_then(|t| self.check(t)),
            NumericValues::Auto => Err(ErrorCode::IllegalParameterValue.into()),
            NumericValues::Value(t) => self.check(t),
        }
    }

    pub fn finish_auto<Auto>(&self, auto: Auto) -> Result<T>
    where
        Auto: FnOnce() -> T,
    {
        match self.value {
            NumericValues::Auto => Ok(auto()),
            _ => self.finish(),
        }
    }
}

impl<'a, T> TryFrom<Token<'a>> for NumericValues<T>
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

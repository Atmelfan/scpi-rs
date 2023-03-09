use core::ops::{Add, Div, Mul, Sub};

use scpi::{
    error::{Error, ErrorCode, Result},
    parser::{mnemonic_compare, tokenizer::Token},
    units::uom::{
        num_traits::Num,
        si::{Dimension, Units},
        Conversion,
    },
};

/// SCPI `<numeric_value>`
///
/// # <numeric_value> Definition
/// The decimal numeric element is abbreviated as <numeric_value> throughout this document.
///
/// This is different from the <NRf> described in section 7.7.2.1 of IEEE 488.2 in several ways.
/// Several forms of <CHARACTER PROGRAM DATA> are defined as special forms of
/// numbers. These are: MINimum, MAXimum, DEFault, UP, DOWN, Not A Number (NAN),
/// INFinity, and Negative INFinity (NINF). Individual commands are required to accept MIN
/// and MAX. DEFault, UP, DOWN, NAN, INFinity, and NINFinity may be implemented at the
/// discretion of the designer, in which case it shall be noted in the instrument documentation.
/// Where an optional form is accepted, it will be noted in the command description.
///
/// A <non-decimal numeric> (IEEE 488.2, section 7.7.4), a <numeric_expression>, or a
/// <label> is part of <numeric_value> if an instrument implements these features.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NumericValue<T> {
    /// ## <NRf>
    Value(T),

    Maximum,
    /// ## MINimum|MAXimum
    ///
    /// The special form numeric parameters MINimum and MAXimum shall be provided which
    /// assume the limit values for the parameter. The maximum and minimum shall be queryable
    /// by sending <header>? MAXimum|MINimum. The MAXimum value refers to the largest
    /// value that the function can currently be set to, and MINimum refers to the value closest to
    /// negative infinity that the function can be currently set to.
    ///
    /// Some commands have multiple parameters. The query form of these commands returns a list
    /// of values representing the current value of each of the parameters, in the order of their
    /// normal occurrence in a program message. If a MINimum/MAXimum query of multiple
    /// parameters is allowed, the keywords MINimum and MAXimum must occur as many times in
    /// the query as there are parameters. MINimum requests that the instrument return the legal
    /// value which is closest to negative infinity for the parameter; MAXimum requests the legal
    /// value which is closest to positive infinity.
    ///
    /// For example, suppose an instrument implements the SYST:TIME command, which requires
    /// three parameters, and allows MIN/MAX queries on this command. The following queries
    /// shall have these results:
    /// * `SYST:TIME?<nl>` shall return the current setting of the time-of-day clock in the
    /// instrument.
    /// * `SYST:TIME? MAX,MAX,MAX<nl>` could return 23,59,59.
    /// * `SYST:TIME? MAX<nl>` shall set an error (-109, “Missing parameter”), since
    /// three parameters are required and only one was sent.
    Minimum,

    /// ## DEFault
    ///
    /// The special <numeric_value> parameter DEFault may be provided to allow the instrument to
    /// select a value for a parameter. When DEFault is sent, the instrument shall select a value
    /// which is deemed to be convenient to the customer. The setting may be device-dependent, or
    /// it may be defined as part of this standard. The use of DEFault is optional on a
    /// command-by-command basis. Individual commands shall document where DEFault is
    /// required.
    ///
    /// For example, to use the `SYST:TIME` command to set a device’s clock ahead one hour
    /// (Daylight Savings Time), a program might send `SYST:TIME UP,DEF,DEF<nl>`.
    /// Another example is found in the MEASure commands. The syntax of the command which
    /// measures DC voltage is:
    /// ```text
    ///  MEASure:VOLTage:DC [<expected value>[,<resolution>]]
    /// ```
    /// The MEASure command specifies that parameters are defaulted from the right and that any
    /// parameter may be defaulted by using DEFault in place of the parameter. The following
    /// command would measure DC voltage, defaulting the range to an instrument dependent value
    /// (possibly autorange), but specifying the resolution at 0.001 Volt:
    /// ```text
    ///  MEASure:VOLTage:DC DEFault,0.001V
    /// ```
    Default,
    Up,
    /// ## UP|DOWN
    ///
    /// An instrument may optionally allow the use of steps for some or all of its numeric entry. If
    /// steps are used, the keywords UP and DOWN shall be used as numeric parameters which
    /// perform stepping. Steps may be adjustable through the step node for each individual
    /// parameter.
    ///
    /// The instrument may step a parameter when UP or DOWN is received in lieu of a numeric
    /// value. This capability is optional. However, if the capability is implemented, the device
    /// shall include a node for each command which accepts step parameters. This node will
    /// specify the step. The step may either be a fixed linear size or a logarithmic number
    /// representing number of decades/step.
    Down,
}

impl<T> Default for NumericValue<T> {
    fn default() -> Self {
        Self::Default
    }
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

    /// Create a [NumericBuilder] with max/min the type maximum and minimum values (see [NumericValueDefaults]).
    ///
    /// ```
    /// # use scpi_contrib::scpi1999::NumericValue;
    /// let x: NumericValue<f32> = NumericValue::Default;
    ///
    /// // Use builder to resolve special values
    /// let value = x.build()
    ///     // Specify maximum value, defaults to f32::MAX
    ///     .max(100.0)
    ///     // Specify minimum value, defaults to f32::MIN
    ///     .min(-100.0)
    ///     // Specify the default value, otherwise DEFault wont be accepted.
    ///     .default(1.0)
    ///     // Finish the builder and resolve the final value
    ///     .finish();
    ///
    /// assert_eq!(value.unwrap(), 1.0)
    /// ```
    pub fn build(self) -> NumericBuilder<T>
    where
        T: PartialOrd + NumericValueDefaults,
    {
        NumericBuilder::from(self)
    }

    /// Shorthand for
    /// ```
    /// # use scpi_contrib::scpi1999::NumericValue;
    /// # let numeric_value: NumericValue<f32> = NumericValue::Default;
    /// # let max = 10.0;
    /// # let min = -10.0;
    /// numeric_value.build().max(max).min(min).finish();
    /// ```
    pub fn finish_with(self, max: T, min: T) -> Result<T>
    where
        T: PartialOrd + NumericValueDefaults,
    {
        self.build().max(max).min(min).finish()
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
            NumericValue::Value(t) => NumericValue::Value(f(t)),
        }
    }
}

impl<T, Rhs> Add<Rhs> for NumericValue<T>
where
    T: Add<Rhs>,
{
    type Output = NumericValue<T::Output>;

    fn add(self, rhs: Rhs) -> Self::Output {
        self.map(|t| t.add(rhs))
    }
}

impl<T, Rhs> Sub<Rhs> for NumericValue<T>
where
    T: Sub<Rhs>,
{
    type Output = NumericValue<T::Output>;

    fn sub(self, rhs: Rhs) -> Self::Output {
        self.map(|t| t.sub(rhs))
    }
}

impl<T, Rhs> Mul<Rhs> for NumericValue<T>
where
    T: Mul<Rhs>,
{
    type Output = NumericValue<T::Output>;

    fn mul(self, rhs: Rhs) -> Self::Output {
        self.map(|t| t.mul(rhs))
    }
}

impl<T, Rhs> Div<Rhs> for NumericValue<T>
where
    T: Div<Rhs>,
{
    type Output = NumericValue<T::Output>;

    fn div(self, rhs: Rhs) -> Self::Output {
        self.map(|t| t.div(rhs))
    }
}

pub trait NumericValueDefaults {
    fn numeric_value_max() -> Self;
    fn numeric_value_min() -> Self;
}

macro_rules! impl_numeric_default {
    ($typ:ident) => {
        impl NumericValueDefaults for $typ {
            fn numeric_value_max() -> Self {
                $typ::MAX
            }

            fn numeric_value_min() -> Self {
                $typ::MIN
            }
        }
    };
}

impl_numeric_default!(i8);
impl_numeric_default!(u8);
impl_numeric_default!(i16);
impl_numeric_default!(u16);
impl_numeric_default!(i32);
impl_numeric_default!(u32);
impl_numeric_default!(i64);
impl_numeric_default!(u64);
impl_numeric_default!(isize);
impl_numeric_default!(usize);
impl_numeric_default!(f32);
impl_numeric_default!(f64);

impl<D, U, V> NumericValueDefaults for scpi::units::uom::si::Quantity<D, U, V>
where
    D: Dimension + ?Sized,
    U: Units<V> + ?Sized,
    V: Num + Conversion<V> + NumericValueDefaults,
{
    fn numeric_value_max() -> Self {
        Self {
            dimension: Default::default(),
            units: Default::default(),
            value: V::numeric_value_max(),
        }
    }

    fn numeric_value_min() -> Self {
        Self {
            dimension: Default::default(),
            units: Default::default(),
            value: V::numeric_value_min(),
        }
    }
}

///  A helper for resolving a [NumericValue] into a final value
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
    /// Create a new builder with specified MAXimum and MINimum values
    pub fn new(value: NumericValue<T>, max: T, min: T) -> Self {
        Self {
            value,
            max,
            min,
            default: None,
        }
    }

    /// Create a new builder using default provided by [NumericValueDefaults] for the given type.
    /// Useful if there's no inherent min/max value for the value other than the datatype limits.
    fn from(value: NumericValue<T>) -> Self
    where
        T: NumericValueDefaults,
    {
        Self {
            value,
            max: T::numeric_value_max(),
            min: T::numeric_value_min(),
            default: Default::default(),
        }
    }

    /// Set MAXimum value
    pub fn max(self, value: T) -> Self {
        Self { max: value, ..self }
    }

    /// Set MINimum value
    pub fn min(self, value: T) -> Self {
        Self { min: value, ..self }
    }

    /// Set DEFault value
    pub fn default(self, value: T) -> Self {
        Self {
            default: Some(value),
            ..self
        }
    }

    /// Resolve value or return an appropriate error
    pub fn finish(self) -> Result<T> {
        match self.value {
            NumericValue::Maximum => Ok(self.max),
            NumericValue::Minimum => Ok(self.min),
            NumericValue::Default => self
                .default
                .ok_or_else(|| ErrorCode::IllegalParameterValue.into()),
            NumericValue::Up => Err(ErrorCode::IllegalParameterValue.into()),
            NumericValue::Down => Err(ErrorCode::IllegalParameterValue.into()),
            NumericValue::Value(t) => {
                if t <= self.max && t >= self.min {
                    Ok(t)
                } else {
                    Err(ErrorCode::DataOutOfRange.into())
                }
            }
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
                x if mnemonic_compare(b"MAXimum", x) => Ok(Self::Maximum),
                x if mnemonic_compare(b"MINimum", x) => Ok(Self::Minimum),
                x if mnemonic_compare(b"DEFault", x) => Ok(Self::Default),
                x if mnemonic_compare(b"UP", x) => Ok(Self::Up),
                x if mnemonic_compare(b"DOWN", x) => Ok(Self::Down),
                _ => Ok(Self::Value(T::try_from(value)?)),
            },
            t => Ok(Self::Value(T::try_from(t)?)),
        }
    }
}

/// A mirror of [NumericValue] which only matches MAXimum|MINimum|DEFault for queries of said values.
#[derive(Debug, Clone, Copy, scpi_derive::ScpiEnum)]
pub enum NumericValueQuery {
    /// See [NumericValue::Maximum]
    #[scpi(mnemonic = b"MAXimum")]
    Maximum,
    /// See [NumericValue::Minimum]
    #[scpi(mnemonic = b"MINimum")]
    Minimum,
    /// See [NumericValue::Default]
    #[scpi(mnemonic = b"DEFault")]
    Default,
}

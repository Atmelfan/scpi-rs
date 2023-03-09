//! # Measurement Instructions
//! The purpose of the MEASure group of instructions is to acquire data using a set of high-level
//! instructions. The MEASure group of instructions have a duality, in that they exhibit
//! command and query characteristics. The exception to this is CONFigure, which has distinct
//! query and command forms. These instructions are independent of the block diagram and
//! refer to the characteristics of the signal being measured. These instructions are intended to be
//! used with the measurement <functions> referenced later in this chapter.
//!
//! The MEASure group of commands are structured to allow the user to trade off
//! interchangeability with fine control of the measurement process. MEASure? provides a
//! complete capability where the instrument is configured, a measurement taken, and results
//! returned in one operation. Often, more precise control of the measurement is required.
//! Therefore, MEASure? is complemented by providing two commands, CONFigure and
//! READ?. CONFigure performs the configuration portion of the measurement and READ?
//! performs the data acquisition, postprocessing, and data output portions of the measurement.
//! This allows the user to perform a generic configuration of the measurement through
//! CONFigure and then customize the measurement by changing particular
//! instrument-dependent functions. The READ? then completes the measurement process.
//!
//! READ?, in turn, is broken down into two additional commands INITiate\[:IMMediate\] and
//! FETCh?. INITiate\[:IMMediate\] performs the data acquisition. This command is described in
//! chapter 22. FETCh? performs the postprocessing function and returns the data. This allows
//! the user to perform several different FETCh? functions on a single set of acquired data. For
//! example, an oscilloscope can acquire measurement data that can yield many different signal
//! characteristics such as frequency or AC and DC voltages. Thus, a transient signal may be
//! captured once using a MEASure?, READ? or INITiate. A FETCh? may then be used to
//! obtain each of the different signal characteristics without reacquiring a new measurement.
//!
//! MEASure? provides the best compatibility between instruments because no knowledge of
//! the instrument is required to perform the operation. CONFigure/READ? is less compatible if
//! instrument reconfiguration is performed between the CONFigure and READ? operations.
//! This is because the reconfiguration is, by necessity, instrument-specific. FETCh? is also less
//! compatible because knowledge of the instrument is necessary to determine whether the
//! necessary information has been captured by the instrument. For example, an oscilloscope
//! can capture both rise time and pulse width in a single acquisition. Therefore, if pulse width
//! was acquired through a MEASure? command, it would be possible to FETCh? rise time.
//! However, if pulse width were measured with a counter, the rise time information might not
//! be available without performing another data acquisition. Therefore FETCh? could not be
//! used.
//!
//! Changing certain parts of an instrument’s configuration shall cause existing measurement
//! data to be invalidated. Specifically, in figure 2-1, “Generalized Model of a Programmable
//! Instrument” in chapter 2, “Instrument Model,” any reconfiguration of signal routing,
//! measurement function, signal generation and/or trigger shall cause existing readings to be
//! invalidated. For example, the sequence:
//! ```text
//! INITiate;CONFIGure:VOLTage;FETCh:VOLTage?
//! ```
//! would cause an error to be generated as the data was invalidated by the CONFigure
//! command. Reconfiguring the display or format blocks shall not have this effect.
//!
//! | KEYWORD                | PARAMETER FORM                 | NOTES          |
//! |------------------------|--------------------------------|----------------|
//! | `CONFigure:<function>` | `<parameters>[,<source_list>]` |                |
//! | `FETCh[:<function>]?`  | `<parameters>[,<source_list>]` | `[query only]` |
//! | `READ[:<function>]`    | `<parameters>[,<source_list>]` | `[query only]` |
//! | `MEASure:<function>?`  | `<parameters>[,<source_list>]` | `[query only]` |
//!

use core::marker::PhantomData;

use scpi::{
    error::{Error, Result},
    parser::expression::channel_list::ChannelList,
    tree::prelude::*,
};

use crate::{
    trigger::{abort::Abort, initiate::Initiate, Trigger},
    ScpiDevice,
};

pub trait MeasurementFunction {
    type ConfigureParameters: Copy;
    type FetchData: ResponseData;
}

pub trait Configure: Trigger {
    type Function: ResponseData;
    type FetchData: ResponseData;

    /// ## `CONFigure? [<channel_list>]`
    ///
    /// Should return a list of enabled functions for specified channels or all channels if None.
    ///
    /// NOTE: May only support one channel and return an error if channel_list is not None.
    ///
    fn configure(&mut self, channel: Option<ChannelList>) -> Result<Self::Function>;

    /// ## `FETCh? [<channel_list>]`
    ///
    ///
    fn fetch(&mut self, source_list: Option<ChannelList>) -> Result<Self::FetchData>;

    /// ## `READ? [<channel_list>]`
    ///
    ///
    fn read(&mut self, source_list: Option<ChannelList>) -> Result<Self::FetchData> {
        self.abort();
        self.initiate();
        self.fetch(source_list)
    }
}

pub struct ConfigureCommand;
impl<D> Command<D> for ConfigureCommand
where
    D: ScpiDevice + Configure,
{
    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
        mut response: scpi::tree::prelude::ResponseUnit,
    ) -> Result<()> {
        let channel: Option<ChannelList> = params.next_optional_data()?;
        let resp = device.configure(channel)?;
        response.data(resp).finish()
    }
}

pub struct FetchCommand;
impl<D> Command<D> for FetchCommand
where
    D: ScpiDevice + Configure,
{
    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
        mut response: scpi::tree::prelude::ResponseUnit,
    ) -> Result<()> {
        let channel: Option<ChannelList> = params.next_optional_data()?;
        let resp = device.fetch(channel)?;
        response.data(resp).finish()
    }
}

pub struct ReadCommand;
impl<D> Command<D> for ReadCommand
where
    D: ScpiDevice + Configure,
{
    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
        mut response: scpi::tree::prelude::ResponseUnit,
    ) -> Result<()> {
        let channel: Option<ChannelList> = params.next_optional_data()?;
        let resp = device.fetch(channel)?;
        response.data(resp).finish()
    }
}

pub trait Conf<Func>: Configure
where
    Func: MeasurementFunction,
{
    fn conf_function(
        &mut self,
        params: Func::ConfigureParameters,
        source_list: Option<ChannelList>,
    ) -> Result<()>;
}

pub struct ConfScalFuncCommand<Func> {
    _phantom: PhantomData<Func>,
}

impl<Func> ConfScalFuncCommand<Func> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<D, Func, T1, T2> Command<D> for ConfScalFuncCommand<Func>
where
    D: ScpiDevice + Conf<Func>,
    Func: MeasurementFunction<ConfigureParameters = (T1, T2)>,
    T1: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
    T2: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
{
    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
    ) -> scpi::error::Result<()> {
        let mut t1: T1 = Default::default();
        let mut t2: T2 = Default::default();

        let source_list = 'block: {
            // Try to read a parameter, exit early if it's a channel list
            t1 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };
            // Try to read a parameter, exit early if it's a channel list
            t2 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };

            // Next should be a channel list if anything
            params.next_optional_data()?
        };

        device.conf_function((t1, t2), source_list)
    }

    fn query(
        &self,
        _device: &mut D,
        _context: &mut scpi::Context,
        _params: Parameters,
        _response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        todo!()
    }
}

pub trait Fetch<Func>: Configure
where
    Func: MeasurementFunction,
{
    fn fetch_function(
        &mut self,
        params: Func::ConfigureParameters,
        source_list: Option<ChannelList>,
    ) -> Result<Func::FetchData>;
}

pub struct FetchScalFuncCommand<Func> {
    _phantom: PhantomData<Func>,
}

impl<Func> FetchScalFuncCommand<Func> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<D, Func, T1, T2> Command<D> for FetchScalFuncCommand<Func>
where
    D: ScpiDevice + Fetch<Func>,
    Func: MeasurementFunction<ConfigureParameters = (T1, T2)>,
    T1: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
    T2: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
{
    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let mut t1: T1 = Default::default();
        let mut t2: T2 = Default::default();

        let source_list = 'block: {
            // Try to read a parameter, exit early if it's a channel list
            t1 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };
            // Try to read a parameter, exit early if it's a channel list
            t2 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };

            // Next should be a channel list if anything
            params.next_optional_data()?
        };

        let data = device.fetch_function((t1, t2), source_list)?;
        response.data(data).finish()
    }
}

pub trait Read<Func>: Configure + Fetch<Func> + Abort + Initiate
where
    Func: MeasurementFunction,
{
    fn read_function(
        &mut self,
        params: Func::ConfigureParameters,
        source_list: Option<ChannelList>,
    ) -> Result<Func::FetchData> {
        self.abort();
        self.initiate();
        self.fetch_function(params, source_list)
    }
}

pub struct ReadScalFuncCommand<Func> {
    _phantom: PhantomData<Func>,
}

impl<Func> ReadScalFuncCommand<Func> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<D, Func, T1, T2> Command<D> for ReadScalFuncCommand<Func>
where
    D: ScpiDevice + Read<Func>,
    Func: MeasurementFunction<ConfigureParameters = (T1, T2)>,
    T1: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
    T2: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
{
    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let mut t1: T1 = Default::default();
        let mut t2: T2 = Default::default();

        let source_list = 'block: {
            // Try to read a parameter, exit early if it's a channel list
            t1 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };
            // Try to read a parameter, exit early if it's a channel list
            t2 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };

            // Next should be a channel list if anything
            params.next_optional_data()?
        };

        let data = device.read_function((t1, t2), source_list)?;
        response.data(data).finish()
    }
}

pub trait Measure<Func>: Configure + Conf<Func> + Read<Func> + Abort
where
    Func: MeasurementFunction,
{
    fn measure_function(
        &mut self,
        params: Func::ConfigureParameters,
        source_list: Option<ChannelList>,
    ) -> Result<Func::FetchData> {
        self.abort();
        self.conf_function(params, source_list.clone())?;
        self.read_function(params, source_list)
    }
}

pub struct MeasScalFuncCommand<Func> {
    _phantom: PhantomData<Func>,
}

impl<Func> MeasScalFuncCommand<Func> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<D, Func, T1, T2> Command<D> for MeasScalFuncCommand<Func>
where
    D: ScpiDevice + Measure<Func>,
    Func: MeasurementFunction<ConfigureParameters = (T1, T2)>,
    T1: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
    T2: for<'t> TryFrom<Token<'t>, Error = Error> + Default,
{
    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut params: Parameters,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let mut t1: T1 = Default::default();
        let mut t2: T2 = Default::default();

        let source_list = 'block: {
            // Try to read a parameter, exit early if it's a channel list
            t1 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };
            // Try to read a parameter, exit early if it's a channel list
            t2 = match params.next_optional_token()? {
                Some(tok @ Token::ExpressionProgramData(expr)) if expr.starts_with(b"@") => {
                    break 'block Some(ChannelList::try_from(tok)?)
                }
                Some(tok) => tok.try_into()?,
                None => break 'block None,
            };

            // Next should be a channel list if anything
            params.next_optional_data()?
        };

        let data = device.measure_function((t1, t2), source_list)?;
        response.data(data).finish()
    }
}

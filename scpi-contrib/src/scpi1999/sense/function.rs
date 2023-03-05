use super::Sense;
use crate::scpi1999::ScpiDevice;
use scpi::{error::Result, option::ScpiEnum, tree::prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scpi_derive::ScpiEnum)]
#[non_exhaustive]
pub enum Presentation {
    /// g
    #[scpi(mnemonic = b"XNONe")]
    XNone,
    #[scpi(mnemonic = b"XTIMe")]
    XTime,
    #[scpi(mnemonic = b"XFRequency")]
    XFrequency,
    #[scpi(mnemonic = b"XPOWer")]
    XPower,
    #[scpi(mnemonic = b"XVOLtage")]
    XVoltage,
    #[scpi(mnemonic = b"XCURrent")]
    XCurrent,
}

impl Default for Presentation {
    fn default() -> Self {
        Self::XNone
    }
}

/// A subset of SCPI 18.13.2.8
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scpi_derive::ScpiEnum)]
#[non_exhaustive]
pub enum Function {
    #[scpi(mnemonic = b"CAPacitance")]
    Capacitance,
    #[scpi(mnemonic = b"CURRent")]
    Current(CurrentFunction),
    #[scpi(mnemonic = b"DIODe")]
    Diode,
    #[scpi(mnemonic = b"FREQuency")]
    Frequency,
    #[scpi(mnemonic = b"PERiod")]
    Period,
    #[scpi(mnemonic = b"FRESistance")]
    FResistance,
    #[scpi(mnemonic = b"POWer")]
    Power(PowerFunction),
    #[scpi(mnemonic = b"RESistance")]
    Resistance,
    #[scpi(mnemonic = b"TEMPerature")]
    Temperature(TemperatureFunction),
    #[scpi(mnemonic = b"VOLTage")]
    Voltage(VoltageFunction),
}

impl From<VoltageFunction> for Function {
    fn from(v: VoltageFunction) -> Self {
        Self::Voltage(v)
    }
}

impl From<TemperatureFunction> for Function {
    fn from(v: TemperatureFunction) -> Self {
        Self::Temperature(v)
    }
}

impl From<PowerFunction> for Function {
    fn from(v: PowerFunction) -> Self {
        Self::Power(v)
    }
}

impl From<CurrentFunction> for Function {
    fn from(v: CurrentFunction) -> Self {
        Self::Current(v)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, scpi_derive::ScpiEnum)]
pub enum CurrentFunction {
    #[scpi(mnemonic = b"AC")]
    Ac,
    #[scpi(mnemonic = b"DC")]
    #[default]
    Dc,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, scpi_derive::ScpiEnum)]
pub enum PowerFunction {
    #[scpi(mnemonic = b"AC")]
    Ac,
    #[scpi(mnemonic = b"DC")]
    #[default]
    Dc,
    #[scpi(mnemonic = b"DISTortion")]
    Distortion,
    #[scpi(mnemonic = b"PSDensity")]
    PsDensity,
    #[scpi(mnemonic = b"SNDRatio")]
    SndRatio,
    #[scpi(mnemonic = b"SNR")]
    Snr,
    #[scpi(mnemonic = b"THD")]
    Thd,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, scpi_derive::ScpiEnum)]
pub enum TemperatureFunction {
    #[scpi(mnemonic = b"TC")]
    #[default]
    Thermocouple,
    #[scpi(mnemonic = b"FRTD")]
    FRtd,
    #[scpi(mnemonic = b"RTD")]
    Rtd,
    #[scpi(mnemonic = b"FTHermistor")]
    FThermistor,
    #[scpi(mnemonic = b"THERmistor")]
    Thermistor,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, scpi_derive::ScpiEnum)]
pub enum VoltageFunction {
    #[scpi(mnemonic = b"AC")]
    Ac,
    #[scpi(mnemonic = b"DC")]
    #[default]
    Dc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct SensorFunction {
    pub presentation: Presentation,
    pub function: Function,
}

impl SensorFunction {
    fn new(presentation: Presentation, function: Function) -> Self {
        Self {
            presentation,
            function,
        }
    }

    fn new_from_str(sensor_function: &[u8]) -> Option<Self> {
        let mut toks = Tokenizer::new(sensor_function).peekable();
        // Get presentation
        let presentation = if let Token::ProgramMnemonic(first) = toks.peek()?.ok()? {
            if let Some(presentation) = Presentation::from_mnemonic(first) {
                toks.next();
                presentation
            } else {
                Presentation::XNone
            }
        } else {
            return None;
        };

        // Consume seperator
        toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));

        // Get function
        let function = if let Token::ProgramMnemonic(func) = toks.next()?.ok()? {
            match Function::from_mnemonic(func)? {
                f @ Function::Current(_) => {
                    toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
                    match toks.next() {
                        // Maybe an additional specifier [DC/AC]?
                        Some(Ok(Token::ProgramMnemonic(current_func))) => {
                            CurrentFunction::from_mnemonic(current_func)
                                .map(|x| Function::from(x))
                                .unwrap_or(f)
                        }
                        // Something went wrong
                        Some(Err(_)) => return None,
                        // No additional mnemonic
                        _ => f,
                    }
                }
                f @ Function::Power(_) => {
                    toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
                    match toks.next() {
                        Some(Ok(Token::ProgramMnemonic(power_func))) => {
                            PowerFunction::from_mnemonic(power_func)
                                .map(|x| Function::from(x))
                                .unwrap_or(f)
                        }
                        Some(Err(_)) => return None,
                        _ => f,
                    }
                }
                f @ Function::Temperature(_) => {
                    toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
                    match toks.next() {
                        Some(Ok(Token::ProgramMnemonic(temp_func))) => {
                            TemperatureFunction::from_mnemonic(temp_func)
                                .map(|x| Function::from(x))
                                .unwrap_or(f)
                        }
                        Some(Err(_)) => return None,
                        _ => f,
                    }
                }
                f @ Function::Voltage(_) => {
                    toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
                    match toks.next() {
                        Some(Ok(Token::ProgramMnemonic(voltage_func))) => {
                            VoltageFunction::from_mnemonic(voltage_func)
                                .map(|x| Function::from(x))
                                .unwrap_or(f)
                        }
                        Some(Err(_)) => return None,
                        _ => f,
                    }
                }
                func => func,
            }
        } else {
            return None;
        };

        //toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
        //TODO: Input blocks, subnodes

        // No straggling tokens
        if toks.peek().is_some() {
            return None;
        }

        Some(Self::new(presentation, function))
    }
}

impl<'a> TryFrom<Token<'a>> for SensorFunction {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Self> {
        if let Token::StringProgramData(string) = value {
            if let Some(func) = Self::new_from_str(string) {
                Ok(func)
            } else {
                Err(Error::extended(
                    ErrorCode::IllegalParameterValue,
                    b"Invalid sensor function",
                ))
            }
        } else {
            Err(ErrorCode::DataTypeError.into())
        }
    }
}

impl ResponseData for SensorFunction {
    fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
        formatter.push_byte(b'\"')?;
        // Presentation layer
        match self.presentation {
            Presentation::XNone => {}
            presentation => {
                formatter.push_ascii(presentation.short_form())?;
                formatter.push_byte(b':')?;
            }
        }
        // Function
        formatter.push_ascii(self.function.short_form())?;
        match self.function {
            Function::Current(x) => {
                formatter.push_byte(b':')?;
                formatter.push_ascii(x.short_form())?;
            }
            Function::Power(x) => {
                formatter.push_byte(b':')?;
                formatter.push_ascii(x.short_form())?;
            }
            Function::Temperature(x) => {
                formatter.push_byte(b':')?;
                formatter.push_ascii(x.short_form())?;
            }
            Function::Voltage(x) => {
                formatter.push_byte(b':')?;
                formatter.push_ascii(x.short_form())?;
            }
            _ => {}
        }
        formatter.push_byte(b'\"')?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SensFuncOnCommand<const N: usize = 1>;

impl<const N: usize, D> Command<D> for SensFuncOnCommand<N>
where
    D: ScpiDevice + Sense<N>,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let sensor_func = args.data::<D::Function>()?;
        device.function_on(sensor_func)?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let sensor_func = device.get_function_on()?;
        response.data(sensor_func).finish()
    }
}

#[cfg(test)]
mod tests {
    use scpi::arrayvec::ArrayVec;

    use super::*;

    #[test]
    fn test_sensor_function() {
        let volt = SensorFunction::new_from_str(b"VOLT").unwrap();
        assert_eq!(
            volt,
            SensorFunction {
                presentation: Presentation::XNone,
                function: Function::Voltage(VoltageFunction::Dc),
            }
        );

        let volt_ac = SensorFunction::new_from_str(b"voltage:ac").unwrap();
        assert_eq!(
            volt_ac,
            SensorFunction {
                presentation: Presentation::XNone,
                function: Function::Voltage(VoltageFunction::Ac),
            }
        );

        let xnone_volt_ac = SensorFunction::new_from_str(b"xtime:voltage:ac").unwrap();
        assert_eq!(
            xnone_volt_ac,
            SensorFunction {
                presentation: Presentation::XTime,
                function: Function::Voltage(VoltageFunction::Ac),
            }
        );

        let volt_ac_ratio = SensorFunction::new_from_str(b"volt:ac:potato");
        assert_eq!(volt_ac_ratio, None);
    }

    #[test]
    fn func_response() {
        let volt_dc = SensorFunction {
            presentation: Presentation::XNone,
            function: Function::Voltage(VoltageFunction::Dc),
        };
        let mut buf = ArrayVec::<u8, 100>::new();
        volt_dc.format_response_data(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), b"\"VOLT:DC\"");

        let xtime_volt_ac = SensorFunction {
            presentation: Presentation::XTime,
            function: Function::Voltage(VoltageFunction::Ac),
        };
        let mut buf = ArrayVec::<u8, 100>::new();
        xtime_volt_ac.format_response_data(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), b"\"XTIM:VOLT:AC\"");
    }
}

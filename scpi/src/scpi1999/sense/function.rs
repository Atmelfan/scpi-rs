use super::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ScpiEnum)]
#[non_exhaustive]
pub enum Presentation {
    #[scpi(mnemonic = b"XNONe")]
    None,
    #[scpi(mnemonic = b"XTIMe")]
    Time,
    #[scpi(mnemonic = b"XFRequency")]
    Frequency,
    #[scpi(mnemonic = b"XPOWer")]
    Power,
    #[scpi(mnemonic = b"XVOLtage")]
    Voltage,
    #[scpi(mnemonic = b"XCURrent")]
    Current,
}

impl Default for Presentation {
    fn default() -> Self {
        Self::None
    }
}

/// A subset of SCPI 18.13.2.8
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ScpiEnum)]
#[non_exhaustive]
pub enum Function {
    #[scpi(mnemonic = b"CURRent")]
    Current(CurrentFunction),
    #[scpi(mnemonic = b"FREQuency")]
    Frequency,
    #[scpi(mnemonic = b"FRESistance")]
    FResistance,
    #[scpi(mnemonic = b"POWer")]
    Power(PowerFunction),
    #[scpi(mnemonic = b"RESistance")]
    Resistance,
    #[scpi(mnemonic = b"TEMPerature")]
    Temperature,
    #[scpi(mnemonic = b"VOLTage")]
    Voltage(VoltageFunction),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ScpiEnum)]
pub enum CurrentFunction {
    #[scpi(mnemonic = b"AC")]
    Ac,
    #[scpi(mnemonic = b"DC")]
    Dc,
}

impl Default for CurrentFunction {
    fn default() -> Self {
        Self::Dc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ScpiEnum)]
pub enum PowerFunction {
    #[scpi(mnemonic = b"AC")]
    Ac,
    #[scpi(mnemonic = b"DC")]
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

impl Default for PowerFunction {
    fn default() -> Self {
        Self::Dc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ScpiEnum)]
pub enum VoltageFunction {
    #[scpi(mnemonic = b"AC")]
    Ac,
    #[scpi(mnemonic = b"DC")]
    Dc,
}

impl Default for VoltageFunction {
    fn default() -> Self {
        Self::Dc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ScpiEnum)]
#[non_exhaustive]
pub enum Suffix {
    #[scpi(mnemonic = b"RATio")]
    Ratio,
    #[scpi(mnemonic = b"SUM")]
    Sum,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SensorFunction {
    pub presentation: Presentation,
    pub function: Function,
    pub suffix: Option<Suffix>,
}

impl SensorFunction {
    fn new(presentation: Presentation, function: Function, suffix: Option<Suffix>) -> Self {
        Self {
            presentation,
            function,
            suffix,
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
                Presentation::None
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
                    match toks.peek() {
                        // Maybe an additional specifier [DC/AC]?
                        Some(Ok(Token::ProgramMnemonic(current_func))) => {
                            // :DC|AC or Ratio/Sum
                            if let Some(current_func) = CurrentFunction::from_mnemonic(current_func)
                            {
                                toks.next();
                                Function::Current(current_func)
                            } else {
                                f
                            }
                        }
                        // Something went wrong
                        Some(Err(_)) => return None,
                        // No additional mnemonic
                        _ => f,
                    }
                }
                f @ Function::Power(_) => {
                    toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
                    match toks.peek() {
                        Some(Ok(Token::ProgramMnemonic(power_func))) => {
                            // :DC|AC or Ratio/Sum
                            if let Some(current_func) = PowerFunction::from_mnemonic(power_func) {
                                toks.next();
                                Function::Power(current_func)
                            } else {
                                f
                            }
                        }
                        Some(Err(_)) => return None,
                        _ => f,
                    }
                }
                f @ Function::Voltage(_) => {
                    toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));
                    match toks.peek() {
                        Some(Ok(Token::ProgramMnemonic(voltage_func))) => {
                            // :DC|AC or Ratio/Sum
                            if let Some(current_func) = VoltageFunction::from_mnemonic(voltage_func)
                            {
                                toks.next();
                                Function::Voltage(current_func)
                            } else {
                                f
                            }
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

        toks.next_if(|t| t.ok() == Some(Token::HeaderMnemonicSeparator));

        // Get suffix
        let suffix = match toks.peek() {
            Some(Ok(Token::ProgramMnemonic(suffix))) => {
                let suffix = Suffix::from_mnemonic(suffix)?;
                toks.next();
                Some(suffix)
            }
            Some(Err(_)) => return None,
            _ => None,
        };

        //TODO: Input blocks, subnodes

        // No straggling tokens
        if toks.peek().is_some() {
            return None;
        }

        Some(Self::new(presentation, function, suffix))
    }
}

impl<'a> TryFrom<Token<'a>> for SensorFunction {
    type Error = Error;

    fn try_from(value: Token<'a>) -> Result<Self, Self::Error> {
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
    fn format_response_data(
        &self,
        formatter: &mut dyn scpi::response::Formatter,
    ) -> scpi::error::Result<()> {
        formatter.push_byte(b'\"')?;
        // Presentation layer
        match self.presentation {
            Presentation::None => {}
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
            Function::Voltage(x) => {
                formatter.push_byte(b':')?;
                formatter.push_ascii(x.short_form())?;
            }
            _ => {}
        }
        // Suffix
        if let Some(suffix) = self.suffix {
            formatter.push_byte(b':')?;
            formatter.push_ascii(suffix.short_form())?;
        }
        formatter.push_byte(b'\"')?;
        Ok(())
    }
}

#[derive(Debug)]
struct SensFuncOn<const N: usize>;

impl<const N: usize, D> Command<D> for SensFuncOn<N>
where
    D: ScpiDevice + Sense<N>,
{
    fn meta(&self) -> scpi::command::CommandTypeMeta {
        scpi::command::CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: scpi::parameters::Arguments,
    ) -> scpi::error::Result<()> {
        let sensor_func = args.next::<SensorFunction>()?;
        device.function_on(sensor_func)?;
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
    ) -> scpi::error::Result<()> {
        let sensor_func = device.get_function_on()?;
        response.data(sensor_func).finish()
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;

    use super::*;

    #[test]
    fn test_sensor_function() {
        let volt = SensorFunction::new_from_str(b"VOLT").unwrap();
        assert_eq!(
            volt,
            SensorFunction {
                presentation: Presentation::None,
                function: Function::Voltage(VoltageFunction::Dc),
                suffix: None
            }
        );

        let volt_ac = SensorFunction::new_from_str(b"voltage:ac").unwrap();
        assert_eq!(
            volt_ac,
            SensorFunction {
                presentation: Presentation::None,
                function: Function::Voltage(VoltageFunction::Ac),
                suffix: None
            }
        );

        let xnone_volt_ac = SensorFunction::new_from_str(b"xtime:voltage:ac").unwrap();
        assert_eq!(
            xnone_volt_ac,
            SensorFunction {
                presentation: Presentation::Time,
                function: Function::Voltage(VoltageFunction::Ac),
                suffix: None
            }
        );

        let volt_ratio = SensorFunction::new_from_str(b"voltage:ratio").unwrap();
        assert_eq!(
            volt_ratio,
            SensorFunction {
                presentation: Presentation::None,
                function: Function::Voltage(VoltageFunction::Dc),
                suffix: Some(Suffix::Ratio)
            }
        );

        let volt_ac_ratio = SensorFunction::new_from_str(b"volt:ac:sum").unwrap();
        assert_eq!(
            volt_ac_ratio,
            SensorFunction {
                presentation: Presentation::None,
                function: Function::Voltage(VoltageFunction::Ac),
                suffix: Some(Suffix::Sum)
            }
        );

        let volt_ac_ratio = SensorFunction::new_from_str(b"volt:ac:potato");
        assert_eq!(volt_ac_ratio, None);
    }

    #[test]
    fn func_response() {
        let volt_dc = SensorFunction {
            presentation: Presentation::None,
            function: Function::Voltage(VoltageFunction::Dc),
            suffix: None,
        };
        let mut buf = ArrayVec::<u8, 100>::new();
        volt_dc.format_response_data(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), b"\"VOLT:DC\"");

        let volt_ac_ratio = SensorFunction {
            presentation: Presentation::Time,
            function: Function::Voltage(VoltageFunction::Ac),
            suffix: Some(Suffix::Ratio),
        };
        let mut buf = ArrayVec::<u8, 100>::new();
        volt_ac_ratio.format_response_data(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), b"\"XTIM:VOLT:AC:RAT\"");
    }
}

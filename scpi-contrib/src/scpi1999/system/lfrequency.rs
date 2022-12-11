use scpi::{
    error::Result,
    tree::prelude::*,
};

use crate::{scpi1999::{numeric::NumericValue, util::Auto, ScpiDevice}, NumericValueQuery};

use scpi::units::Frequency;

pub trait LineFrequency {
    /// Get maximum line frequency
    fn max_line_freq(&self) -> Frequency;

    /// Get minimum line frequency
    fn min_line_freq(&self) -> Frequency;

    /// Set manual line frequency
    ///
    /// **Note** This should disable any automatic mechanism if implemented
    fn line_frequency(&mut self, freq: Frequency) -> Result<()>;

    /// Get currently set line frequency
    fn get_line_frequency(&self) -> Frequency;
}

pub trait LineFrequencyAuto: LineFrequency {
    /// Set automatic line frequency mode.
    fn auto(&mut self, auto: Auto);

    /// Get automatic line frequency mode
    fn get_auto(&self) -> Auto;
}

pub struct SystLfrequencyCommand;

impl<D> Command<D> for SystLfrequencyCommand
where
    D: ScpiDevice + LineFrequency,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let freq: NumericValue<Frequency> = args.data()?;
        // Set frequency or enable auto once as default
        match freq {
            NumericValue::Value(freq) => device.line_frequency(freq),
            NumericValue::Maximum => device.line_frequency(device.max_line_freq()),
            NumericValue::Minimum => device.line_frequency(device.min_line_freq()),
            _ => Err(ErrorCode::IllegalParameterValue.into()),
        }
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let freq: Frequency = match args.optional_data::<NumericValueQuery>()? {
            Some(NumericValueQuery::Maximum) => device.max_line_freq(),
            Some(NumericValueQuery::Minimum) => device.min_line_freq(),
            None => device.get_line_frequency(),
            _ => return Err(ErrorCode::IllegalParameterValue.into()),
        };
        response.data(freq).finish()
    }
}

pub struct SystLfrAutoCommand;

impl<D> Command<D> for SystLfrAutoCommand
where
    D: ScpiDevice + LineFrequency + LineFrequencyAuto,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(&self, device: &mut D, _context: &mut Context, mut args: Arguments) -> Result<()> {
        let auto: Auto = args.data()?;
        device.auto(auto);
        Ok(())
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        response.data(device.get_auto()).finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    impl LineFrequency for () {
        fn max_line_freq(&self) -> Frequency {
            unimplemented!()
        }

        fn min_line_freq(&self) -> Frequency {
            unimplemented!()
        }

        fn line_frequency(&mut self, _freq: Frequency) -> Result<()> {
            unimplemented!()
        }

        fn get_line_frequency(&self) -> Frequency {
            unimplemented!()
        }
    }
}

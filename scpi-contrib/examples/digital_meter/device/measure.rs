use scpi::{
    error::Result,
    parser::expression::channel_list::ChannelList,
    tree::prelude::*,
    units::{uom::si::electric_potential::volt, ElectricPotential},
};
use scpi_contrib::{
    measurement::{Conf, Configure, Fetch, Measure, MeasurementFunction, Read},
    NumericValue,
};

use crate::device::FakeSensorMode;

use super::{util::Auto, ScalVoltAc, Voltmeter};

#[derive(Debug, Clone, Copy)]
pub enum MeasFunction {
    VoltageAc {
        expected_value: Auto<f32>,
        resolution: f32,
    },
    VoltageDc {
        expected_value: Auto<f32>,
        resolution: f32,
    },
}

impl Default for MeasFunction {
    fn default() -> Self {
        Self::VoltageDc {
            expected_value: Auto::Auto,
            resolution: 1.0,
        }
    }
}

impl MeasFunction {
    fn short(&self) -> &'static [u8] {
        match self {
            MeasFunction::VoltageAc { .. } => b"VOLT:AC",
            MeasFunction::VoltageDc { .. } => b"VOLT:DC",
        }
    }
}

impl ResponseData for MeasFunction {
    fn format_response_data(
        &self,
        formatter: &mut dyn scpi::tree::prelude::Formatter,
    ) -> scpi::error::Result<()> {
        match self {
            MeasFunction::VoltageAc {
                expected_value,
                resolution,
            }
            | MeasFunction::VoltageDc {
                expected_value,
                resolution,
            } => {
                formatter.push_byte(b'"')?;
                formatter.push_ascii(self.short())?;
                formatter.push_byte(b' ')?;
                expected_value.format_response_data(formatter)?;
                formatter.push_byte(b',')?;
                resolution.format_response_data(formatter)?;
                formatter.push_byte(b'"')
            }
        }
    }
}

impl Configure for Voltmeter {
    type Function = MeasFunction;
    type FetchData = Vec<f32>;

    fn configure(&mut self, channel: Option<ChannelList>) -> scpi::error::Result<Self::Function> {
        // Don't have channels
        if channel.is_some() {
            return Err(ErrorCode::ParameterNotAllowed.into());
        }
        Ok(self.measfunc)
    }

    fn fetch(&mut self, source_list: Option<ChannelList>) -> scpi::error::Result<Self::FetchData> {
        // Don't have channels
        if source_list.is_some() {
            return Err(ErrorCode::ParameterNotAllowed.into());
        }
        self.measurement
            .clone()
            .ok_or_else(|| Error::new(ErrorCode::DataCorruptOrStale))
    }
}

impl MeasurementFunction for ScalVoltAc {
    type ConfigureParameters = (
        NumericValue<Auto<ElectricPotential>>,
        NumericValue<ElectricPotential>,
    );
    type FetchData = Vec<f32>;
}

impl Conf<ScalVoltAc> for Voltmeter {
    fn conf_function(
        &mut self,
        (expected_value, resolution): <ScalVoltAc as MeasurementFunction>::ConfigureParameters,
        _source_list: Option<ChannelList>,
    ) -> Result<()> {
        //dbg!(expected_value, resolution);

        self.sensor.mode = FakeSensorMode::VoltageAc;
        self.measfunc = MeasFunction::VoltageAc {
            expected_value: match expected_value {
                NumericValue::Value(Auto::Auto) => Auto::Auto,
                NumericValue::Value(Auto::Numeric(x)) => Auto::Numeric(x.value),
                NumericValue::Maximum => Auto::Numeric(100.0),
                NumericValue::Minimum => Auto::Numeric(0.01),
                NumericValue::Default => Auto::Auto,
                _ => return Err(ErrorCode::IllegalParameterValue.into()),
            },
            resolution: resolution
                .build()
                .default(ElectricPotential::new::<volt>(0.001))
                .finish()?
                .value,
        };
        Ok(())
    }
}
impl Fetch<ScalVoltAc> for Voltmeter {
    fn fetch_function(
        &mut self,
        _params: <ScalVoltAc as MeasurementFunction>::ConfigureParameters,
        _source_list: Option<ChannelList>,
    ) -> Result<<ScalVoltAc as MeasurementFunction>::FetchData> {
        // Return measurment value if valid and function is the same
        if matches!(self.measfunc, MeasFunction::VoltageAc { .. }) {
            self.measurement
                .clone()
                .ok_or_else(|| Error::new(ErrorCode::DataCorruptOrStale))
        } else {
            Err(ErrorCode::SettingsConflict.into())
        }
    }
}

// Default
impl Read<ScalVoltAc> for Voltmeter {}
impl Measure<ScalVoltAc> for Voltmeter {}

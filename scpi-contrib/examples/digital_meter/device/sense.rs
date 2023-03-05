use scpi::{
    prelude::ErrorCode,
    units::{uom::si::electric_potential::volt, ElectricPotential},
};
use scpi_contrib::sense::{
    common::{SenseRange, SenseResolution},
    function::{Function, Presentation, SensorFunction, VoltageFunction},
    FunctionError, Sens, Sense, SenseFunction,
};

use super::{FakeSensorMode, FakeSensorRange, ScalVoltAc};

impl Sense for super::Voltmeter {
    type Function = SensorFunction;

    fn function_on(&mut self, function: SensorFunction) -> Result<(), FunctionError> {
        // Error on any presentation/suffix other than XNone/No suffix
        if !matches!(function.presentation, Presentation::XNone) {
            return Err(FunctionError::PresentationNotSupported);
        }

        // Enable function
        self.sensor.mode = match function.function {
            Function::Voltage(VoltageFunction::Ac) => FakeSensorMode::VoltageAc,
            Function::Voltage(VoltageFunction::Dc) => FakeSensorMode::VoltageDc,

            // Other functions not supported
            _ => return Err(FunctionError::FunctionNotSupported),
        };

        Ok(())
    }

    fn get_function_on(&self) -> Result<SensorFunction, FunctionError> {
        Ok(SensorFunction {
            presentation: Presentation::XNone,
            function: match self.sensor.mode {
                FakeSensorMode::VoltageAc => Function::Voltage(VoltageFunction::Ac),
                FakeSensorMode::VoltageDc => Function::Voltage(VoltageFunction::Dc),
            },
        })
    }
}

impl Sens<ScalVoltAc> for super::Voltmeter {}

impl SenseFunction for ScalVoltAc {
    type Unit = scpi::units::ElectricPotential;
}

impl SenseRange<ScalVoltAc> for super::Voltmeter {
    fn range_upper(
        &mut self,
        upper: scpi_contrib::NumericValue<<ScalVoltAc as SenseFunction>::Unit>,
    ) -> scpi::error::Result<()> {
        let upper = upper
            .build()
            .max(ElectricPotential::new::<volt>(ScalVoltAc::MAX_RANGE))
            .min(ElectricPotential::new::<volt>(ScalVoltAc::MIN_RANGE))
            .finish()?
            .get::<volt>();

        // Set best sensor mode based on desired range
        self.sensor.range_upper = match upper {
            v if v > ScalVoltAc::MIN_RANGE && v <= 0.1 => FakeSensorRange::V0_1,
            v if v > 0.1 && v <= 1.0 => FakeSensorRange::V1,
            v if v > 1.0 && v <= 10.0 => FakeSensorRange::V10,
            v if v > 10.0 && v <= ScalVoltAc::MAX_RANGE => FakeSensorRange::V100,
            _ => return Err(ErrorCode::DataOutOfRange.into()),
        };

        Ok(())
    }

    fn get_range_upper(&self) -> <ScalVoltAc as SenseFunction>::Unit {
        match self.sensor.range_upper {
            FakeSensorRange::V0_1 => ElectricPotential::new::<volt>(0.1),
            FakeSensorRange::V1 => ElectricPotential::new::<volt>(1.0),
            FakeSensorRange::V10 => ElectricPotential::new::<volt>(10.0),
            FakeSensorRange::V100 => ElectricPotential::new::<volt>(100.0),
        }
    }

    fn range_lower(
        &mut self,
        _lower: scpi_contrib::NumericValue<<ScalVoltAc as SenseFunction>::Unit>,
    ) -> scpi::error::Result<()> {
        unimplemented!()
    }

    fn get_range_lower(&self) -> <ScalVoltAc as SenseFunction>::Unit {
        unimplemented!()
    }

    fn auto(&mut self, auto: scpi_contrib::util::Auto) -> scpi::error::Result<()> {
        self.sensor.auto = auto;
        Ok(())
    }

    fn get_auto(&self) -> scpi_contrib::util::Auto {
        self.sensor.auto
    }
}

impl SenseResolution<ScalVoltAc> for super::Voltmeter {
    fn resolution(
        &mut self,
        _upper: scpi_contrib::NumericValue<<ScalVoltAc as SenseFunction>::Unit>,
    ) -> scpi::error::Result<()> {
        // TODO
        Ok(())
    }

    fn get_resolution(&self) -> <ScalVoltAc as SenseFunction>::Unit {
        ElectricPotential::new::<volt>(0.001)
    }
}

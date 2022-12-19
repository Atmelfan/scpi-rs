use scpi_contrib::sense::{
    function::{Function, Presentation, SensorFunction, VoltageFunction},
    FunctionError, Sense,
};

use super::FakeSensorMode;

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

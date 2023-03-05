use crate::{
    measurement::{Measure, MeasurementFunction},
    sense::{Sens, SenseFunction},
    trigger::Trigger,
    ScpiDevice,
};

/// `<meter_fn>`
pub trait MeterFn {
    const KEYWORD: &'static [u8];
}

macro_rules! meter_fn {
    ($fn:ident; $bf:literal) => {
        pub struct $fn;
        impl MeterFn for $fn {
            const KEYWORD: &'static [u8] = $bf;
        }
    };
}
meter_fn!(ScalVoltAc; b"DCVOLTMETER");
meter_fn!(ScalVoltDc; b"ACVOLTMETER");
meter_fn!(ScalCurrAc; b"DCAMMETER");
meter_fn!(ScalCurrDc; b"ACAMMETER");
meter_fn!(ScalResistance; b"OHMMETER");
meter_fn!(ScalFResistance; b"FOHMMETER");

pub trait DigitalMeter<Func>: ScpiDevice + Trigger + Sens<Func> + Measure<Func>
where
    Func: MeasurementFunction + SenseFunction,
{
}

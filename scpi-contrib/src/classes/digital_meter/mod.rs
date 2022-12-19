use crate::{trigger::Trigger, sense::Sense, ScpiDevice, measurement::{Measure, MeasurementFunction}};

/// `<meter_fn>`
trait MeterFn {}

macro_rules! meter_fn {
    ($fn:ident; $bf:literal) => {
        pub struct $fn;
    };
}
meter_fn!(VoltageAcFn; b"DCVOLTMETER");
meter_fn!(VoltageDcFn; b"ACVOLTMETER");
meter_fn!(CurrentAcFn; b"DCAMMETER");
meter_fn!(CurrentDcFn; b"ACAMMETER");
meter_fn!(ResistanceFn; b"OHMMETER");
meter_fn!(FResistanceFn; b"FOHMMETER");

pub trait DigitalMeter<Func>: ScpiDevice + Trigger + Sense + Measure<Func> where Func: MeasurementFunction {
    
}


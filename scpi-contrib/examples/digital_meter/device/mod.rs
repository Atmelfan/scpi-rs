use rand::Rng;
use scpi::{
    units::{uom::si::electric_potential::volt, ElectricPotential},
    Device,
};
use scpi_contrib::{
    classes::digital_meter::DigitalMeter,
    scpi1999::prelude::*,
    trigger::{TriggerSource, TriggerState},
    util::Auto,
    IEEE4882,
};

use std::collections::VecDeque;

pub(crate) mod util;

pub(crate) mod measure;
pub(crate) mod sense;
pub(crate) mod trigger;

/// Fake sensor. This would be the ADC with range switches etc in a real device
#[derive(Debug, Default)]
pub(crate) enum FakeSensorMode {
    /// Take multiple measurements and calculate RMS
    VoltageAc,
    /// Take one or more measurements and calculate average
    #[default]
    VoltageDc,
}

#[derive(Debug, Default)]
pub(crate) struct FakeSensor {
    pub(crate) mode: FakeSensorMode,
    pub range_upper: FakeSensorRange,
    pub auto: Auto,
}

impl FakeSensor {
    // Take one measurement of whatever function is configured
    fn sense(&self) -> f32 {
        match self.mode {
            FakeSensorMode::VoltageAc => rand::thread_rng().gen_range(0.0..10.0),
            FakeSensorMode::VoltageDc => rand::thread_rng().gen_range(-10.0..10.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum FakeSensorRange {
    V0_1,
    #[default]
    V1,
    V10,
    V100,
}

// Measurement functions
pub struct ScalVoltAc;

impl ScalVoltAc {
    pub(crate) const MAX_RANGE: f32 = 100.0;
    pub(crate) const MIN_RANGE: f32 = 0.1;
}
pub struct ScalVoltDc;

#[derive(Debug, Default)]
pub(crate) struct Voltmeter {
    /// # Mandatory IEEE488/SCPI registers
    /// Event Status Register
    pub esr: u8,
    /// Event Status Enable register
    pub ese: u8,
    /// Service Request Enable register
    pub sre: u8,
    /// OPERation:ENABle register
    pub operation: EventRegister,
    /// QUEStionable:ENABle register
    pub questionable: EventRegister,
    /// Error queue
    pub errors: VecDeque<Error>,

    // Trigger
    pub trigger_src: TriggerSource,
    pub trigger_state: TriggerState,
    pub trigger_cnt: usize,

    // Measurment config
    pub measfunc: measure::MeasFunction,
    pub sensor: FakeSensor,

    // measurment buffer
    pub measurement: Option<Vec<f32>>,
}

impl Voltmeter {
    pub(crate) fn new() -> Self {
        Voltmeter {
            esr: 0,
            ese: 0,
            sre: 0,
            operation: Default::default(),
            questionable: Default::default(),
            errors: Default::default(),
            trigger_src: Default::default(),
            measfunc: Default::default(),
            sensor: Default::default(),
            measurement: None,
            trigger_state: Default::default(),
            trigger_cnt: 1,
        }
    }
}

impl DigitalMeter<ScalVoltAc> for Voltmeter {}

impl ScpiDevice for Voltmeter {}

impl Device for Voltmeter {
    fn handle_error(&mut self, err: Error) {
        self.push_error(err);
    }
}

impl IEEE4882 for Voltmeter {
    fn stb(&self) -> u8 {
        0x00
    }

    fn sre(&self) -> u8 {
        self.sre
    }

    fn set_sre(&mut self, value: u8) {
        self.sre = value;
    }

    fn esr(&self) -> u8 {
        self.esr
    }

    fn set_esr(&mut self, value: u8) {
        self.esr = value;
    }

    fn ese(&self) -> u8 {
        self.ese
    }

    fn set_ese(&mut self, value: u8) {
        self.ese = value;
    }

    fn tst(&mut self) -> scpi::error::Result<()> {
        Ok(())
    }

    fn rst(&mut self) -> scpi::error::Result<()> {
        self.trigger_cnt = 1;
        self.trigger_src = Default::default();
        self.trigger_state = Default::default();
        self.measfunc = Default::default();
        self.sensor = Default::default();

        self.measurement = None;
        Ok(())
    }

    fn cls(&mut self) -> scpi::error::Result<()> {
        self.cls_standard()
    }

    fn opc(&mut self) -> scpi::error::Result<()> {
        self.opc_standard()
    }
}

impl ErrorQueue for Voltmeter {
    fn push_back_error(&mut self, err: Error) {
        self.errors.push_back(err);
    }

    fn pop_front_error(&mut self) -> Option<Error> {
        self.errors.pop_front()
    }

    fn num_errors(&self) -> usize {
        self.errors.len()
    }

    fn clear_errors(&mut self) {
        self.errors.clear()
    }
}

impl GetEventRegister<Questionable> for Voltmeter {
    fn register(&self) -> &EventRegister {
        &self.questionable
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.questionable
    }
}

impl GetEventRegister<Operation> for Voltmeter {
    fn register(&self) -> &EventRegister {
        &self.operation
    }

    fn register_mut(&mut self) -> &mut EventRegister {
        &mut self.operation
    }
}

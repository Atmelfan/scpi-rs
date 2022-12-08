use super::{common::*, *};

pub struct VoltageDc;
impl SenseFunction for VoltageDc {
    type Unit = uom::si::f32::ElectricPotential;
}

pub type SensVoltDcRangUpp<const N: usize = 1> = SenseRangeUpperCommand<VoltageDc, N>;
pub type SensVoltDcRangLow<const N: usize = 1> = SenseRangeLowerCommand<VoltageDc, N>;
pub type SensVoltDcRangAuto<const N: usize = 1> = SenseRangeAutoCommand<VoltageDc, N>;
pub type SensVoltDcRes<const N: usize = 1> = SenseResolutionCommand<VoltageDc, N>;
pub type SensVoltDcAper<const N: usize = 1> = SenseApertureCommand<VoltageDc, N>;
pub type SensVoltDcNplc<const N: usize = 1> = SenseNPLCyclesCommand<VoltageDc, N>;

pub struct VoltageAc;
impl SenseFunction for VoltageAc {
    type Unit = uom::si::f32::ElectricPotential;
}

pub type SensVoltAcRangUpp<const N: usize = 1> = SenseRangeUpperCommand<VoltageAc, N>;
pub type SensVoltAcRangLow<const N: usize = 1> = SenseRangeLowerCommand<VoltageAc, N>;
pub type SensVoltAcRangAuto<const N: usize = 1> = SenseRangeAutoCommand<VoltageAc, N>;
pub type SensVoltAcRes<const N: usize = 1> = SenseResolutionCommand<VoltageAc, N>;
pub type SensVoltAcAper<const N: usize = 1> = SenseApertureCommand<VoltageAc, N>;
pub type SensVoltAcNplc<const N: usize = 1> = SenseNPLCyclesCommand<VoltageAc, N>;

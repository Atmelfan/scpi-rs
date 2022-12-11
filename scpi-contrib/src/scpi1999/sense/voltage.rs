use super::{common::*, *};

pub struct VoltageDc;
impl SenseFunction for VoltageDc {
    type Unit = uom::si::f32::ElectricPotential;
}

pub type SensVoltDcRangeUpper<const N: usize = 1> = SensRangUpperCommand<VoltageDc, N>;
pub type SensVoltDcRangeLower<const N: usize = 1> = SensRangLowerCommand<VoltageDc, N>;
pub type SensVoltDcRangAuto<const N: usize = 1> = SensRangAutoCommand<VoltageDc, N>;
pub type SensVoltDcResolution<const N: usize = 1> = SensResolutionCommand<VoltageDc, N>;
#[cfg(feature = "unit-time")]
pub type SensVoltDcAperture<const N: usize = 1> = aperture::SensApertureCommand<VoltageDc, N>;
#[cfg(feature = "unit-ratio")]
pub type SensVoltDcNPLCycles<const N: usize = 1> = nplc::SensNPLCyclesCommand<VoltageDc, N>;

pub struct VoltageAc;
impl SenseFunction for VoltageAc {
    type Unit = uom::si::f32::ElectricPotential;
}

pub type SensVoltAcRangeUpper<const N: usize = 1> = SensRangUpperCommand<VoltageAc, N>;
pub type SensVoltAcRangeLower<const N: usize = 1> = SensRangLowerCommand<VoltageAc, N>;
pub type SensVoltAcRangAuto<const N: usize = 1> = SensRangAutoCommand<VoltageAc, N>;
pub type SensVoltAcResolution<const N: usize = 1> = SensResolutionCommand<VoltageAc, N>;
#[cfg(feature = "unit-time")]
pub type SensVoltAcAperture<const N: usize = 1> = aperture::SensApertureCommand<VoltageAc, N>;
#[cfg(feature = "unit-ratio")]
pub type SensVoltAcNPLCycles<const N: usize = 1> = nplc::SensNPLCyclesCommand<VoltageAc, N>;

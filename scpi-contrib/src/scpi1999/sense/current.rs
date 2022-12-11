use super::{common::*, *};

pub struct CurrentDc;
impl SenseFunction for CurrentDc {
    type Unit = uom::si::f32::ElectricCurrent;
}

pub type SensCurrDcRangUpper<const N: usize = 1> = SensRangUpperCommand<CurrentDc, N>;
pub type SensCurrDcRangLower<const N: usize = 1> = SensRangLowerCommand<CurrentDc, N>;
pub type SensCurrDcRangAuto<const N: usize = 1> = SensRangAutoCommand<CurrentDc, N>;
pub type SensCurrDcResolution<const N: usize = 1> = SensResolutionCommand<CurrentDc, N>;
#[cfg(feature = "unit-time")]
pub type SensCurrDcAperture<const N: usize = 1> = aperture::SensApertureCommand<CurrentDc, N>;
#[cfg(feature = "unit-ratio")]
pub type SensCurrDcNPLCycles<const N: usize = 1> = nplc::SensNPLCyclesCommand<CurrentDc, N>;

pub struct CurrentAc;
impl SenseFunction for CurrentAc {
    type Unit = uom::si::f32::ElectricCurrent;
}

pub type SensCurrAcRangUpper<const N: usize = 1> = SensRangUpperCommand<CurrentAc, N>;
pub type SensCurrAcRangLower<const N: usize = 1> = SensRangLowerCommand<CurrentAc, N>;
pub type SensCurrAcRangAuto<const N: usize = 1> = SensRangAutoCommand<CurrentAc, N>;
pub type SensCurrAcResolution<const N: usize = 1> = SensResolutionCommand<CurrentAc, N>;
#[cfg(feature = "unit-time")]
pub type SensCurrAcAperture<const N: usize = 1> = aperture::SensApertureCommand<CurrentAc, N>;
#[cfg(feature = "unit-ratio")]
pub type SensCurrAcNPLCycles<const N: usize = 1> = nplc::SensNPLCyclesCommand<CurrentAc, N>;

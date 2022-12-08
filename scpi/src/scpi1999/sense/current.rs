use super::{common::*, *};

pub struct CurrentDc;
impl SenseFunction for CurrentDc {
    type Unit = uom::si::f32::ElectricCurrent;
}

pub type SensCurrDcRangUpp<const N: usize = 1> = SenseRangeUpperCommand<CurrentDc, N>;
pub type SensCurrDcRangLow<const N: usize = 1> = SenseRangeLowerCommand<CurrentDc, N>;
pub type SensCurrDcRangAuto<const N: usize = 1> = SenseRangeAutoCommand<CurrentDc, N>;
pub type SensCurrDcRes<const N: usize = 1> = SenseResolutionCommand<CurrentDc, N>;
pub type SensCurrDcAper<const N: usize = 1> = SenseApertureCommand<CurrentDc, N>;
pub type SensCurrDcNplc<const N: usize = 1> = SenseNPLCyclesCommand<CurrentDc, N>;

pub struct CurrentAc;
impl SenseFunction for CurrentAc {
    type Unit = uom::si::f32::ElectricCurrent;
}

pub type SensCurrAcRangUpp<const N: usize = 1> = SenseRangeUpperCommand<CurrentAc, N>;
pub type SensCurrAcRangLow<const N: usize = 1> = SenseRangeLowerCommand<CurrentAc, N>;
pub type SensCurrAcRangAuto<const N: usize = 1> = SenseRangeAutoCommand<CurrentAc, N>;
pub type SensCurrAcRes<const N: usize = 1> = SenseResolutionCommand<CurrentAc, N>;
pub type SensCurrAcAper<const N: usize = 1> = SenseApertureCommand<CurrentAc, N>;
pub type SensCurrAcNplc<const N: usize = 1> = SenseNPLCyclesCommand<CurrentAc, N>;

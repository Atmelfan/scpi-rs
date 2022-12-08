use super::{common::*, *};

pub struct Resistance;
impl SenseFunction for Resistance {
    type Unit = uom::si::f32::ElectricalResistance;
}

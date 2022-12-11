use super::*;

pub struct Resistance;
impl SenseFunction for Resistance {
    type Unit = uom::si::f32::ElectricPotential;
}

pub struct FResistance;
impl SenseFunction for FResistance {
    type Unit = uom::si::f32::ElectricPotential;
}

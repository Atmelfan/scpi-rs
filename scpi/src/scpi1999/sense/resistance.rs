use super::*;

pub struct Resistance;
impl SenseFunction for Resistance {
    #[cfg(feature = "unit-electric-resistance")]
    type Unit = uom::si::f32::ElectricPotential;

    #[cfg(not(feature = "unit-electric-resistance"))]
    type Unit = f32;
}

pub struct FResistance;
impl SenseFunction for FResistance {
    #[cfg(feature = "unit-electric-resistance")]
    type Unit = uom::si::f32::ElectricPotential;

    #[cfg(not(feature = "unit-electric-resistance"))]
    type Unit = f32;
}

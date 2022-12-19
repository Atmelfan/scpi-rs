use core::marker::PhantomData;

use scpi::{
    error::Result,
    tree::prelude::*,
};

use crate::scpi1999::{numeric::NumericValue, util::Auto};

use super::*;

pub trait SenseRange<FUNC, const N: usize = 1>: Sense<N>
where
    FUNC: SenseFunction,
{
    ///
    fn range_upper(&mut self, upper: NumericValue<FUNC::Unit>) -> Result<()>;
    fn get_range_upper(&self) -> FUNC::Unit;

    ///
    fn range_lower(&mut self, lower: NumericValue<FUNC::Unit>) -> Result<()>;
    fn get_range_lower(&self) -> FUNC::Unit;

    /// Enable/disable auto-ranging
    fn auto(&mut self, auto: Auto) -> Result<()>;
    fn get_auto(&self) -> Auto;
}

pub struct SensRangUpperCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SensRangUpperCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseRange<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: Arguments,
    ) -> scpi::error::Result<()> {
        let upper = args.data::<NumericValue<FUNC::Unit>>()?;
        device.range_upper(upper)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let upper = device.get_range_upper();
        response.data(upper).finish()
    }
}

pub struct SensRangLowerCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SensRangLowerCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseRange<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: Arguments,
    ) -> scpi::error::Result<()> {
        let upper = args.data::<NumericValue<FUNC::Unit>>()?;
        device.range_lower(upper)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let upper = device.get_range_lower();
        response.data(upper).finish()
    }
}

pub struct SensRangAutoCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SensRangAutoCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseRange<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: Arguments,
    ) -> Result<()> {
        let auto = args.data::<Auto>()?;
        device.auto(auto)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let auto = device.get_auto();
        response.data(auto).finish()
    }
}

trait SenseResolution<FUNC, const N: usize = 1>: Sense<N>
where
    FUNC: SenseFunction,
{
    fn resolution(&mut self, upper: NumericValue<FUNC::Unit>) -> Result<()>;
    fn get_resolution(&self) -> FUNC::Unit;
}

pub struct SensResolutionCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SensResolutionCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseResolution<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> CommandTypeMeta {
        CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: Arguments,
    ) -> scpi::error::Result<()> {
        let resolution = args.data::<NumericValue<FUNC::Unit>>()?;
        device.resolution(resolution)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: Arguments,
        mut response: ResponseUnit,
    ) -> scpi::error::Result<()> {
        let resolution = device.get_resolution();
        response.data(resolution).finish()
    }
}

#[cfg(feature = "unit-time")]
pub(crate) mod aperture {
    use super::*;
    use uom::si::f32::Time;

    pub trait SenseAperture<FUNC, const N: usize = 1>: Sense<N>
    where
        FUNC: SenseFunction,
    {
        ///
        fn aperture(&mut self, aperture: NumericValue<Time>) -> Result<()>;
        fn get_aperture(&self) -> Time;
    }

    pub struct SensApertureCommand<FUNC, const N: usize = 1> {
        _phantom: PhantomData<FUNC>,
    }

    impl<FUNC, const N: usize> SensApertureCommand<FUNC, N> {
        pub fn new() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl<D, FUNC, const N: usize> Command<D> for SensApertureCommand<FUNC, N>
    where
        D: ScpiDevice + Sense<N> + SenseAperture<FUNC, N>,
        FUNC: SenseFunction,
    {
        fn meta(&self) -> CommandTypeMeta {
            CommandTypeMeta::Both
        }

        fn event(
            &self,
            device: &mut D,
            _context: &mut scpi::Context,
            mut args: Arguments,
        ) -> scpi::error::Result<()> {
            let aperture: NumericValue<Time> = args.data::<NumericValue<Time>>()?;
            device.aperture(aperture)
        }

        fn query(
            &self,
            device: &mut D,
            _context: &mut scpi::Context,
            _args: Arguments,
            mut response: ResponseUnit,
        ) -> scpi::error::Result<()> {
            let aperture: Time = device.get_aperture();
            response.data(aperture).finish()
        }
    }
}

#[cfg(feature = "unit-ratio")]
pub(crate) mod nplc {
    use super::*;
    use uom::si::f32::Ratio;

    pub trait SenseNPLCycles<FUNC, const N: usize = 1>: Sense<N>
    where
        FUNC: SenseFunction,
    {
        ///
        fn nplc(&mut self, aperture: NumericValue<Ratio>) -> Result<()>;
        fn get_nplc(&self) -> Ratio;
    }

    pub struct SensNPLCyclesCommand<FUNC, const N: usize = 1> {
        _phantom: PhantomData<FUNC>,
    }

    impl<FUNC, const N: usize> SensNPLCyclesCommand<FUNC, N> {
        pub fn new() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl<D, FUNC, const N: usize> Command<D> for SensNPLCyclesCommand<FUNC, N>
    where
        D: ScpiDevice + Sense<N> + SenseNPLCycles<FUNC, N>,
        FUNC: SenseFunction,
    {
        fn meta(&self) -> CommandTypeMeta {
            CommandTypeMeta::Both
        }

        fn event(
            &self,
            device: &mut D,
            _context: &mut scpi::Context,
            mut args: Arguments,
        ) -> scpi::error::Result<()> {
            let nplc: NumericValue<Ratio> = args.data()?;
            device.nplc(nplc)
        }

        fn query(
            &self,
            device: &mut D,
            _context: &mut scpi::Context,
            mut args: Arguments,
            mut response: ResponseUnit,
        ) -> scpi::error::Result<()> {
            let nplc: Ratio = match args.optional_data::<NumericValue<()>>()? {
                None => device.get_nplc(),
                Some(NumericValue::Maximum) => todo!(),
                Some(NumericValue::Minimum) => todo!(),
                Some(_) => todo!(),
            };

            // Convert aperture time to NPL with instrument line frequency
            response.data(nplc).finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        scpi1999::{sense::*, tests::fixture_scpi_device}, sense::function::SensorFunction,
    };

    struct Test;
    fixture_scpi_device!(Test);

    struct TestSenseFunction;
    impl SenseFunction for TestSenseFunction {
        type Unit = f32;
    }

    impl Sense for Test {
        
        fn function_on(&mut self, _function: SensorFunction) -> Result<(), FunctionError> {
            unimplemented!()
        }

        fn get_function_on(&self) -> Result<SensorFunction, FunctionError> {
            unimplemented!()
        }

        type Function = SensorFunction;
    }

    #[cfg(feature = "unit-time")]
    use super::aperture::{SenseAperture, SensApertureCommand};

    #[cfg(feature = "unit-time")]
    impl SenseAperture<TestSenseFunction> for Test {
        fn aperture(
            &mut self,
            _aperture: crate::scpi1999::NumericValue<uom::si::f32::Time>,
        ) -> crate::error::Result<()> {
            unimplemented!()
        }

        fn get_aperture(&self) -> uom::si::f32::Time {
            unimplemented!()
        }
    }

    #[cfg(feature = "unit-time")]
    #[test]
    fn test_aperture() {
        let _: Node<Test> = Node::Leaf {
            name: b"",
            default: false,
            handler: &SensApertureCommand::new(),
        };
    }

    #[cfg(feature = "unit-ratio")]
    use super::nplc::{SensNPLCyclesCommand, SenseNPLCycles};

    #[cfg(feature = "unit-ratio")]
    impl SenseNPLCycles<TestSenseFunction> for Test {
        fn nplc(
            &mut self,
            _aperture: scpi::scpi1999::NumericValue<uom::si::f32::Ratio>,
        ) -> scpi::error::Result<()> {
            unimplemented!()
        }

        fn get_nplc(&self) -> uom::si::f32::Ratio {
            unimplemented!()
        }
    }

    #[cfg(feature = "unit-ratio")]
    #[test]
    fn test_nplc() {
        let _: Node<Test> = Node::Leaf {
            name: b"",
            default: false,
            handler: &SensNPLCyclesCommand::new(),
        };
    }
}

use core::marker::PhantomData;

use crate::{
    command::CommandTypeMeta,
    error::Result,
    prelude::*,
    scpi1999::{system::LineFrequency, util::Auto},
};

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

pub struct SenseRangeUpperCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SenseRangeUpperCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseRange<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> scpi::command::CommandTypeMeta {
        scpi::command::CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: scpi::parameters::Arguments,
    ) -> scpi::error::Result<()> {
        let upper = args.data::<NumericValue<FUNC::Unit>>()?;
        device.range_upper(upper)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
    ) -> scpi::error::Result<()> {
        let upper = device.get_range_upper();
        response.data(upper).finish()
    }
}

pub struct SenseRangeLowerCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SenseRangeLowerCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseRange<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> scpi::command::CommandTypeMeta {
        scpi::command::CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: scpi::parameters::Arguments,
    ) -> scpi::error::Result<()> {
        let upper = args.data::<NumericValue<FUNC::Unit>>()?;
        device.range_lower(upper)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
    ) -> scpi::error::Result<()> {
        let upper = device.get_range_lower();
        response.data(upper).finish()
    }
}

pub struct SenseRangeAutoCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SenseRangeAutoCommand<FUNC, N>
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
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
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

pub struct SenseResolutionCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SenseResolutionCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseResolution<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> scpi::command::CommandTypeMeta {
        scpi::command::CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: scpi::parameters::Arguments,
    ) -> scpi::error::Result<()> {
        let resolution = args.data::<NumericValue<FUNC::Unit>>()?;
        device.resolution(resolution)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
    ) -> scpi::error::Result<()> {
        let resolution = device.get_resolution();
        response.data(resolution).finish()
    }
}

trait SenseAperture<FUNC, const N: usize = 1>: Sense<N>
where
    FUNC: SenseFunction,
{
    ///
    fn aperture(&mut self, aperture: NumericValue<uom::si::f32::Time>) -> Result<()>;
    fn get_aperture(&self) -> uom::si::f32::Time;
}

pub struct SenseApertureCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SenseApertureCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseAperture<FUNC, N>,
    FUNC: SenseFunction,
{
    fn meta(&self) -> scpi::command::CommandTypeMeta {
        scpi::command::CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: scpi::parameters::Arguments,
    ) -> scpi::error::Result<()> {
        let aperture: NumericValue<uom::si::f32::Time> = args.data()?;
        device.aperture(aperture)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
    ) -> scpi::error::Result<()> {
        let aperture: uom::si::f32::Time = device.get_aperture();
        response.data(aperture).finish()
    }
}

pub struct SenseNPLCyclesCommand<FUNC, const N: usize = 1> {
    phantom: PhantomData<FUNC>,
}

impl<D, FUNC, const N: usize> Command<D> for SenseNPLCyclesCommand<FUNC, N>
where
    D: ScpiDevice + Sense<N> + SenseAperture<FUNC, N> + LineFrequency,
    FUNC: SenseFunction,
{
    fn meta(&self) -> scpi::command::CommandTypeMeta {
        scpi::command::CommandTypeMeta::Both
    }

    fn event(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        mut args: scpi::parameters::Arguments,
    ) -> scpi::error::Result<()> {
        let aperture: NumericValue<uom::si::f32::Time> = args
            .data::<NumericValue<f32>>()?
            .map(|npl| npl / device.get_line_frequency());
        device.aperture(aperture)
    }

    fn query(
        &self,
        device: &mut D,
        _context: &mut scpi::Context,
        _args: scpi::parameters::Arguments,
        mut response: scpi::response::ResponseUnit,
    ) -> scpi::error::Result<()> {
        let aperture: uom::si::f32::Time = device.get_aperture();
        // Convert aperture time to NPL with instrument line frequency
        response
            .data(aperture * device.get_line_frequency())
            .finish()
    }
}

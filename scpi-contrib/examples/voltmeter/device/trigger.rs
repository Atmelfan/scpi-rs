use scpi::{units, error::Result};

use scpi_contrib::scpi1999::{
    trigger::{abort::Abort, initiate::Initiate, Trigger, TriggerSource},
    NumericValue,
};

use super::Voltmeter;

impl Trigger for Voltmeter {
    type Source = TriggerSource;

    fn count(&mut self, count: NumericValue<usize>) -> Result<()> {
        todo!()
    }

    fn get_count(&self) -> usize {
        todo!()
    }

    fn delay(&mut self, delay: NumericValue<units::Time>) -> Result<()> {
        todo!()
    }

    fn get_delay(&self) -> units::Time {
        todo!()
    }

    fn source(&mut self, source: Self::Source) -> Result<()> {
        Ok(())
    }

    fn get_source(&self) -> Self::Source {
        todo!()
    }
}

impl Initiate for Voltmeter {
    fn initiate(&mut self) {
        todo!()
    }
}

impl Abort for Voltmeter {
    fn abort(&mut self) {
        todo!()
    }
}

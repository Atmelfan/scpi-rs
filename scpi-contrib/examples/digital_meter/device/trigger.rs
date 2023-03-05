use scpi::{error::Result, prelude::ErrorCode, units};

use scpi_contrib::{
    scpi1999::{
        trigger::{abort::Abort, initiate::Initiate, Trigger, TriggerSource},
        NumericValue,
    },
    trg::CommonTrg,
    trigger::TriggerState,
    ScpiDevice,
};

use super::Voltmeter;

impl Trigger for Voltmeter {
    type Source = TriggerSource;

    fn count(&mut self, count: NumericValue<usize>) -> Result<()> {
        self.trigger_cnt = count.build().default(1).max(10).min(1).finish()?;
        Ok(())
    }

    fn get_count(&self) -> usize {
        self.trigger_cnt
    }

    fn delay(&mut self, _delay: NumericValue<units::Time>) -> Result<()> {
        todo!()
    }

    fn get_delay(&self) -> units::Time {
        todo!()
    }

    fn source(&mut self, source: Self::Source) -> Result<()> {
        match source {
            TriggerSource::Bus | TriggerSource::Immediate => self.trigger_src = source,
            _ => return Err(ErrorCode::IllegalParameterValue.into()),
        }
        Ok(())
    }

    fn get_source(&self) -> Self::Source {
        self.trigger_src
    }
}

impl Initiate for Voltmeter {
    fn initiate(&mut self) {
        if self.trigger_state != TriggerState::Idle {
            self.push_error(ErrorCode::InitIgnored.into())
        }

        self.trigger_state = TriggerState::WaitingForTrigger;

        if self.trigger_src == TriggerSource::Immediate {
            // Take measurments immediately
            let meas = self.measurement.get_or_insert(Vec::new());
            for _i in 0..self.trigger_cnt {
                meas.push(self.sensor.sense());
            }
            self.trigger_state = TriggerState::Idle
        } else {
            // Invalidate current measurement
            self.measurement = None;
        }
    }
}

impl Abort for Voltmeter {
    fn abort(&mut self) {
        // Reset trigger to idle and invalidate measurement data
        if self.trigger_state != TriggerState::Idle {
            self.trigger_state = TriggerState::Idle;
            self.measurement = None;
        }
    }
}

impl CommonTrg for Voltmeter {
    fn trig_bus(&mut self) -> Result<()> {
        if self.trigger_src == TriggerSource::Bus
            && self.trigger_state == TriggerState::WaitingForTrigger
        {
            // Take measurment immediately
            let meas = self.measurement.get_or_insert(Vec::new());
            meas.push(self.sensor.sense());
            if meas.len() == self.trigger_cnt {
                self.trigger_state = TriggerState::Idle;
            }
        } else {
            // Log error
            self.push_error(ErrorCode::TriggerIgnored.into())
        }
        Ok(())
    }
}

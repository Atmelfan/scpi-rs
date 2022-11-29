//!Contains SCPI modules and mandatory commands
//!
//!

use crate::error::{Error, Result};
use crate::prelude::ErrorCode;
use crate::{ieee488::IEEE488Device, prelude::ErrorQueue, Device};

pub use self::status::{GetEventRegister, Operation, Questionable};

pub mod mandatory;
pub mod status;
pub mod system;

// Measurement instructions
pub mod measurement;
pub mod trigger;

pub trait ScpiDevice:
    Device + ErrorQueue + GetEventRegister<Operation> + GetEventRegister<Questionable>
{
    /// Get device status
    ///
    fn status(&self) -> u8 {
        0x00
    }

    /// See [crate::ieee488::IEEE488Device::sre]
    fn sre(&self) -> u8;

    /// See [crate::ieee488::IEEE488Device::set_sre]
    fn set_sre(&mut self, value: u8);

    /// See [crate::ieee488::IEEE488Device::esr]
    fn esr(&self) -> u8;

    /// See [crate::ieee488::IEEE488Device::set_esr]
    fn set_esr(&mut self, value: u8);

    /// See [crate::ieee488::IEEE488Device::ese]
    fn ese(&self) -> u8;

    /// See [crate::ieee488::IEEE488Device::set_ese]
    fn set_ese(&mut self, value: u8);

    ///
    fn exec_preset(&mut self) -> Result<()> {
        // Clear operation register
        <Self as GetEventRegister<Operation>>::register_mut(self).preset();
        <Self as GetEventRegister<Questionable>>::register_mut(self).preset();
        Ok(())
    }

    /// See [crate::ieee488::IEEE488Device::exec_tst]
    fn exec_tst(&mut self) -> crate::error::Result<()> {
        Ok(())
    }

    /// See [crate::ieee488::IEEE488Device::exec_rst]
    fn exec_rst(&mut self) -> crate::error::Result<()> {
        Ok(())
    }

    /// See [crate::ieee488::IEEE488Device::exec_opc]
    fn exec_opc(&mut self) -> Result<()> {
        let esr = self.esr() | ErrorCode::OperationComplete.esr_mask();
        self.push_back_error(ErrorCode::OperationComplete.into());
        self.set_esr(esr);
        Ok(())
    }

    /// See [crate::ieee488::IEEE488Device::exec_cls]
    fn exec_cls(&mut self) -> crate::error::Result<()> {
        // Clear ESR
        self.set_esr(0);
        // Clear event registers
        <Self as GetEventRegister<Operation>>::register_mut(self).clear_event();
        <Self as GetEventRegister<Questionable>>::register_mut(self).clear_event();
        Ok(())
    }

    fn _handle_error(&mut self, err: Error) {
        // Set ESR mask
        let esr = self.esr() | err.esr_mask();
        self.set_esr(esr);
        // Add error to error/event queue
        self.push_back_error(err);
    }
}

/// SCPI devices are a superset of IEEE488 devices
/// and must implement all mandatory IEEE488 functions.
impl<T> IEEE488Device for T
where
    T: ScpiDevice,
{
    fn status(&self) -> u8 {
        let mut reg = <Self as ScpiDevice>::status(self);
        //Set OPERation status bits

        if <Self as GetEventRegister<Operation>>::register(self).get_summary() {
            reg |= 0x80;
        }
        //Set QUEStionable status bit
        if <Self as GetEventRegister<Questionable>>::register(self).get_summary() {
            reg |= 0x08;
        }
        //Set error queue empty bit
        if !self.is_empty() {
            reg |= 0x04;
        }
        reg
    }

    fn sre(&self) -> u8 {
        <Self as ScpiDevice>::sre(self)
    }

    fn set_sre(&mut self, value: u8) {
        <Self as ScpiDevice>::set_sre(self, value)
    }

    fn esr(&self) -> u8 {
        <Self as ScpiDevice>::esr(self)
    }

    fn set_esr(&mut self, value: u8) {
        <Self as ScpiDevice>::set_esr(self, value)
    }

    fn ese(&self) -> u8 {
        <Self as ScpiDevice>::ese(self)
    }

    fn set_ese(&mut self, value: u8) {
        <Self as ScpiDevice>::set_ese(self, value)
    }

    fn exec_tst(&mut self) -> crate::error::Result<()> {
        <Self as ScpiDevice>::exec_tst(self)
    }

    fn exec_rst(&mut self) -> crate::error::Result<()> {
        <Self as ScpiDevice>::exec_rst(self)
    }

    fn exec_cls(&mut self) -> crate::error::Result<()> {
        <Self as ScpiDevice>::exec_cls(self)
    }

    fn exec_opc(&mut self) -> Result<()> {
        <Self as ScpiDevice>::exec_opc(self)
    }

    fn _handle_error(&mut self, err: Error) {
        <Self as ScpiDevice>::_handle_error(self, err)
    }
}

/// This struct contains a register with event/enable functionality
/// (used in OPERation/QUEStionable registers)
///
///
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct EventRegister {
    pub condition: u16,
    pub event: u16,
    pub enable: u16,
    pub ntr_filter: u16,
    pub ptr_filter: u16,
}

/// Utility trait
pub trait BitFlags<T> {
    /// Return a bitmask with the relevant bits set and others cleared
    ///
    fn get_mask(self) -> T;
    /// Return the position/offset of the relevant bit(s)
    fn get_pos(self) -> T;
}

/// The OPERation status register contains conditions which are part of the instrument’s normal
/// operation.
pub enum OperationBits {
    /// The instrument is currently performing a calibration.
    Calibrating = 0,
    /// The instrument is waiting for signals it controls to stabilize
    /// enough to begin measurements.
    Settling = 1,
    /// The instrument is currently changing its range.
    Ranging = 2,
    /// A sweep is in progress.
    Sweeping = 3,
    /// The instrument is actively measuring.
    Measuring = 4,
    /// The instrument is in a “wait for trigger” state of the
    /// trigger model.
    WaitingForTrig = 5,
    /// The instrument is in a “wait for arm” state of the trigger
    /// model.
    WaitingForArm = 6,
    /// The instrument is currently performing a correction.
    Correcting = 7,
    /// Available to designer.
    Designer1 = 8,
    /// Available to designer.
    Designer2 = 9,
    /// Available to designer.
    Designer3 = 10,
    /// Available to designer.
    Designer4 = 11,
    /// Available to designer.
    Designer5 = 12,
    /// One of n multiple logical instruments is
    /// reporting OPERational status.
    InstrumentSummary = 13,
    /// A user-defined programming is currently in the run
    /// state.
    ProgramRunning = 14,
}

impl BitFlags<u16> for OperationBits {
    fn get_mask(self) -> u16 {
        1 << (self as u16)
    }

    fn get_pos(self) -> u16 {
        self as u16
    }
}

/// The QUEStionable status register set contains bits which give an indication of the quality of
/// various aspects of the signal.
pub enum QuestionableBits {
    /// Indicates that the data is currently being acquired or generated
    SummaryVoltage = 0,
    SummaryCurrent = 1,
    SummaryTime = 2,
    SummaryPower = 3,
    SummaryTemperature = 4,
    SummaryFrequency = 5,
    SummaryPhase = 6,
    SummaryModulation = 7,
    SummaryCalibration = 8,
    Designer1 = 9,
    Designer2 = 10,
    Designer3 = 11,
    Designer4 = 12,
    InstrumentSummary = 13,
    /// Bit 14 is defined as the Command Warning bit. This bit indicates a non-fatal warning that
    /// relates to the instrument’s interpretation of a command, query, or one or more parameters of
    /// a specific command or query. Setting this bit is a warning to the application that the resultant
    /// instrument state or action is probably what was expected but may deviate in some manner.
    ///
    /// For example, the Command Warning bit is set whenever a parameter in one of the
    /// Measurement Instruction commands or queries is ignored during execution. Such a
    /// parameter may be ignored because it cannot be specified by a particular instrument.
    CommandWarning = 14,
}

impl BitFlags<u16> for QuestionableBits {
    fn get_mask(self) -> u16 {
        1 << (self as u16)
    }

    fn get_pos(self) -> u16 {
        self as u16
    }
}

impl Default for EventRegister {
    fn default() -> Self {
        EventRegister {
            condition: 0,
            event: 0,
            enable: 0,
            ntr_filter: 0,
            ptr_filter: 0xffff,
        }
    }
}

impl EventRegister {
    /// Create a new event register
    pub fn new() -> Self {
        EventRegister::default()
    }

    /// Preset the registers to default values
    pub fn preset(&mut self) {
        self.enable = 0u16;
        self.condition = 0u16;
        self.ptr_filter = 0xffffu16;
        self.ntr_filter = 0u16;
    }

    pub fn clear_event(&mut self) {
        self.event = 0;
    }

    /// Return the enabled operation bits summary.
    /// Returns true if any enabled condition bit is set, false otherwise.
    ///
    pub fn get_summary(&self) -> bool {
        (self.condition & self.enable) & 0x7fffu16 != 0u16
    }

    /// Get the state of relevant bit in status register. Returns true if bit is set, false otherwise.
    pub fn get_condition_bit(&self, bit: OperationBits) -> bool {
        self.condition & (bit as u16) != 0
    }

    /// Update condition register and event register based on pos-/neg-transition filters
    pub fn set_condition(&mut self, condition: u16) {
        let transitions = self.condition ^ condition;
        // Record pos-/negative-transitions to event register
        self.event |=
            transitions & ((condition & self.ptr_filter) | (!condition & self.ntr_filter));
        //Save new condition
        self.condition = condition;
    }

    /// Set relevant bit in condition register
    pub fn set_condition_bits(&mut self, bitmask: u16) {
        self.set_condition(self.condition | bitmask)
    }

    /// Clear relevant bit in condition register
    pub fn clear_condition_bits(&mut self, bitmask: u16) {
        self.set_condition(self.condition & !bitmask)
    }
}

#[cfg(test)]
mod tests {}

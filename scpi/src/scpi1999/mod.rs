//!Contains SCPI modules and mandatory commands
//!
//!

use crate::error::{Error, ErrorCode, ErrorQueue, Result};
use crate::{ieee488::IEEE488Device, Device};

pub use self::status::{Operation, Questionable};

pub mod numeric;
#[doc(inline)]
pub use numeric::{NumericBuilder, NumericValue};

// Subsystems
pub mod input;
pub mod measurement;
pub mod sense;
pub mod status;
pub mod system;
pub mod trigger;
pub mod unit;

pub mod prelude {
    pub use super::{
        status::{Operation, Questionable},
        EventRegister, GetEventRegister, ScpiDevice,
    };
    pub use crate::error::{Error, ErrorQueue};
}

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
        self.get_register_mut::<Operation>().clear_event();
        self.get_register_mut::<Questionable>().clear_event();
        Ok(())
    }

    fn _handle_error(&mut self, err: Error) {
        // Set ESR mask
        let esr = self.esr() | err.esr_mask();
        self.set_esr(esr);
        // Add error to error/event queue
        self.push_back_error(err);
    }

    fn get_register<REG>(&self) -> &EventRegister
    where
        Self: GetEventRegister<REG>,
        REG: EventRegisterName,
    {
        <Self as GetEventRegister<REG>>::register(self)
    }

    fn get_register_mut<REG>(&mut self) -> &mut EventRegister
    where
        Self: GetEventRegister<REG>,
        REG: EventRegisterName,
    {
        <Self as GetEventRegister<REG>>::register_mut(self)
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

        if self.get_register::<Operation>().get_summary() {
            reg |= 0x80;
        }
        //Set QUEStionable status bit
        if self.get_register::<Questionable>().get_summary() {
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

pub trait EventRegisterName {
    type BitFlags;
}

pub trait GetEventRegister<X>
where
    X: EventRegisterName,
{
    fn register(&self) -> &EventRegister;
    fn register_mut(&mut self) -> &mut EventRegister;
}

/// Utility trait
pub trait BitFlags<T> {
    /// Return a bitmask with the relevant bits set and others cleared
    ///
    fn get_mask(self) -> T;
    /// Return the position/offset of the relevant bit(s)
    fn get_pos(self) -> T;
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
    pub fn get_condition_bit(&self, bitmask: u16) -> bool {
        self.condition & bitmask != 0
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

pub mod util {
    use crate::{
        error::{Error, Result},
        parser::{
            mnemonic_compare,
            response::{Formatter, ResponseData},
            tokenizer::Token,
        },
    };

    /// `AUTO <Boolean>|ONCE`
    #[derive(Debug, Clone, Copy)]
    pub enum Auto {
        Once,
        Bool(bool),
    }

    impl<'a> TryFrom<Token<'a>> for Auto {
        type Error = Error;

        fn try_from(value: Token<'a>) -> Result<Self> {
            match value {
                Token::CharacterProgramData(s) => match s {
                    //Check for special float values
                    x if mnemonic_compare(b"ONCE", x) => Ok(Self::Once),
                    _ => Ok(Self::Bool(bool::try_from(value)?)),
                },
                t => Ok(Self::Bool(bool::try_from(t)?)),
            }
        }
    }

    impl ResponseData for Auto {
        fn format_response_data(&self, formatter: &mut dyn Formatter) -> Result<()> {
            match self {
                Auto::Once => formatter.push_ascii(b"ONCE"),
                Auto::Bool(value) => value.format_response_data(formatter),
            }
        }
    }

    impl From<bool> for Auto {
        fn from(value: bool) -> Self {
            Self::Bool(value)
        }
    }

    impl Auto {
        /// Autorange has been run. If self is [Auto::Once] it will be set to `false`.
        pub fn auto_once(&mut self) {
            *self = match &self {
                Auto::Once => Self::Bool(false),
                Auto::Bool(value) => Self::Bool(*value),
            };
        }

        /// Returns true if autorange should be used
        ///
        /// **Note:** Call `auto_once()` when autorange has been executed to handle [Auto::Once] logic
        pub fn auto_enabled(&self) -> bool {
            matches!(self, Self::Once | Self::Bool(true))
        }
    }
}

#[cfg(test)]
mod tests {
    // Test fixture
    macro_rules! fixture_scpi_device {
        ($dev:ident) => {
            impl $crate::scpi1999::ScpiDevice for $dev {
                fn sre(&self) -> u8 {
                    unimplemented!()
                }

                fn set_sre(&mut self, _value: u8) {
                    unimplemented!()
                }

                fn esr(&self) -> u8 {
                    unimplemented!()
                }

                fn set_esr(&mut self, _value: u8) {
                    unimplemented!()
                }

                fn ese(&self) -> u8 {
                    unimplemented!()
                }

                fn set_ese(&mut self, _value: u8) {
                    unimplemented!()
                }
            }

            impl $crate::scpi1999::GetEventRegister<$crate::scpi1999::Questionable> for $dev {
                fn register(&self) -> &$crate::scpi1999::EventRegister {
                    unimplemented!()
                }

                fn register_mut(&mut self) -> &mut $crate::scpi1999::EventRegister {
                    unimplemented!()
                }
            }

            impl $crate::scpi1999::GetEventRegister<$crate::scpi1999::Operation> for $dev {
                fn register(&self) -> &$crate::scpi1999::EventRegister {
                    unimplemented!()
                }

                fn register_mut(&mut self) -> &mut $crate::scpi1999::EventRegister {
                    unimplemented!()
                }
            }

            impl $crate::error::ErrorQueue for $dev {
                fn push_back_error(&mut self, _err: $crate::error::Error) {
                    unimplemented!()
                }

                fn pop_front_error(&mut self) -> Option<$crate::error::Error> {
                    unimplemented!()
                }

                fn num_errors(&self) -> usize {
                    unimplemented!()
                }

                fn clear_errors(&mut self) {
                    unimplemented!()
                }
            }
        };
    }
    pub(crate) use fixture_scpi_device;
}

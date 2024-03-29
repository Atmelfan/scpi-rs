//!Contains SCPI modules and mandatory commands
//!
//!

use core::fmt::Display;

use scpi::error::{Error, ErrorCode, ErrorQueue, Result};
use scpi::Device;

use crate::ieee488::{StatusBit, IEEE4882};

use self::status::{operation::Operation, questionable::Questionable};

#[doc(hidden)]
mod numeric;
#[doc(inline)]
pub use numeric::{NumericBuilder, NumericValue, NumericValueDefaults, NumericValueQuery};

// Subsystems
//pub mod input;
//pub mod measurement;
//pub mod output;
//pub mod sense;
pub mod status;
pub mod system;
//pub mod trigger;
//pub mod unit;

pub mod prelude {
    pub use super::{
        status::{operation::Operation, questionable::Questionable},
        EventRegister, GetEventRegister, ScpiDevice,
    };
    pub use scpi::error::{Error, ErrorQueue};
}

pub trait ScpiDevice:
    Device + ErrorQueue + GetEventRegister<Operation> + GetEventRegister<Questionable> + IEEE4882
{
    fn request_service(&mut self, _status: u8) {
        // Do nothing
    }

    /// Preset registers
    fn preset(&mut self) -> Result<()> {
        // Preset operation register
        self.preset_register::<Operation>();
        // Preset questionable
        self.preset_register::<Questionable>();
        Ok(())
    }

    /// Calculate STB byte according to SCPI standard
    ///
    /// Should be called from [crate::ieee488::IEEE4882::stb]
    fn scpi_stb(&self) -> u8 {
        let mut stb = 0x00;
        if !self.is_empty() {
            stb |= StatusBit::ErrorEventQueue.mask();
        }
        if self.get_register_summary::<Questionable>() {
            stb |= StatusBit::Questionable.mask();
        }
        if self.get_register_summary::<Operation>() {
            stb |= StatusBit::Operation.mask();
        }
        // ESB
        if self.esr() & self.ese() != 0 {
            stb |= StatusBit::Esb.mask();
        }
        // MSS
        if stb & self.sre() != 0 {
            stb |= StatusBit::RqsMss.mask();
        }
        stb
    }

    /// Handle OPC command
    ///
    /// Should be called from [crate::ieee488::IEEE4882::opc]
    fn scpi_opc(&mut self) -> Result<()> {
        let esr = self.esr() | ErrorCode::OperationComplete.esr_mask();
        self.push_back_error(ErrorCode::OperationComplete.into());
        self.set_esr(esr);
        Ok(())
    }

    /// Handle CLS command
    ///
    /// Should be called from [crate::ieee488::IEEE4882::cls]
    fn scpi_cls(&mut self) -> Result<()> {
        // Clear ESR
        self.set_esr(0);
        // Clear event registers
        self.get_register_mut::<Operation>().clear_event();
        self.get_register_mut::<Questionable>().clear_event();
        Ok(())
    }

    fn push_error(&mut self, err: Error) {
        // Set ESR mask
        let esr = self.esr() | err.esr_mask();
        self.set_esr(esr);
        // Add error to error/event queue
        self.push_back_error(err);
    }

    /// Get event register
    fn get_register<REG>(&self) -> &EventRegister
    where
        Self: GetEventRegister<REG>,
        REG: EventRegisterName,
    {
        <Self as GetEventRegister<REG>>::register(self)
    }

    /// Get event register as mutable
    fn get_register_mut<REG>(&mut self) -> &mut EventRegister
    where
        Self: GetEventRegister<REG>,
        REG: EventRegisterName,
    {
        <Self as GetEventRegister<REG>>::register_mut(self)
    }

    /// Preset event register
    fn preset_register<REG>(&mut self)
    where
        Self: GetEventRegister<REG>,
        REG: EventRegisterName,
    {
        <Self as GetEventRegister<REG>>::register_mut(self).preset()
    }

    /// Get event register summary
    fn get_register_summary<REG>(&self) -> bool
    where
        Self: GetEventRegister<REG>,
        REG: EventRegisterName,
    {
        <Self as GetEventRegister<REG>>::register(self).get_summary()
    }
}

/// This struct contains a register with event/enable functionality
/// (used in OPERation/QUEStionable registers)
///
///
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct EventRegister {
    pub condition: u16,
    pub event: u16,
    pub enable: u16,
    pub ntr_filter: u16,
    pub ptr_filter: u16,
}

impl Display for EventRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}, evt={}, en={}",
            self.condition, self.event, self.enable
        )
    }
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
    use scpi::{
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

    impl Default for Auto {
        fn default() -> Self {
            Self::Bool(false)
        }
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

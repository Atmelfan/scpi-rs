use super::*;

pub struct Operation;
impl EventRegisterName for Operation {
    type BitFlags = OperationBits;
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

///## 20.1.4 \[:EVENt\]?
///> `STATus:OPERation:EVENt?`
///> This query returns the contents of the event register associated with the status structure
///> defined in the command.
///> The response is (NR1 NUMERIC RESPONSE DATA) (range: 0 through 32767) unless
///> changed by the :FORMat:SREGister command.
///>
///> Note that reading the event register clears it.
pub type StatOperEventCommand = EventCommand<Operation>;

///## 20.1.2 :CONDition?
///> `STATus:OPERation:CONDition?`
///> Returns the contents of the condition register associated with the status structure defined in
///> the command. Reading the condition register is nondestructive.
pub type StatOperConditionCommand = ConditionCommand<Operation>;

///## 20.1.3 :ENABle \<NRf\> | \<non-decimal numeric\>
///> `STATus:OPERation:ENABle`
///> Sets the enable mask which allows true conditions in the event register to be reported in the
///> summary bit. If a bit is 1 in the enable register and its associated event bit transitions to true,
///> a positive transition will occur in the associated summary bit.
///> The command accepts parameter values of either format in the range 0 through 65535
///> (decimal) without error.
///>
///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
///> Note that 32767 is the maximum value returned as the most-significant bit of the register
///> cannot be set true.
pub type StatOperEnableCommand = EnableCommand<Operation>;

///# 20.1.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:OPERation:NTRansition`
///> Sets the negative transition filter. Setting a bit in the negative transition filter shall cause a 1
///> to 0 transition in the corresponding bit of the associated condition register to cause a 1 to be
///> written in the associated bit of the corresponding event register.
///> The command accepts parameter values of either format in the range 0 through 65535
///> (decimal) without error.
///>
///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
///> Note that 32767 is the maximum value returned as the most-significant bit of the register
///> cannot be set true.
pub type StatOperNTransitionCommand = NTransitionCommand<Operation>;

///# 20.1.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
///> STATus:OPERation:PTRansition
///> Sets the positive transition filter. Setting a bit in the positive transition filter shall cause a 0 to
///> transition in the corresponding bit of the associated condition register to cause a 1 to be
///> written in the associated bit of the corresponding event register.
///> The command accepts parameter values of either format in the range 0 through 65535
///> (decimal) without error.
///>
///> The query response format is <NR1> unless changed by the :FORMat:SREGister command.
///> Note that 32767 is the maximum value returned as the most-significant bit of the register
///> cannot be set true.
pub type StatOperPTransitionCommand = PTransitionCommand<Operation>;

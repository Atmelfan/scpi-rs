use super::*;

pub struct Questionable;
impl EventRegisterName for Questionable {
    type BitFlags = QuestionableBits;
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

///# 20.3.4 \[:EVENt\]?
///> `STATus:QUEStionable:EVENt?`
///> Defined the same as STATus:OPERation:EVENt. See Section 20.1.4 for details.
pub type StatQuesEventCommand = EventCommand<Questionable>;

///# 20.3.2 :CONDition?
///> `STATus:QUEStionable:CONDition?`
///> Defined the same as STATus:OPERation:CONDition. See Section 20.1.2 for details.
pub type StatQuesConditionCommand = ConditionCommand<Questionable>;

///# 20.3.3 :ENABle \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:ENABle`
///Defined the same as STATus:OPERation:ENABle. See Section 20.1.3 for details.
pub type StatQuesEnableCommand = EnableCommand<Questionable>;

///# 20.3.6 :NTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:NTRansition`
///> Defined the same as STATus:OPERation:NTRansition. See Section 20.1.6 for details.
pub type StatQuesNTransitionCommand = NTransitionCommand<Questionable>;

///# 20.3.7 :PTRansition \<NRf\> | \<non-decimal numeric\>
///> `STATus:QUEStionable:PTRansition`
///> Defined the same as STATus:OPERation:PTRansition. See Section 20.1.7 for details.
pub type StatQuesPTransitionCommand = PTransitionCommand<Questionable>;

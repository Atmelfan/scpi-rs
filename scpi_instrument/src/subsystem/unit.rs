//! # 25 Default units are defined, where applicable, for each SCPI command. The UNIT subsystem
//! provides a mechanism to change the default values. The units selected apply to the
//! designated command parameters for both command and response.
//!
//! The UNIT command at the root level has a global effect on the selected units. The UNIT
//! command may also be applied to lower levels in the SCPI command hierarchy to have a
//! localized effect. When the UNIT command is applied to a node, then all the nodes below the
//! node to which the unit command was applied shall be affected by the localized UNIT
//! command. There is no restriction on the number of levels to which UNIT may be applied. In
//! this way the more global units are overridden by the more local units. Units may also be
//! overridden temporarily by attaching the desired unit as a suffix to the appropriate parameter
//! in the command, if the instrument supports that unit.
//!
//! For example, to program a source with units of Volts, the command UNIT:VOLTage VOLT
//! would be used. To program only the modulator of that same source in dBuVs, the additional
//! command \[SOURce:]MODulation:UNIT DBUV would be required.
//!
//! The UNIT command cannot be applied to either SENSe, SOURce or ROUTe nodes, since
//! the UNIT command at these nodes and at the root cannot be unambiguously recognized.
//! This condition arises from the ability to have one of SENSe, SOURce or ROUTe nodes
//! optional.
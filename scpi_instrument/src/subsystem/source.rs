//! # 19 SOURce Subsystem
//! The SOURce setup commands are divided into several sections. Each section or subsystem
//! deals with controls that directly affect device-specific settings of the device, and not those
//! related to the signal-oriented characteristics. These commands are referenced through the
//! SOURCe node.
//!
//! The SOURce node may be optional in a particular device. The reason that the SOURce is
//! optional is to allow devices which are primarily sources to accept shorter commands. The
//! SOURce node cannot be optional if either SENSe or ROUTe is optional, since only one node
//! at a given level may be optional. For example, a typical power supply may elect to make the
//! SOURce node optional, since the most frequently used commands would be under the
//! SOURce subsystem. If the power supply also contained either source or routing
//! functionality, these would be controlled through their respective nodes; however, the SENSe
//! or ROUTe nodes would be required. An optional node, such as SOURce, implies that the
//! device shall accept and process commands with or without the optional node and have the
//! same result. That is, the device is required to accept the optional node, if sent, without error.
//!
//! In some instances, a source may contain subservient source or sensor functions. In such
//! cases, the additional subservient functionality shall be placed as SENSe or SOURce
//! subnodes under the SOURce subsystem. Further, no optional nodes (SENSe, SOURce or
//! ROUTe) are permitted if such subservient functionality exists in a device. Otherwise
//! conflicts in interpreting commands would occur, for example, between SOURce and
//! \[SENSe:]SOURce if the SENSe were allowed to be optional.
//!
//! CURRent, POWer, VOLTage, FREQuency, and RESistance contain several sets of
//! commands which have complex couplings. This includes the sweep commands STARt,
//! STOP, CENTer, and SPAN as one set, and AMPLitude, OFFSet, HIGH, and LOW as
//! another. Sending any one of a set singly will affect two others of the set. However, if two are
//! sent in a single message unit, then these two should be set to the values specified and the
//! other two changed. This is in accordance with the style guidelines and IEEE 488.2. STARt,
//! STOP, CENTer, and SPAN must be implemented as a set. That is, if any one is
//! implemented, then all must be implemented.
//!
//! If the requested setting is unachievable given the other instrument settings, then an error
//! must be generated (-211 “settings conflict”). The device may resolve the conflict in a
//! device-dependent manner (for example, change STARt, STOP, CENTer, and/or SPAN) to
//! resolve the error. Note that when more than one of the four sweep settings are received in the
//! same program message unit, the sweep will be determined by the last two received.
//!
//! The coupled commands may define commands as subnodes which alter these couplings. The
//! command description will define how the couplings are altered. However, any command
//! which alters couplings must default to the couplings described in this section as the *RST
//! couplings.
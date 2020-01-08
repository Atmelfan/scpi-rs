//! # 18 SENSe Subsystem
//! The SENSe setup commands are divided into several sections. Each section or subsystem
//! deals with controls that directly affect device-specific settings of the device and not those
//! related to the signal-oriented characteristics. These commands are referenced through the
//! SENSe node.
//!
//! The SENSe node may be optional in a particular device. The reason that the SENSe is
//! optional is to allow devices which are primarily sensors to accept shorter commands. The
//! SENSe node cannot be optional if either SOURce or ROUTe is optional, since only one
//! node at a given level may be optional. For example, a typical counter may elect to make the
//! SENSe node optional, since the most frequently used commands would be under the SENSe
//! subsystem. If the counter also contained either source or routing functionality, these would
//! be controlled through their respective nodes; however, the SOURce or ROUTe nodes would
//! be required. An optional node, such as SENSe, implies that the device shall accept and
//! process commands with or without the optional node and have the same result. That is, the
//! device is required to accept the optional node, if sent, without error.
//!
//! In some instances, a sensor may contain subservient source or sensor functions. In such
//! cases, the additional subservient functionality shall be placed as SENSe or SOURce
//! subnodes under the SENSe subsystem. Further, no optional nodes (SENSe, SOURce or
//! ROUTe) are permitted if such subservient functionality exists in a device. Otherwise
//! conflicts in interpreting commands would occur, for example, between SENSe and
//! \[SOURce:]SENSe if the SOURce were allowed to be optional.
//!
//! The FREQuency subsystem contains several sets of commands which have complex
//! couplings. This includes the swept commands STARt, STOP, CENTer, and SPAN as one
//! set. Sending any one of a set singly will affect two others of the set. However, if two are sent
//! in a single message unit, then these two should be set to the values specified and the other
//! two changed. This is in accordance with the style guidelines and IEEE 488.2. If any
//! command in the set is implemented then all the commands in the set shall be implemented.
//!
//! If the requested setting is unachievable given the other instrument settings, then an error
//! must be generated (-221 “Settings conflict”). The device may resolve the conflict in a
//! device-dependent manner (for example, change STARt, STOP, CENTer, and/or SPAN) to
//! resolve the error. Note that when more than one of the four sweep settings are received in the
//! same program message unit, the sweep will be determined by the last two received.
//!
//! The coupled commands may define commands as subnodes which alter these couplings. The
//! command description will define how the couplings are altered. However, any command
//! which alters couplings must default to the couplings described in this section as the *RST
//! couplings.
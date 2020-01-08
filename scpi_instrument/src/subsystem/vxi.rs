//! # 26 VXI Subsystem
//! The VXI subsystem contains commands that control the administration functions associated
//! with operating a VXI-based system. This section will track the work of the VXI consortium.
//!
//! This section describes ASCII commands which are issued to the system from an external
//! host. The exact internal destination device and routing methods to that device are system
//! specific. These commands are used for access to system configuration information, and for
//! common capabilities. The terminology used in this section is patterned after IEEE 488.2.
//!
//! The following commands are the standard set of VXIbus system commands which are
//! ASCII encoded. If a device implements one or more of these commands, the syntax of this
//! section shall be used. The response syntax shall also follow this section.
//!
//! The COMMON ASCII SYSTEM COMMANDS are organized into subsystems. If a device
//! implements a required command in a subsystem then it shall implement all the required
//! commands in the subsystem. If a device implements an optional command in a subsystem
//! then it shall implement all the required commands in the subsystem. If a device implements a
//! command from any subsystem then it shall also implement the VXI:SELect command.
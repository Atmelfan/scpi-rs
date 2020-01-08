//! # 20 STATus Subsystem
//! This subsystem controls the SCPI-defined status-reporting structures. SCPI defines, in
//! addition to those in IEEE 488.2, QUEStionable, OPERation, Instrument SUMmary and
//! INSTrument registers. These registers conform to the IEEE 488.2 specification and each
//! may be comprised of a condition register, an event register, an enable register, and negative
//! and positive transition filters. The purpose and definition of the SCPI-defined registers is
//! described in “Volume 1: Syntax and Style”.
//!
//! SCPI also defines an IEEE 488.2 queue for status. The queue provides a human readable
//! record of instrument events. The application programmer may individually enable events
//! into the queue. STATus:PRESet enables errors and disables all other events. If the summary
//! of the queue is reported, it shall be reported in bit 2 of the status byte register. A subset of
//! error/event numbers is defined by SCPI. Additional error/event numbers will be defined at a
//! later date.
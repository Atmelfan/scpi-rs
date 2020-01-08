//! # 13 MEMory Subsystem
//! The purpose of the memory subsystem is to manage instrument memory. This specifically
//! excludes memory used for mass storage, which is defined in the MMEMory Subsystem.
//! Instrument memory may be used to store several data types (e.g. ASCii, BINary, instrument
//! STATe, TRACe or MACRo.) The data types available for a given instrument are
//! device-dependent.
//!
//! An instrument may support either fixed or dynamic allocation of instrument memory,
//! between the different data types. With fixed allocation a certain amount of memory is
//! dedicated to each of the supported data types. The dedicated memory is unavailable for use
//! by any other data type. Alternatively, with dynamic allocation, instrument memory is shared
//! between the various data types supported on a demand basis.
//!
//! In the MEMory:TABLe subsystems, many fundamental measurement are defined as
//! subnodes below :TABLe, such as MEMory:TABLe:FREQuency, :VOLTage, :POWer, etc.
//! These nodes define standardized entries in the table (note that these entries can be
//! conceptualized as either a column or a row). While it makes sense to define widely used
//! entries in the SCPI Volume 2 standard, there is sometimes a desire to standardize very
//! instrument class specific names. Where instrument class specific names require
//! standardization, and little or no overlap is expected with other instrument classes, these
//! instrument class specific MEMory:TABLe nodes are defined in the Volume 4, Instrument
//! classes chapter to which they apply.
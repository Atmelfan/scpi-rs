//! # 14 MMEMory Subsystem
//! The Mass MEMory subsystem provides mass storage capabilities for instruments. All mass
//! memory device commands are contained in the MMEMory subsystem. The mass storage
//! may be either internal or external to the instrument. If the mass memory device is external,
//! then the instrument must have controller capability on the mass memory device bus.
//!
//! The CLOSe, FEED, NAME, and OPEN commands permit the streaming of data from
//! anywhere in the data flow into a file; this is particularly useful for saving HCOPy output.
//!
//! Mass storage media may be formatted in any one of a number of standard formats.
//!
//! ## Selecting Mass Memory Devices
//! When an instrument has multiple attached mass storage devices, the desired mass storage
//! unit is selected with the mass storage unit specifier \<msus>. The syntax of the \<msus> string
//! is instrument specific. Each instrument’s documentation shall describe the syntax applicable
//! to that instrument. For example:
//! ```text
//! “C:”
//! “:700,2”
//! “fileserv”
//! ```
//! Note that some file systems do not make use of the <msus> concept.
//!
//! ## File Names
//! The \<file_name> parameter described in the commands in this subsystem is a string. The
//! contents of the string are dependent on the needs of the format of the mass storage media. In
//! particular, the file name may contain characters for specifying subdirectories (that is, \\ for
//! DOS, / for HFS) and the separator for extensions in DOS (that is, period).
//! The following commands are provided for mass memory device operations.
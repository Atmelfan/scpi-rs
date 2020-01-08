//! # 4 CALCulate Subsystem
//! The CALCulate subsystem exists to perform post-acquisition data processing. Functions in
//! the SENSe subsystem are related to data acquisition, while the CALCulate subsystem
//! operates on the data acquired by a SENSe function.
//!
//! The CALCulate subsystem is logically between the SENSe subsystem and data output to
//! either the bus or the display. When a measurement is triggered by a MEASure command, an
//! INITiate command, or meeting the prevailing TRIGger conditions, the SENSe subsystem
//! collects data. This data is transformed by CALCulate, as specified, and then passed on to the
//! selected output. In effect, the collection of new data triggers the CALCulate subsystem. The
//! CALCulate subsystem may also be directed by command to perform a transform, making it
//! possible to change the configuration of CALCulate and consequently derive a different set of
//! results from the same SENSed data set without reacquiring sense data.
//!
//! The CALCulate subsystem consists of a number of independent subsystems. Each of the
//! subsystems is a sub-block of the CALCulate block. The data flows through the sub-blocks in
//! a serial fashion, through the first sub-block, then onto the second sub-block and so on. The
//! manner in which these sub-blocks are arranged is specified in the PATH command. It is
//! permissible for a CALCulate block to have more than one instance of any of the sub-blocks.
//! Instances of the same sub-block are differentiated by a numeric suffix. For example, two
//! independent filters would exist as the :CALCulate:FILTer1 and the :CALCulate:FILTer2
//! subsystem sub-blocks.

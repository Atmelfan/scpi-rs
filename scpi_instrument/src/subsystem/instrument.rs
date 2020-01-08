//! # 12 INSTrument Subsystem
//! An instrument may support multiple logical instruments. A dual channel power supply for
//! example is considered as two logical instruments. The logical instruments are not required to
//! have identical functionality, nor does the functionality have to be available simultaneously.
//!
//! The INSTrument subsystem provides a mechanism to identify and select logical instruments
//! by either name or number. In this way a particular logical instrument could be selected and it
//! would respond to commands, such as MEASure, in the same manner as a dedicated
//! instrument having the same functionality as the logical instrument. The INSTrument
//! identifiers have no fixed correspondence to the numeric suffixes allowed with command
//! headers.
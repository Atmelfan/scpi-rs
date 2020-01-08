//! # 9 FORMat Subsystem
//! The FORMat subsystem sets a data format for transferring numeric and array information.
//!
//! This data format is used for both command and response data by those commands that are
//! specifically designated to be affected by the FORMat subsystem. The designation is either
//! given as part of a command description, or in the definition of block or array data used by a
//! command. The data format for command data may override the definition of FORMat if the
//! data received is self typing (indicates its type), for the duration of that data transfer.
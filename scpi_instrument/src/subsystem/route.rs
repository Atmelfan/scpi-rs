//! # 17 ROUTe Subsystem
//! In the SCPI instrument model, the block where user accessibility to the actual signals occurs
//! is called “signal routing.” In some instruments this function may be trivial and thus
//! commands for this block would not be required. In such cases the ports associated with the
//! INPut or OUTput blocks are available directly to the user, typically on the instrument’s front
//! panel. The ROUTe commands apply equally to instruments whose primary purpose is to
//! provide signal routing and to instruments that provide some routing capability “in front of”
//! the INPut and/or OUTput blocks.
//!
//! The ROUTe node may be optional in a particular instrument. This capability is intended for
//! instruments whose primary function is signal routing. The ROUTe node cannot be optional if
//! either SENSe or SOURce is optional, since only one node at a given level may be optional.
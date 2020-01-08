//! # 8 DISPlay Subsystem
//! The DISPlay subsystem controls the selection and presentation of textual, graphical, and
//! TRACe information. This information includes measurement data, user-interaction displays,
//! and data presented to the instrument by the controller. DISPlay is independent of, and does
//! not modify, how data is returned to the controller.
//!
//! Multiple DISPlay subsystems are used to represent independent display medium. A front
//! panel mounted display and an attached terminal used only for information display are
//! examples of independent displays. Conversely, an instrument with many dedicated
//! indicators is considered to have one general display because of the dependencies in operation
//! that exist between the indicators.
//!
//! Within a DISPlay, information may be separated into individual WINDows (this is always
//! the case for instruments with dedicated indicators). Each window is considered to consist of
//! three overlapped planes, one each for text, graphics, and trace data. Thus, text, graphics, and
//! trace information may be presented at the same time in a given window.
//!
//! Most of the *RST conditions in the DISPlay subsystem are device-dependent, to
//! accommodate for typical display functions that exist for each of the various categories of
//! instrumentation.
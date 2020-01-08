//! # 23 TRACe | DATA
//! A TRACe or a DATA area is a named entity stored in instrument memory. TRACe | DATA
//! areas may also be used to store other types of data, such as constant arrays for use in trace
//! arithmetic or corrections, or displayed waveforms. Alternatively, TRACe | DATA areas may
//! be used for equivalent scalar (single point) purposes.
//!
//! Certain TRACe format and capabilities are the focus of other emerging standards. To
//! prevent diluting the effect of SCPI or any other standards related to the manipulation of
//! TRACes, SCPI has left such capability undefined at this time with the view to adopting an
//! industry standard when one becomes available. The undefined capabilities include the
//! following:
//! * Trace math operations for specifying arithmetic relationships between traces, including the use of user-defined constants and functions.
//! * Trace activation and routing for specifying the destination for measurement results.
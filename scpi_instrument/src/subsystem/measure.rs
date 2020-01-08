//! # 3 Measurement Instructions
//! The purpose of the MEASure group of instructions is to acquire data using a set of high-level
//! instructions. The MEASure group of instructions have a duality, in that they exhibit
//! command and query characteristics. The exception to this is CONFigure, which has distinct
//! query and command forms. These instructions are independent of the block diagram and
//! refer to the characteristics of the signal being measured. These instructions are intended to be
//! used with the measurement <functions> referenced later in this chapter.
//!
//! The MEASure group of commands are structured to allow the user to trade off
//! interchangeability with fine control of the measurement process. MEASure? provides a
//! complete capability where the instrument is configured, a measurement taken, and results
//! returned in one operation. Often, more precise control of the measurement is required.
//! Therefore, MEASure? is complemented by providing two commands, CONFigure and
//! READ?. CONFigure performs the configuration portion of the measurement and READ?
//! performs the data acquisition, postprocessing, and data output portions of the measurement.
//! This allows the user to perform a generic configuration of the measurement through
//! CONFigure and then customize the measurement by changing particular
//! instrument-dependent functions. The READ? then completes the measurement process.
//!
//! READ?, in turn, is broken down into two additional commands INITiate\[:IMMediate] and
//! FETCh?. INITiate\[:IMMediate] performs the data acquisition. This command is described in
//! chapter 22. FETCh? performs the postprocessing function and returns the data. This allows
//! the user to perform several different FETCh? functions on a single set of acquired data. For
//! example, an oscilloscope can acquire measurement data that can yield many different signal
//! characteristics such as frequency or AC and DC voltages. Thus, a transient signal may be
//! captured once using a MEASure?, READ? or INITiate. A FETCh? may then be used to
//! obtain each of the different signal characteristics without reacquiring a new measurement.
//!
//! MEASure? provides the best compatibility between instruments because no knowledge of
//! the instrument is required to perform the operation. CONFigure/READ? is less compatible if
//! instrument reconfiguration is performed between the CONFigure and READ? operations.
//! This is because the reconfiguration is, by necessity, instrument-specific. FETCh? is also less
//! compatible because knowledge of the instrument is necessary to determine whether the
//! necessary information has been captured by the instrument. For example, an oscilloscope
//! can capture both rise time and pulse width in a single acquisition. Therefore, if pulse width
//! was acquired through a MEASure? command, it would be possible to FETCh? rise time.
//! However, if pulse width were measured with a counter, the rise time information might not
//! be available without performing another data acquisition. Therefore FETCh? could not be used.
//!
//! Changing certain parts of an instrument’s configuration shall cause existing measurement
//! data to be invalidated. Specifically, in figure 2-1, “Generalized Model of a Programmable
//! Instrument” in chapter 2, “Instrument Model,” any reconfiguration of signal routing,
//! measurement function, signal generation and/or trigger shall cause existing readings to be
//! invalidated. For example, the sequence:
//!
//! ```scpi
//! INITiate;CONFIGure:VOLTage;FETCh:VOLTage?
//! ```
//!
//! would cause an error to be generated as the data was invalidated by the CONFigure
//! command. Reconfiguring the display or format blocks shall not have this effect.
//! ---
//! # Implementation
//! ![alt text](https://github.com/Atmelfan/scpi-rs/master/scpi_instrument/docs/measure_arch.png)
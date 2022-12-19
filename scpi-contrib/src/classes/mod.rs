pub mod digital_meter;
pub mod signal_switcher;

pub trait InstrumentClass {
    /// Return the `<instrument_specifier>` as specified in SCPI-99 "Instrument Classes chapter 1.4"
    fn instrument_specifier() -> &'static [u8];
}


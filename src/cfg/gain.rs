/// Configuration parameter to set the value for the programmable gain amplifier (PGA).
///
/// **Default:** 1
///
/// # See also
/// [`Configuration`], [`Channel`], [`Resolution`], [`ConversionTime`]
///
/// [`Configuration`]: crate::Configuration
/// [`Channel`]: crate::Channel
/// [`Resolution`]: crate::Resolution
/// [`ConversionTime`]: crate::ConversionTime
///
#[derive(Copy, Clone)]
#[cfg_attr(any(feature = "fmt", test), derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Gain {
    X1,
    X2,
    X4,
    X8,
}

impl Gain {

    pub const fn multiplier(&self) -> i32 {
        return match self {
            Gain::X1 => 1,
            Gain::X2 => 2,
            Gain::X4 => 4,
            Gain::X8 => 8,
        }
    }

    pub const fn mask(&self) -> u8 {
        match self {
            Gain::X1 => 0b00,
            Gain::X2 => 0b01,
            Gain::X4 => 0b10,
            Gain::X8 => 0b11,
        }
    }
}

impl Default for Gain {
    fn default() -> Self {
        Gain::X1
    }
}

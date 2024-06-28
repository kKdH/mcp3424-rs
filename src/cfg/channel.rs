/// Configuration parameter to select the device's channel.
///
/// **Default:** Channel 1
///
/// # See also
/// [`Configuration`], [`Gain`], [`Resolution`], [`ConversionTime`]
///
/// [`Configuration`]: crate::Configuration
/// [`Gain`]: crate::Gain
/// [`Resolution`]: crate::Resolution
/// [`ConversionTime`]: crate::ConversionTime
///
#[derive(Copy, Clone)]
#[cfg_attr(any(feature = "fmt", test), derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channel {
    Channel1,
    Channel2,
    Channel3,
    Channel4,
}

impl Channel {

    pub const fn mask(&self) -> u8 {
        match self {
            Channel::Channel1 => 0b00,
            Channel::Channel2 => 0b01,
            Channel::Channel3 => 0b10,
            Channel::Channel4 => 0b11,
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Channel::Channel1
    }
}

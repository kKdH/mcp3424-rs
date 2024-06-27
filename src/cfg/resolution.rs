/// Configuration parameter to set the resolution used to convert an analogue value.
///
/// **Default:** 12 bits
///
/// # See also
/// [`Configuration`], [`Channel`], [`Gain`], [`ConversionTime`]
///
/// [`Configuration`]: crate::Configuration
/// [`Channel`]: crate::Channel
/// [`Gain`]: crate::Gain
/// [`ConversionTime`]: crate::ConversionTime
///
#[derive(Copy, Clone)]
#[cfg_attr(feature = "fmt", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Resolution {
    /// A resolution of 12 bits results in a sampling rate of 240 samples per second.
    TwelveBits,
    /// A resolution of 14 bits results in a sampling rate of 60 samples per second.
    FourteenBits,
    /// A resolution of 16 bits results in a sampling rate of 15 samples per second.
    SixteenBits,
    /// A resolution of 18 bits results in a sampling rate of 3.75 samples per second.
    EighteenBits,
}

impl Resolution {

    /// Returns the number of bytes required for a sample.
    pub(crate) const fn bytes(&self) -> usize {
        match self {
            Resolution::TwelveBits |
            Resolution::FourteenBits |
            Resolution::SixteenBits => 2,
            Resolution::EighteenBits => 3,
        }
    }

    /// Returns the number of bits used to sample a value.
    pub(crate) const fn bits(&self) -> usize {
        match self {
            Resolution::TwelveBits => 12,
            Resolution::FourteenBits => 14,
            Resolution::SixteenBits => 16,
            Resolution::EighteenBits => 18,
        }
    }

    pub(crate) const fn mask(&self) -> u8 {
        match self {
            Resolution::TwelveBits => 0b00,
            Resolution::FourteenBits => 0b01,
            Resolution::SixteenBits => 0b10,
            Resolution::EighteenBits => 0b11,
        }
    }

    pub(crate) const fn sign_bit(&self) -> u32 {
        match self {
            Resolution::TwelveBits => 0x800,
            Resolution::FourteenBits => 0x2000,
            Resolution::SixteenBits => 0x8000,
            Resolution::EighteenBits => 0x20000,
        }
    }

    pub(crate) const fn sign_extend(&self) -> u32 {
        match self {
            Resolution::TwelveBits => 0xFFFFF000,
            Resolution::FourteenBits => 0xFFFFC000,
            Resolution::SixteenBits => 0xFFFF0000,
            Resolution::EighteenBits => 0xFFFC0000,
        }
    }

    /// Returns the minimum output codes
    pub(crate) const fn min(&self) -> i32 {
        match self {
            Resolution::TwelveBits => -2048,
            Resolution::FourteenBits => -8192,
            Resolution::SixteenBits => -32768,
            Resolution::EighteenBits => -131072,
        }
    }

    /// Returns the maximum output codes
    pub(crate) const fn max(&self) -> i32 {
        match self {
            Resolution::TwelveBits => 2047,
            Resolution::FourteenBits => 8191,
            Resolution::SixteenBits => 32767,
            Resolution::EighteenBits => 131071,
        }
    }

    /// Returns the time required for an A/D conversion in μs.
    pub(crate) const fn conversion_time_us(&self) -> u32 {
        match self {
            Resolution::TwelveBits =>   4167,  // 240 SPS
            Resolution::FourteenBits => 16667, // 60 SPS
            Resolution::SixteenBits =>  66667, // 15 SPS
            Resolution::EighteenBits => 266667 // 3.75 SPS
        }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution::TwelveBits
    }
}

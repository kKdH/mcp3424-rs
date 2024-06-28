use crate::{Channel, Gain, Resolution};

/// Mode independent user configuration.
///
/// A [`Configuration`] specifies the following device parameters:
/// * [`Channel`]: The device's input channel.
/// * [`Resolution`]: The resolution and sampling rate.
/// * [`Gain`]: The on-board programmable gain amplifier (PGA).
///
/// The MCP342[2/3/4] provides a constant conversion time only depending on the configured
/// [`Resolution`]. To tweak timings when reading the device's output buffer the driver's assumed
/// [`ConversionTime`] can be adjusted.
///
#[derive(Clone)]
#[cfg_attr(any(feature = "fmt", test), derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Configuration {
    pub channel: Channel,
    pub resolution: Resolution,
    pub gain: Gain,
    pub conversion_time: ConversionTime,
}

impl Configuration {

    pub fn new(channel: Channel, resolution: Resolution, gain: Gain, conversion_time: ConversionTime) -> Self {
        Self { channel, resolution, gain, conversion_time }
    }

    pub fn with_channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn with_gain(mut self, gain: Gain) -> Self {
        self.gain = gain;
        self
    }

    pub fn with_conversion_time(mut self, conversion_time: ConversionTime) -> Self {
        self.conversion_time = conversion_time;
        self
    }

    pub fn conversion_time_us(&self) -> u32 {
        match self.conversion_time {
            ConversionTime::Absolute(value) => value,
            ConversionTime::Offset(value) => self.resolution.conversion_time_us().saturating_add_signed(value)
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            channel: Channel::Channel1,
            resolution: Resolution::TwelveBits,
            gain: Gain::X1,
            conversion_time: ConversionTime::Offset(0)
        }
    }
}

/// Configuration parameter to adjust the driver's assumed conversion time.
///
/// **Default:** `ConversionTime::Offset(0)`
///
/// # See also
/// [`Configuration`], [`Channel`], [`Gain`], [`Resolution`]
///
#[derive(Copy, Clone)]
#[cfg_attr(any(feature = "fmt", test), derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConversionTime {
    /// Sets conversion time in µs to the specified value.
    Absolute(u32),
    /// Applies an offset in µs to the driver's assumed conversion time.
    Offset(i32)
}

impl Default for ConversionTime {
    fn default() -> Self {
        ConversionTime::Offset(0)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {

    use googletest::prelude::*;
    use rstest::rstest;
    use crate::config::{Configuration, ConversionTime};
    use crate::Resolution;

    #[rstest]
    fn conversion_time_us_should_return_an_absolute_value() -> Result<()> {

        let configuration = Configuration {
            conversion_time: ConversionTime::Absolute(42),
            ..Configuration::default()
        };

        verify_that!(configuration.conversion_time_us(), eq(42))?;

        Ok(())
    }

    #[rstest]
    fn conversion_time_us_should_return_a_relative_value() -> Result<()> {

        let mut configuration = Configuration {
            resolution: Resolution::TwelveBits,
            ..Configuration::default()
        };

        verify_that!(configuration.conversion_time_us(), eq(4167))?;

        configuration.conversion_time = ConversionTime::Offset(1337);

        verify_that!(configuration.conversion_time_us(), eq(5504))?;

        configuration.conversion_time = ConversionTime::Offset(0);

        verify_that!(configuration.conversion_time_us(), eq(4167))?;

        configuration.conversion_time = ConversionTime::Offset(-2167);

        verify_that!(configuration.conversion_time_us(), eq(2000))?;

        configuration.conversion_time = ConversionTime::Offset(-5000);

        verify_that!(configuration.conversion_time_us(), eq(0))?;

        Ok(())
    }
}

/// Error type used by the driver.
#[cfg_attr(feature = "fmt", derive(Debug))]
pub enum Error<BusError>
where
    BusError: embedded_hal_async::i2c::Error
{
    /// Indicates a communication error on the I2C bus.
    BusError(BusError),
    /// Indicates saturation of the converted value.
    IllegalValue { value: i32, min: i32, max: i32},
    /// Indicates that the device's output buffer does not contain new data.
    NotReady,
}

#[cfg(feature = "defmt")]
impl <BusError> defmt::Format for Error<BusError>
where
    BusError: embedded_hal_async::i2c::Error + defmt::Format
{
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::BusError(cause) => defmt::write!(f, "A bus error occurred: {}", cause),
            Error::IllegalValue { value, min, max} => defmt::write!(f, "The measured value '{}' exceeds the valid bounds: {} ≤ {} ≤ {}", value, min, value, max),
            Error::NotReady => defmt::write!(f, "No new data available"),
        }
    }
}

#[cfg(feature = "fmt")]
impl <BusError> core::fmt::Display for Error<BusError>
where
    BusError: embedded_hal_async::i2c::Error + core::fmt::Display
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::BusError(cause) => core::write!(f, "A bus error occurred: {}", cause),
            Error::IllegalValue { value, min, max} => core::write!(f, "The measured value '{}' exceeds the valid bounds: {} ≤ {} ≤ {}", value, min, value, max),
            Error::NotReady => core::write!(f, "No new data available"),
        }
    }
}

impl <BusError> From<BusError> for Error<BusError>
where
    BusError: embedded_hal_async::i2c::Error
{
    fn from(error: BusError) -> Self {
        Error::BusError(error)
    }
}

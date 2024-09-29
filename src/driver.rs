use crate::cfg::Cfg;
use crate::{mode, Error};
use core::marker::PhantomData;
use core::ops::Not;
use embedded_hal_async::i2c::SevenBitAddress;

/// Driver for the MCP342[2/3/4].
///
/// Depending on the enabled crate features and the specified [`mode::Mode`], there are different measure
/// functions available.
///
/// # See also
/// [`mode::ContinuousMode`], [`mode::MultiShotMode`], [`mode::OneShotMode`]
///
pub struct MCP3424<I2c, BusError, Delay, Mode> {
    pub(crate) i2c: I2c,
    pub(crate) address: u8,
    pub(crate) delay: Delay,
    pub(crate) mode: Mode,
    _phantom: PhantomData<BusError>
}

impl <I2c, BusError, Delay, Mode> MCP3424<I2c, BusError, Delay, Mode>
where
    I2c: embedded_hal_async::i2c::I2c,
    BusError: embedded_hal_async::i2c::Error,
    Delay: embedded_hal_async::delay::DelayNs,
    Error<BusError>: From<<I2c as embedded_hal_async::i2c::ErrorType>::Error>,
    Mode: mode::Mode
{
    const REFERENCE_VOLTAGE: i64 = 2_048_000_000;
    const REFERENCE_VOLTAGE_X2: i64 = Self::REFERENCE_VOLTAGE * 2;

    pub fn new(i2c: I2c, address: SevenBitAddress, delay: Delay, mode: Mode) -> Self {
        Self {
            i2c,
            address,
            delay,
            mode,
            _phantom: PhantomData,
        }
    }

    pub(crate) async fn read(&mut self, read: &mut [u8]) -> Result<(), I2c::Error> {
        self.i2c.read(self.address, read).await
    }

    pub(crate) async fn write(&mut self, write: &[u8]) -> Result<(), I2c::Error> {
        self.i2c.write(self.address, write).await
    }

    pub(crate) fn convert(buffer: &[u8; 4]) -> Result<f32, Error<BusError>> {

        let cfg = if buffer[3] & 0b1100 == 0b1100 {
            Cfg::from(buffer[3])
        }
        else {
            Cfg::from(buffer[2])
        };

        if cfg.ready.not() {
            return Err(Error::NotReady)
        }

        let value = {
            let mut value = 0_u32;
            for i in 0..cfg.resolution.bytes() {
                value <<= 8;
                value |= buffer[i] as u32
            }
            if value & cfg.resolution.sign_bit() != 0 {
                value |= cfg.resolution.sign_extend()
            }
            value as i32
        };

        let min = cfg.resolution.min();
        let max = cfg.resolution.max();

        if value > min && value < max {
            Ok((value as i64 * Self::REFERENCE_VOLTAGE_X2 / (1 << cfg.resolution.bits())) as f32 / (1_000_000 * cfg.gain.multiplier()) as f32)
        }
        else {
            Err(Error::IllegalValue { value, min, max })
        }
    }

    pub fn into_inner(self) -> (I2c, Delay) {
        (self.i2c, self.delay)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use embedded_hal_async::i2c::ErrorKind;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::i2c::Mock as I2c;
    use googletest::prelude::*;
    use rstest::rstest;

    use super::MCP3424;
    use crate::OneShotMode;

    type Testee = MCP3424<I2c, ErrorKind, NoopDelay, OneShotMode>;

    #[rstest]
    #[case([0, 0, 0b00000000, 0], 0.0)]
    #[case([0, 1, 0b00000000, 0], 1.0)] // LSB @ 12 bit
    #[case([255, 255, 0b00000000, 0], -1.0)] // -LSB @ 12 bit
    #[case([0, 1, 0b00000100, 0], 0.25)] // LSB @ 14 bit
    #[case([255, 255, 0b00000100, 0], -0.25)] // LSB @ 14 bit
    #[case([0, 1, 0b00001000, 0], 0.0625)] // LSB @ 16 bit
    #[case([255, 255, 0b00001000, 0], -0.0625)] // LSB @ 16 bit
    #[case([0, 0, 1, 0b00001100], 0.015625)] // LSB @ 18 bit
    #[case([255, 255, 255, 0b00001100], -0.015625)] // LSB @ 18 bit
    #[case([0, 1, 0b00000001, 0], 0.5)] // gain 2x
    #[case([0, 1, 0b00000010, 0], 0.25)] // gain 4x
    #[case([0, 1, 0b00000011, 0], 0.125)] // gain 8x
    fn A_MCP3424_should_convert_an_output_code_into_millivolts(
        #[case] code: [u8; 4],
        #[case] expected: f32
    ) -> Result<()> {

        assert_that!(&Testee::convert(&code), ok(eq(&expected)));

        Ok(())
    }

    #[rstest]
    #[case([8, 0, 0b00000000, 0], -2048)]
    #[case([7, 255, 0b00000000, 0], 2047)]
    #[case([32, 0, 0b00000100, 0], -8192)]
    #[case([31, 255, 0b00000100, 0], 8191)]
    #[case([128, 0, 0b00001000, 0], -32768)]
    #[case([127, 255, 0b00001000, 0], 32767)]
    #[case([2, 0, 0, 0b00001100], -131072)]
    #[case([1, 255, 255, 0b00001100], 131071)]
    fn A_MCP3424_should_return_an_error_if_the_code_represents_an_invalid_value(
        #[case] code: [u8; 4],
        #[case] value: i32,
    ) -> Result<()> {

        assert_that!(Testee::convert(&code), err(matches_pattern!(crate::Error::IllegalValue { value: eq(&value) })));

        Ok(())
    }

    #[rstest]
    fn A_MCP3424_should_return_an_error_if_the_ready_bit_is_set() -> Result<()> {

        let code = [0, 0, 0b10000000, 0];

        assert_that!(Testee::convert(&code), err(anything()));

        Ok(())
    }
}

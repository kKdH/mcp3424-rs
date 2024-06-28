#[cfg(feature = "stream")]
use futures::{Stream, StreamExt};

use crate::{cfg, Configuration, Error, MCP3424, Mode};
use crate::cfg::Cfg;

/// A mode where the device continuously converts data.
///
/// In Continuous mode, the measure functions prompt the device to continuously convert data and
/// update its output buffer automatically. Therefor each subsequent call to a measure function just
/// reads the mose resent data.
///
/// # Example
///
/// ```
///# use embedded_hal_mock::eh1::i2c::Mock as I2C;
///# use embedded_hal_mock::eh1::i2c::Transaction;
///# use embedded_hal_mock::eh1::delay::NoopDelay as Delay;
/// use mcp3424::{MCP3424, Configuration, ContinuousMode, Channel, Error};
///
///# let mut i2c = I2C::new(&[
///#     Transaction::write(0x68, vec![0b00010000]),
///#     Transaction::read(0x68, vec![0, 1, 0, 0]),
///#     Transaction::read(0x68, vec![0, 2, 0, 0]),
///# ]);
///#
/// let mut adc = MCP3424::new(i2c, 0x68, Delay, ContinuousMode::new(&Configuration::default()));
///
///# let _: Result<(), Error<_>> = async_std::task::block_on(async {
/// println!("First value: {:?}", adc.measure().await?);
/// println!("Second value: {:?}", adc.measure().await?);
///# Ok(())
///# });
///# adc.into_inner().0.done();
/// ```
///
/// # See also
/// [`OneShotMode`], [`MultiShotMode`]
///
/// [`OneShotMode`]: crate::OneShotMode
/// [`MultiShotMode`]: crate::MultiShotMode
///
pub struct ContinuousMode {
    cfg: Cfg,
    delay: u32,
    initialized: bool,
}

impl ContinuousMode {

    pub fn new(configuration: &Configuration) -> Self {
        Self {
            cfg: cfg(&configuration, Cfg::default()),
            delay: configuration.conversion_time_us(),
            initialized: false,
        }
    }
}

impl Mode for ContinuousMode {}

impl <I2c, BusError, Delay> MCP3424<I2c, BusError, Delay, ContinuousMode>
where
    I2c: embedded_hal_async::i2c::I2c,
    BusError: embedded_hal_async::i2c::Error,
    Delay: embedded_hal_async::delay::DelayNs,
    Error<BusError>: From<<I2c as embedded_hal_async::i2c::ErrorType>::Error>
{
    /// Updates the driver's configuration and applies it immediately to the device.
    pub async fn configure(&mut self, configuration: &Configuration) -> Result<(), Error<BusError>> {
        self.mode.cfg = cfg(configuration, Cfg::default());
        self.mode.delay = configuration.conversion_time_us();
        self.write(&[self.mode.cfg.as_byte()]).await?;
        Ok(())
    }

    #[cfg(not(feature = "uom"))]
    pub async fn measure(&mut self) -> Result<f32, Error<BusError>> {
        self.do_measure().await
    }

    #[cfg(feature = "uom")]
    pub async fn measure(&mut self) -> Result<uom::si::f32::ElectricPotential, Error<BusError>> {
        self.do_measure().await
            .map(uom::si::f32::ElectricPotential::new::<uom::si::electric_potential::millivolt>)
    }

    /// Returns a stream of measured values.
    ///
    /// This variant of measure function prompts the device to continuously convert data and returns
    /// a stream providing the last converted value each time the stream gets polled. If there is no
    /// new data available, an [`Error::NotReady`] will be returned by the stream.
    ///
    #[cfg(all(feature = "stream", not(feature = "uom")))]
    pub async fn measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<f32, Error<BusError>>> + 'a, Error<BusError>> {
        self.do_measure_stream().await
    }

    /// Returns a stream of measured values.
    ///
    /// This variant of measure function prompts the device to continuously convert data and returns
    /// a stream providing the last converted value each time the stream gets polled. If there is no
    /// new data available, an [`Error::NotReady`] will be returned by the stream.
    ///
    #[cfg(all(feature = "stream", feature = "uom"))]
    pub async fn measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<uom::si::f32::ElectricPotential, Error<BusError>>> + 'a, Error<BusError>> {
        self.do_measure_stream().await
            .map(|stream| stream
                .map(|result| result
                    .map(uom::si::f32::ElectricPotential::new::<uom::si::electric_potential::millivolt>)))
    }

    async fn do_measure(&mut self) -> Result<f32, Error<BusError>> {

        let mut buffer = [0_u8; 4];

        if !self.mode.initialized {
            self.write(&[self.mode.cfg.as_byte()]).await?;
            self.delay.delay_us(self.mode.delay).await;
            self.mode.initialized = true;
        }

        self.read(&mut buffer).await?;

        Ok(Self::convert(&buffer)?)
    }

    #[cfg(feature = "stream")]
    async fn do_measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<f32, Error<BusError>>> + 'a, Error<BusError>> {

        let buffer = [0_u8; 4];

        self.write(&[self.mode.cfg.as_byte()]).await?;

        self.delay.delay_us(self.mode.delay).await;

        Ok(futures::stream::unfold((self, cfg, buffer), |(device, cfg, mut buffer)| async move {
            let result = device.read(&mut buffer).await
                .map_err(Error::from)
                .and_then(|_| Self::convert(&buffer));
            Some((result, (device, cfg, buffer)))
        }))
    }
}

pub(crate) fn cfg(configuration: &Configuration, mut cfg: Cfg) -> Cfg {
    cfg.set_values_from_configuration(&configuration);
    cfg.mode = cfg::Mode::Continuous;
    cfg
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::i2c::{Mock as I2c, Transaction};
    use googletest::prelude::*;
    use rstest::{fixture, rstest};
    #[cfg(feature = "uom")]
    use uom::si::electric_potential::millivolt;
    #[cfg(feature = "uom")]
    use uom::si::f32::ElectricPotential;

    use crate::{Channel, Configuration, ContinuousMode, Gain, MCP3424, Resolution};
    use crate::cfg::{Cfg, Mode};

    #[fixture]
    fn expected_cfg() -> Cfg {
        Cfg {
            ready: true,
            channel: Channel::Channel1,
            resolution: Resolution::TwelveBits,
            mode: Mode::Continuous,
            gain: Gain::X1
        }
    }

    #[rstest]
    async fn When_in_ContinuousMode_a_MCP3424_should_trigger_conversion(expected_cfg: Cfg) -> Result<()> {

        let returned_cfg = Cfg {
            ready: true,
            ..expected_cfg
        };

        let i2c = I2c::new(&[
            Transaction::write(0x68, vec![expected_cfg.as_byte()]),
            Transaction::read(0x68, vec![0, 1, returned_cfg.as_byte(), 0]),
            Transaction::read(0x68, vec![0, 2, returned_cfg.as_byte(), 0]),
            Transaction::read(0x68, vec![0, 3, returned_cfg.as_byte(), 0]),
        ]);

        let mut testee = MCP3424::new(i2c, 0x68, NoopDelay, ContinuousMode::new(&Configuration::default()));

        #[cfg(feature = "uom")]
        {
            assert_that!(testee.measure().await, ok(eq(ElectricPotential::new::<millivolt>(1.0))));
            assert_that!(testee.measure().await, ok(eq(ElectricPotential::new::<millivolt>(2.0))));
            assert_that!(testee.measure().await, ok(eq(ElectricPotential::new::<millivolt>(3.0))));
        }

        #[cfg(not(feature = "uom"))]
        {
            assert_that!(testee.measure().await, ok(eq(1.0)));
            assert_that!(testee.measure().await, ok(eq(2.0)));
            assert_that!(testee.measure().await, ok(eq(3.0)));
        }

        testee.i2c.done();

        Ok(())
    }
}

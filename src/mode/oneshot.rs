#[cfg(feature = "stream")]
use futures::{Stream, StreamExt};

use crate::{cfg, Configuration, Error, MCP3424, Mode};
use crate::cfg::Cfg;
use crate::mode::oneshot;

/// A mode where the device executes a single conversion.
///
/// In One-Shot mode, the driver initiates a single conversion on each call of measure function and
/// waits for the result. In this mode the device enters a low current standby mode after a conversion
/// automatically until it receives another conversion command.
///
/// # Example
///
/// ```
///# use embedded_hal_mock::eh1::i2c::Mock as I2C;
///# use embedded_hal_mock::eh1::i2c::Transaction;
///# use embedded_hal_mock::eh1::delay::NoopDelay as Delay;
/// use mcp3424::{MCP3424, Configuration, OneShotMode};
///
///# let mut i2c = I2C::new(&[
///#     Transaction::write(0x68, vec![0b10000000]),
///#     Transaction::read(0x68, vec![0, 2, 0, 0]),
///# ]);
///#
/// let mut adc = MCP3424::new(i2c, 0x68, Delay, OneShotMode::new(&Configuration::default()));
///
///# async_std::task::block_on(async {
/// match adc.measure().await {
///     Ok(value) => println!("Measured value: {:?}", value),
///     Err(_) => println!("Failed to measure")
/// }
///# });
///# adc.into_inner().0.done();
/// ```
/// # See also
/// [`MultiShotMode`], [`ContinuousMode`]
///
/// [`MultiShotMode`]: crate::MultiShotMode
/// [`ContinuousMode`]: crate::ContinuousMode
///
pub struct OneShotMode {
    cfg: Cfg,
    delay: u32,
}

impl OneShotMode {

    pub fn new(configuration: &Configuration) -> Self {
        Self {
            cfg: oneshot::cfg(&configuration, Cfg::default()),
            delay: configuration.conversion_time_us(),
        }
    }
}

impl Mode for OneShotMode {}

impl <I2c, BusError, Delay> MCP3424<I2c, BusError, Delay, OneShotMode>
where
    I2c: embedded_hal_async::i2c::I2c,
    BusError: embedded_hal_async::i2c::Error,
    Delay: embedded_hal_async::delay::DelayNs,
    Error<BusError>: From<<I2c as embedded_hal_async::i2c::ErrorType>::Error>
{
    /// Updates the driver's configuration. The configuration is applied to the device lazily on
    /// the next measure call.
    pub fn configure(&mut self, configuration: &Configuration) {
        self.mode.cfg = cfg(configuration, Cfg::default());
        self.mode.delay = configuration.conversion_time_us();
    }

    /// Triggers a single conversion and awaits the result.
    #[cfg(not(feature = "uom"))]
    pub async fn measure(&mut self) -> Result<f32, Error<BusError>> {
        let mut buffer = [0_u8; 4];
        self.do_measure(&mut buffer).await
    }

    /// Triggers a single conversion and awaits the result.
    #[cfg(feature = "uom")]
    pub async fn measure(&mut self) -> Result<uom::si::f32::ElectricPotential, Error<BusError>> {
        let mut buffer = [0_u8; 4];
        self.do_measure(&mut buffer).await
            .map(uom::si::f32::ElectricPotential::new::<uom::si::electric_potential::millivolt>)
    }

    /// Returns a stream of measured values.
    ///
    /// This variant of measure function triggers a single conversion and awaits the result each
    /// time the stream gets polled.
    ///
    #[cfg(all(feature = "stream", not(feature = "uom")))]
    pub async fn measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<f32, Error<BusError>>> + 'a, Error<BusError>> {
        self.do_measure_stream().await
    }

    /// Returns a stream of measured values.
    ///
    /// This variant of measure function triggers a single conversion and awaits the result each
    /// time the stream gets polled.
    ///
    #[cfg(all(feature = "stream", feature = "uom"))]
    pub async fn measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<uom::si::f32::ElectricPotential, Error<BusError>>> + 'a, Error<BusError>> {
        self.do_measure_stream().await
            .map(|stream| stream
                .map(|result| result
                    .map(uom::si::f32::ElectricPotential::new::<uom::si::electric_potential::millivolt>)))
    }

    async fn do_measure(&mut self, buffer: &mut [u8; 4]) -> Result<f32, Error<BusError>> {

        self.write(&[self.mode.cfg.as_byte()]).await?;

        self.delay.delay_us(self.mode.delay).await;

        self.read(buffer).await?;

        Ok(Self::convert(&buffer)?)
    }

    #[cfg(feature = "stream")]
    async fn do_measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<f32, Error<BusError>>> + 'a, Error<BusError>> {

        let buffer = [0_u8; 4];

        Ok(futures::stream::unfold((self, buffer), |(this, mut buffer)| async move {
            let result = this.do_measure(&mut buffer).await;
            Some((result, (this, buffer)))
        }))
    }
}

pub(crate) fn cfg(configuration: &Configuration, mut cfg: Cfg) -> Cfg {
    cfg.set_values_from_configuration(&configuration);
    cfg.ready = false;
    cfg.mode = cfg::Mode::OneShot;
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

    use crate::{Channel, Configuration, Gain, MCP3424, OneShotMode, Resolution};
    use crate::cfg::{Cfg, Mode};

    #[fixture]
    fn expected_cfg() -> Cfg {
        Cfg {
            ready: false,
            channel: Channel::Channel1,
            resolution: Resolution::TwelveBits,
            mode: Mode::OneShot,
            gain: Gain::X1
        }
    }

    #[rstest]
    async fn When_in_OneShotMode_a_MCP3424_should_trigger_a_single_conversion(expected_cfg: Cfg) -> Result<()> {

        let returned_cfg = Cfg {
            ready: true,
            ..expected_cfg
        };

        let i2c = I2c::new(&[
            Transaction::write(0x68, vec![expected_cfg.as_byte()]),
            Transaction::read(0x68, vec![0, 1, returned_cfg.as_byte(), 0]),
        ]);

        let mut testee = MCP3424::new(i2c, 0x68, NoopDelay, OneShotMode::new(&Configuration::default()));

        #[cfg(feature = "uom")]
        assert_that!(&testee.measure().await, ok(eq(&ElectricPotential::new::<millivolt>(1.0))));

        #[cfg(not(feature = "uom"))]
        assert_that!(&testee.measure().await, ok(eq(&1.0)));

        testee.i2c.done();

        Ok(())
    }

    #[rstest]
    async fn When_in_OneShotMode_a_MCP3424_should_return_an_error_if_there_is_no_data_available(expected_cfg: Cfg) -> Result<()> {

        let returned_cfg = Cfg {
            ready: false,
            ..expected_cfg
        };

        let i2c = I2c::new(&[
            Transaction::write(0x68, vec![expected_cfg.as_byte()]),
            Transaction::read(0x68, vec![0, 0, returned_cfg.as_byte(), 0]),
        ]);

        let mut testee = MCP3424::new(i2c, 0x68, NoopDelay, OneShotMode::new(&Configuration::default()));

        assert_that!(testee.measure().await, err(anything()));

        testee.i2c.done();

        Ok(())
    }
}

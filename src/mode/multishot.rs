#[cfg(feature = "stream")]
use futures::{Stream, StreamExt};

use crate::{Configuration, Error, MCP3424, Mode};
use crate::cfg::Cfg;
use crate::mode::oneshot;

/// A mode which triggers a sequence of one-shot conversions.
///
/// The Multi-Shot mode is a variation of the [`OneShotMode`] where the measure functions initiate a
/// series of One-Shot conversions and return an array containing all values at once. In this mode
/// the device enters a low current standby mode after the last conversion automatically.
///
/// <div class="warning">
/// <b>Important:</b>
///
/// This mode <b>is not</b> a device feature and is completely implemented within software by triggering
/// multiple one-shot conversions sequentially. Therefor the returned results <b>do not</b> represent
/// values from ADC converted at the same time!
/// </div>
///
/// # Example
///
/// ```
///# use embedded_hal_mock::eh1::i2c::Mock as I2C;
///# use embedded_hal_mock::eh1::i2c::Transaction;
///# use embedded_hal_mock::eh1::delay::NoopDelay as Delay;
/// use mcp3424::{MCP3424, Configuration, MultiShotMode, Channel};
///
///# let mut i2c = I2C::new(&[
///#     Transaction::write(0x68, vec![0b11000000]),
///#     Transaction::read(0x68, vec![0, 1, 0, 0]),
///#     Transaction::write(0x68, vec![0b11100000]),
///#     Transaction::read(0x68, vec![0, 2, 0, 0]),
///# ]);
///#
/// let mut adc = MCP3424::new(i2c, 0x68, Delay, MultiShotMode::new(&[
///     Configuration::default()
///         .with_channel(Channel::Channel3),
///     Configuration::default()
///         .with_channel(Channel::Channel4)
/// ]));
///
///# async_std::task::block_on(async {
/// match adc.measure().await {
///     Ok(value) => {
///         println!("Measured value of channel 3: {:?}", value[0]);
///         println!("Measured value of channel 4: {:?}", value[1]);
///     }
///     Err(cause) => println!("Failed to measure, due to error: {}", cause)
/// }
///# });
///# adc.into_inner().0.done();
/// ```
///
/// # See also
/// [`OneShotMode`], [`ContinuousMode`]
///
/// [`OneShotMode`]: crate::OneShotMode
/// [`ContinuousMode`]: crate::ContinuousMode
///
pub struct MultiShotMode<const N: usize> {
    cfgs: [Cfg; N],
    delays: [u32; N],
}

impl <const N: usize> MultiShotMode<N> {

    pub fn new(configurations: &[Configuration; N]) -> Self {
        let (cfgs, delays) = cfgs_and_delays(configurations);
        Self {
            cfgs,
            delays,
        }
    }
}

impl <const N: usize> Mode for MultiShotMode<N> {}

impl <I2c, BusError, Delay, const N: usize> MCP3424<I2c, BusError, Delay, MultiShotMode<N>>
where
    I2c: embedded_hal_async::i2c::I2c,
    BusError: embedded_hal_async::i2c::Error,
    Delay: embedded_hal_async::delay::DelayNs,
    Error<BusError>: From<<I2c as embedded_hal_async::i2c::ErrorType>::Error>
{
    /// Updates the driver's configuration. The configuration is applied to the device lazily on
    /// the next measure call.
    pub fn configure(&mut self, configurations: &[Configuration]) {
        let (cfgs, delays) = cfgs_and_delays(&configurations);
        self.mode.cfgs = cfgs;
        self.mode.delays = delays;
    }

    /// Triggers multiple conversions and awaits all results.
    #[cfg(not(feature = "uom"))]
    pub async fn measure(&mut self) -> Result<[f32; N], Error<BusError>> {
        let mut buffer = [0_u8; 4];
        self.do_measure(&mut buffer).await
    }

    /// Triggers multiple conversions and awaits all results.
    #[cfg(feature = "uom")]
    pub async fn measure(&mut self) -> Result<[uom::si::f32::ElectricPotential; N], Error<BusError>> {
        let mut buffer = [0_u8; 4];
        self.do_measure(&mut buffer).await
            .map(|values| values
                .map(uom::si::f32::ElectricPotential::new::<uom::si::electric_potential::millivolt>))
    }

    /// Returns a stream of multiple measured values.
    ///
    /// This variant of measure function triggers a sequence of conversions and awaits their results
    /// each time the stream gets polled.
    ///
    #[cfg(all(feature = "stream", not(feature = "uom")))]
    pub async fn measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<[f32; N], Error<BusError>>> + 'a, Error<BusError>> {
        self.do_measure_stream().await
    }

    /// Returns a stream of multiple measured values.
    ///
    /// This variant of measure function triggers a sequence of conversions and awaits their results
    /// each time the stream gets polled.
    ///
    #[cfg(all(feature = "stream", feature = "uom"))]
    pub async fn measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<[uom::si::f32::ElectricPotential; N], Error<BusError>>> + 'a, Error<BusError>> {
        self.do_measure_stream().await
            .map(|stream| stream
                .map(|result| result
                    .map(|values| values
                        .map(uom::si::f32::ElectricPotential::new::<uom::si::electric_potential::millivolt>))))
    }

    async fn do_measure(&mut self, buffer: &mut [u8; 4]) -> Result<[f32; N], Error<BusError>> {

        let mut values = [0_f32; N];

        for i in 0..N {
            self.write(&[self.mode.cfgs[i].as_byte()]).await?;
            self.delay.delay_us(self.mode.delays[i]).await;
            self.read(buffer).await?;
            values[i] = Self::convert(&buffer)?;
        }

        Ok(values)
    }

    #[cfg(feature = "stream")]
    async fn do_measure_stream<'a>(&'a mut self) -> Result<impl Stream<Item=Result<[f32; N], Error<BusError>>> + 'a, Error<BusError>> {

        let buffer = [0_u8; 4];

        Ok(futures::stream::unfold((self, buffer), |(this, mut buffer)| async move {
            let result = this.do_measure(&mut buffer).await;
            Some((result, (this, buffer)))
        }))
    }
}

fn cfgs_and_delays<const N: usize>(configurations: &[Configuration]) -> ([Cfg; N], [u32; N]) {
    let mut cfgs = [Cfg::default(); N];
    let mut delays = [0_u32; N];
    for i in 0..N {
        cfgs[i] = oneshot::cfg(&configurations[i], cfgs[i]);
        delays[i] = configurations[i].conversion_time_us()
    }
    (cfgs, delays)
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

    use crate::{Channel, Configuration, Gain, MCP3424, MultiShotMode, Resolution};
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
    async fn When_in_MultiShotMode_a_MCP3424_should_trigger_a_multiple_conversions(expected_cfg: Cfg) -> Result<()> {

        let expected_cfg_1 = Cfg {
            channel: Channel::Channel2,
            ..expected_cfg
        };

        let expected_cfg_2 = Cfg {
            resolution: Resolution::SixteenBits,
            ..expected_cfg
        };

        let returned_cfg_1 = Cfg {
            ready: true,
            ..expected_cfg_1
        };

        let returned_cfg_2 = Cfg {
            ready: true,
            ..expected_cfg_2
        };

        let i2c = I2c::new(&[
            Transaction::write(0x68, vec![expected_cfg_1.as_byte()]),
            Transaction::read(0x68, vec![0, 1, returned_cfg_1.as_byte(), 0]),
            Transaction::write(0x68, vec![expected_cfg_2.as_byte()]),
            Transaction::read(0x68, vec![0, 2, returned_cfg_2.as_byte(), 0]),
        ]);

        let mut testee = MCP3424::new(i2c, 0x68, NoopDelay, MultiShotMode::new(&[
            Configuration::default().with_channel(Channel::Channel2),
            Configuration::default().with_resolution(Resolution::SixteenBits)
        ]));

        let result = testee.measure().await;

        #[cfg(feature = "uom")]
        assert_that!(result, ok(eq([ElectricPotential::new::<millivolt>(1.0), ElectricPotential::new::<millivolt>(0.125)])));

        #[cfg(not(feature = "uom"))]
        assert_that!(result, ok(eq([1.0, 0.125])));

        testee.i2c.done();

        Ok(())
    }

    #[rstest]
    async fn When_in_MultiShotMode_a_MCP3424_should_return_an_error_if_there_is_no_data_available(expected_cfg: Cfg) -> Result<()> {

        let returned_cfg_1 = Cfg {
            ready: true,
            ..expected_cfg
        };

        let returned_cfg_2 = Cfg {
            ready: false,
            ..expected_cfg
        };

        let i2c = I2c::new(&[
            Transaction::write(0x68, vec![expected_cfg.as_byte()]),
            Transaction::read(0x68, vec![0, 1, returned_cfg_1.as_byte(), 0]),
            Transaction::write(0x68, vec![expected_cfg.as_byte()]),
            Transaction::read(0x68, vec![0, 2, returned_cfg_2.as_byte(), 0]),
        ]);

        let mut testee = MCP3424::new(i2c, 0x68, NoopDelay, MultiShotMode::new(&[
            Configuration::default(),
            Configuration::default()
        ]));

        let result = testee.measure().await;

        assert_that!(result, err(anything()));

        testee.i2c.done();

        Ok(())
    }
}

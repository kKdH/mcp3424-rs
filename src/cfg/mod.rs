use core::ops::Not;

pub use crate::cfg::channel::Channel;
pub use crate::cfg::gain::Gain;
pub use crate::cfg::mode::Mode;
pub use crate::cfg::resolution::Resolution;
use crate::Configuration;

mod gain;
mod channel;
mod mode;
mod resolution;

#[derive(Copy, Clone)]
#[cfg_attr(any(feature = "fmt", test), derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Cfg {
    pub ready: bool,
    pub channel: Channel,
    pub mode: Mode,
    pub resolution: Resolution,
    pub gain: Gain,
}

impl Cfg {

    pub fn set_values_from_configuration(&mut self, other: &Configuration) {
        self.channel = other.channel;
        self.gain = other.gain;
        self.resolution = other.resolution;
    }

    pub fn as_byte(&self) -> u8 {
        let mut result = 0_u8;
        result |= self.ready.not() as u8;
        result <<= 2;
        result |= self.channel.mask();
        result <<= 1;
        result |= self.mode.mask();
        result <<= 2;
        result |= self.resolution.mask();
        result <<= 2;
        result |= self.gain.mask();
        result
    }
}

impl From<u8> for Cfg {

    fn from(value: u8) -> Self {
        let ready = {
            value >> 7 == 0
        };
        let channel = {
            match (value >> 5) & 0b11 {
                0b00 => Channel::Channel1,
                0b01 => Channel::Channel2,
                0b10 => Channel::Channel3,
                0b11 => Channel::Channel4,
                _ => unreachable!()
            }
        };
        let mode = {
            if (value >> 4) & 1 == 1 {
                Mode::Continuous
            }
            else {
                Mode::OneShot
            }
        };
        let resolution = {
            match (value >> 2) & 0b11 {
                0b00 => Resolution::TwelveBits,
                0b01 => Resolution::FourteenBits,
                0b10 => Resolution::SixteenBits,
                0b11 => Resolution::EighteenBits,
                _ => unreachable!()
            }
        };
        let gain = {
            match value & 0b11 {
                0b00 => Gain::X1,
                0b01 => Gain::X2,
                0b10 => Gain::X4,
                0b11 => Gain::X8,
                _ => unreachable!()
            }
        };
        Self {
            ready,
            channel,
            mode,
            resolution,
            gain
        }
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            ready: true,
            channel: Channel::default(),
            mode: Mode::default(),
            resolution: Resolution::default(),
            gain: Gain::default(),
        }
    }
}

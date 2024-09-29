//!
//! This crate provides an async Rust driver for the MCP342[2/3/4] ADC, based on the
//! [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) traits.
//!
//! # Modes
//!
//! | Mode                                        | Description                                                                                                                       |
//! | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
//! | [OneShot](`crate::mode::OneShotMode`)       | Instructs the device to do a single conversion and awaits the result.                                                             |
//! | [Continuous](`crate::mode::ContinuousMode`) | Instructs the device to do conversions continuously. Every subsequent call will read the last available value only.               |
//! | [MultiShot](`crate::mode::MultiShotMode`)   | A variation of the [`OneShotMode`]. The measure functions execute a series of one-shot conversions and return all values at once. |
//!
//! # MCP3422 and MCP3423
//! In contrast to the MCP3424, the MCP3422 and MCP3423 provide only two channels instead of four.
//! But these device offer the same I2C interface. Therefor this crate can also be used for MCP3422
//! and MCP3423 devices with the exception that these devices treat [`Channel::Channel3`] as
//! [`Channel::Channel1`] and [`Channel::Channel4`] as [`Channel::Channel2`].
//!
//! # Crate Features
//!
//! Enable or disable features according to your needs and in order to optimize for compile time and space.
//!
//! | Feature   | Default  | Description                                                                                                                    |
//! | --------- |:--------:| ------------------------------------------------------------------------------------------------------------------------------ |
//! | defmt     | &#x2717; | When enabled, certain types will provide an implementation for the [`defmt::Format`] trait.                                    |
//! | fmt       | &#x2714; | When enabled, certain types will provide an implementation for [`core::fmt::Debug`] and [`core::fmt::Display`] traits.         |
//! | stream    | &#x2717; | When enabled, the driver offers additional measure functions which return a [`futures::stream::Stream`].                       |
//! | uom       | &#x2717; | When enabled, all measure functions return the measured value as [`uom::si::f32::ElectricPotential`] instead of a plain `f32`. |
//!
//! <sup>&#x2714; enabled, &#x2717; disabled</sup>
//!
//! # UOM
//!
//! This driver integrates with the [uom](https://docs.rs/uom) crate which provides units of
//! measurement. After activating the corresponding `uom` feature all measure functions return the
//! measured value as [`uom::si::f32::ElectricPotential`] instead of a plain `f32`.
//!
//! [Read more](crate::doc::uom)
//!

#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub use crate::cfg::{Channel, Gain, Resolution};

pub use crate::config::{Configuration, ConversionTime};
pub use crate::driver::MCP3424;
pub use crate::error::Error;
pub use crate::mode::{ContinuousMode, Mode, MultiShotMode, OneShotMode};

mod cfg;
mod config;
mod driver;
mod error;
mod mode;

#[cfg(doc)]
pub mod doc;

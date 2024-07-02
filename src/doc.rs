//! This module delivers supplementary documentation.

#[cfg(feature = "uom")]
pub mod uom {
//! # UOM
//!
//! At first glance [uom] may seam cumbersome to use and its benefits may unclear. The following
//! paragraphs explain how [uom] can be used and which benefits it offers.
//!
//! ## Example
//!
//! This example uses a voltage divider to illustrate the usage of [uom].
//!
//! ```text
//!
//! I -->    +--------------+         +--------------+
//! ---------|      R1      |---------|      R2      |---------
//!          +--------------+         +--------------+
//!
//!
//!    +----------- U1 ----------+---------- U2 ----------+
//!
//!    +----------------------- Us -----------------------+
//! ```
//!
//! Given the total voltage `Us` and the resistance values `R1` and `R2`. We can compute the total
//! current by `I = Us / (R1 + R2)`. Further, we can use `I` to compute `U1` by `U1 = I * R1` and
//! `U2` by `U2 = I * R2`.
//!
//! ```
//! use uom::fmt::DisplayStyle::Abbreviation;
//! use uom::si::f32::{ElectricPotential, ElectricalResistance, ElectricCurrent};
//! use uom::si::electric_potential::{millivolt, volt};
//! use uom::si::electric_current::ampere;
//! use uom::si::electrical_resistance::ohm;
//!
//! let fmt_current = ElectricCurrent::format_args(ampere, Abbreviation);
//! let fmt_voltage = ElectricPotential::format_args(volt, Abbreviation);
//!
//! let Us = ElectricPotential::new::<volt>(5.0);
//! let R1 = ElectricalResistance::new::<ohm>(150.0);
//! let R2 = ElectricalResistance::new::<ohm>(100.0);
//!
//! let I: ElectricCurrent = Us / (R1 + R2);
//! let U1: ElectricPotential = I * R1;
//! let U2: ElectricPotential = I * R2;
//! let U12 = U1 + U2;
//!
//! assert_eq!(U12, Us);
//!
//! assert_eq!("I = 0.02 A", format!("I = {}", fmt_current.with(I)));
//! assert_eq!("U1 = 3 V", format!("U1 = {}", fmt_voltage.with(U1)));
//! assert_eq!("U2 = 2 V", format!("U2 = {}", fmt_voltage.with(U2)));
//! ```
//!
//! Additionally, [uom] offers some utilities for formatting and printing quantities. Quantity types
//! provide a `format_args` function to create formating structures.
//!
//! ## Benefits
//!
//! [uom] protects us from errors and mistakes when working with values.
//!
//! * For example wrongly combining values of different quantities.
//!   ```compile_fail
//!   # use uom::si::f32::{ElectricalResistance, ElectricCurrent};
//!   # use uom::si::electric_current::ampere;
//!   # use uom::si::electrical_resistance::ohm;
//!   let I = ElectricCurrent::new::<ampere>(1.0);
//!   let R1 = ElectricalResistance::new::<ohm>(150.0);
//!
//!   let U1 = I + R1; // will not compile
//!   ```
//! * Or using values of different scale
//!   ```should_panic
//!   # use uom::si::f32::ElectricPotential;
//!   # use uom::si::electric_potential::{volt, millivolt};
//!   let U1 = ElectricPotential::new::<volt>(2.0);
//!   let U2 = ElectricPotential::new::<millivolt>(3.0);
//!   let U12 = U1 + U2;
//!
//!   assert_eq!(U12.get::<volt>(), 5.0); // will fail because U12 = 2.003
//!   ```
//!
//! [uom]: https://docs.rs/uom
//!
}

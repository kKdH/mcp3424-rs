[![github](https://img.shields.io/github/actions/workflow/status/kkdh/mcp3424-rs/build.yaml?branch=main&style=for-the-badge&logo=githubactions&label=build)](https://github.com/kkdh/mcp3424-rs/actions?query=branch%3Amain)
[![docs.rs](https://img.shields.io/docsrs/mcp3424?style=for-the-badge&logo=rust)](https://docs.rs/mcp3424)
[![crates.io](https://img.shields.io/crates/v/mcp3424?style=for-the-badge&logo=rust)](https://crates.io/crates/mcp3424)

# MCP3424

This crate provides an async Rust driver for the MCP342[2/3/4] ADC, based on the embedded-hal traits.

The [MCP3422](https://www.microchip.com/en-us/product/mcp3424), [MCP3423](https://www.microchip.com/en-us/product/mcp3424) and [MCP3424](https://www.microchip.com/en-us/product/mcp3424) are low-noise, high accuracy delta-sigma A/D converter with differential inputs, on-board precision 2.048V reference voltage and up to 18 bits of resolution. The devices offer a two-wire I2C compatible serial interface.

## Status

- [x] One-Shot conversions.
- [x] Continuous conversions.
- [x] Configuration:
  - [x] Programmable gain amplifier (PGA)
  - [x] Resolution / Sample Rate
  - [x] Channel
- [x] Async API
  - [x] Future based
  - [x] Stream based (optional)
- [x] [Defmt](https://crates.io/crates/defmt) integration (optional)
- [x] [UOM](https://crates.io/crates/uom) integration (optional)

## Appetizer

Trigger a one-shot conversion and awaits the result:
```rust
use mcp3424::{MCP3424, Configuration, OneShotMode};

let mut adc = MCP3424::new(i2c, 0x68, Delay, OneShotMode::new(&Configuration::default()));

match adc.measure().await {
    Ok(value) => println!("Measured value: {:?}", value),
    Err(_) => println!("Failed to measure"),
}
```

## [Documentation](https://docs.rs/mcp3424)

## License
Licensed using the [Apache License Version 2.0](LICENSE).

## Contributing

All contributions are welcome. Any contribution intentionally submitted for inclusion in this crate by you, as defined in the [Apache-2.0 license](LICENSE), shall be licensed without any additional terms or conditions.

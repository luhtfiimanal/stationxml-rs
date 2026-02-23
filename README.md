# stationxml-rs

Pure Rust FDSN StationXML and SeisComP SC3ML reader/writer. Zero `unsafe`, zero C dependencies.

[![Crates.io](https://img.shields.io/crates/v/stationxml-rs.svg)](https://crates.io/crates/stationxml-rs)
[![docs.rs](https://docs.rs/stationxml-rs/badge.svg)](https://docs.rs/stationxml-rs)
[![CI](https://github.com/luhtfiimanal/stationxml-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/luhtfiimanal/stationxml-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)

## What is StationXML?

[FDSN StationXML](https://www.fdsn.org/xml/station/) is the standard XML format for seismic station metadata, used by earthquake monitoring networks worldwide (IRIS, FDSN, BMKG, etc.). It describes station locations, instrument responses, poles & zeros, and data logger configurations.

[SeisComP SC3ML](https://www.seiscomp.de/doc/base/concepts/inventory.html) is an alternative XML format used by the [SeisComP](https://www.seiscomp.de/) earthquake monitoring system. It stores the same information in a flat, reference-based structure rather than FDSN's nested hierarchy.

This crate provides a **format-agnostic inventory model** with pluggable backends for both formats, enabling read, write, and cross-format conversion.

## Features

- **FDSN StationXML 1.2** read and write
- **SeisComP SC3ML 0.6--0.13** read and write
- **Cross-format conversion**: FDSN <-> SC3ML via shared inventory model
- **Auto-detect** format from root XML element
- **Full instrument response**: poles & zeros, FIR coefficients, stage gains, sensitivity
- **Builder pattern** for constructing inventories programmatically
- **Sensor library**: embedded database of common seismometers (GS-11D, Trillium, etc.)
- **ADC conversion helpers**: counts <-> voltage <-> physical units
- **Zero unsafe** -- no FFI, no transmute, no raw pointers
- **Zero C dependencies** -- pure Rust, compiles anywhere `rustc` runs

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
stationxml-rs = "0.2"
```

### Read with auto-detection

```rust
use stationxml_rs::read_from_file;

// Auto-detects FDSN or SC3ML from the XML content
let inv = read_from_file("station.xml").unwrap();

for net in &inv.networks {
    for sta in &net.stations {
        println!("{}.{} ({}, {})", net.code, sta.code, sta.latitude, sta.longitude);
        for ch in &sta.channels {
            println!("  {}.{} {} Hz", ch.code, ch.location_code, ch.sample_rate);
        }
    }
}
```

### Read and write with explicit format

```rust
use stationxml_rs::{Fdsn, Sc3ml, StationXmlFormat};

// Read FDSN StationXML
let inv = Fdsn::read_from_str("<FDSNStationXML ...>...</FDSNStationXML>").unwrap();

// Write as SC3ML (cross-format conversion)
let sc3ml_xml = Sc3ml::write_to_string(&inv).unwrap();
```

### Build an inventory from scratch

```rust
use stationxml_rs::{InventoryBuilder, write_to_string, Fdsn};

let inv = InventoryBuilder::new("MyOrg")
    .network("IU", |net| {
        net.description("Global Seismograph Network")
           .station("ANMO", -34.9462, 138.5866, 48.0, |sta| {
               sta.description("Albuquerque, New Mexico")
                  .channel("BHZ", "00", |ch| {
                      ch.sample_rate(20.0)
                        .azimuth(0.0)
                        .dip(-90.0)
                  })
           })
    })
    .build();

let xml = write_to_string::<Fdsn>(&inv).unwrap();
```

## API Overview

```rust
use stationxml_rs::{
    // Top-level functions
    read_from_file, read_from_str,      // auto-detect format
    read_from_file_as, write_to_file,   // explicit format
    write_to_string, detect_format,     // utility

    // Format backends
    Fdsn, Sc3ml, StationXmlFormat,      // format markers + trait
    Format,                             // enum: Fdsn | Sc3ml

    // Core inventory types
    Inventory, Network, Station, Channel,
    Response, ResponseStage,
    PolesZeros, Coefficients, FIR,
    Equipment, Site,

    // Builder
    InventoryBuilder,

    // Helpers
    AdcConversion,                      // counts <-> physical units
    SensorEntry, find_sensor,           // sensor database
    load_sensor_library,

    // Error handling
    StationXmlError, Result,
};
```

| Type | Description |
|------|-------------|
| `Inventory` | Top-level container for all station metadata |
| `Network` | Seismic network (e.g. "IU", "GE") with stations |
| `Station` | Station with location, site info, and channels |
| `Channel` | Channel with orientation, sample rate, and response |
| `Response` | Instrument response with sensitivity and stage chain |
| `ResponseStage` | Single response stage (poles/zeros, coefficients, or FIR) |
| `PolesZeros` | Analog/digital transfer function (Laplace or Z-transform) |
| `FIR` | Finite Impulse Response filter coefficients |
| `Coefficients` | Gain-only stage (e.g. ADC stage) |
| `Equipment` | Sensor or data logger description |
| `Fdsn` | FDSN StationXML 1.2 format backend |
| `Sc3ml` | SeisComP SC3ML 0.6--0.13 format backend |
| `InventoryBuilder` | Fluent builder for constructing inventories |
| `AdcConversion` | ADC conversion helpers (counts, voltage, physical) |

## Supported Formats

| Format | Namespace | Read | Write |
|--------|-----------|------|-------|
| FDSN StationXML 1.2 | `http://www.fdsn.org/xml/station/1` | Yes | Yes |
| SeisComP SC3ML 0.6--0.13 | `http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/*` | Yes | Yes |

### SC3ML Reference Resolution

SC3ML uses a flat structure with `publicID` references. The reader automatically resolves:

```text
<Inventory>
  <sensor publicID="Sensor#GS11D" response="ResponsePAZ#GS11D">  -->  Channel.sensor
  <datalogger publicID="Datalogger#PB24">                        -->  Channel.data_logger
  <responsePAZ publicID="ResponsePAZ#GS11D">                     -->  Response stage 1
  <responseFIR publicID="ResponseFIR#DECIM2">                     -->  Response FIR stage
  <network>
    <station>
      <sensorLocation>
        <stream sensor="Sensor#GS11D" datalogger="Datalogger#PB24">  (references above)
```

The writer reverses this: deduplicating shared sensors/responses into top-level definitions.

## Architecture

```
src/
  lib.rs           -- crate root, re-exports, top-level read/write functions
  inventory.rs     -- core types: Inventory, Network, Station, Channel, Response
  format.rs        -- StationXmlFormat trait, Format enum, detect_format()
  error.rs         -- StationXmlError enum (thiserror)
  datetime.rs      -- shared ISO 8601 datetime parsing/formatting
  builder.rs       -- InventoryBuilder fluent API
  sensor.rs        -- embedded sensor database (GS-11D, Trillium, etc.)
  conversion.rs    -- ADC conversion helpers
  fdsn/
    mod.rs         -- pub struct Fdsn; impl StationXmlFormat
    types.rs       -- FDSN-specific XML serde structs
    reader.rs      -- FDSN XML -> Inventory
    writer.rs      -- Inventory -> FDSN XML
  sc3ml/
    mod.rs         -- pub struct Sc3ml; impl StationXmlFormat
    types.rs       -- SC3ML-specific XML serde structs
    reader.rs      -- SC3ML XML -> Inventory (reference resolution)
    writer.rs      -- Inventory -> SC3ML XML (deduplication)
```

### Design Decisions

- **Format-agnostic core**: `Inventory` types don't depend on any XML format
- **Trait-based backends**: `StationXmlFormat` trait for each format (Fdsn, Sc3ml)
- **Auto-detect**: `detect_format()` inspects root XML element name
- **Roundtrip-safe**: read -> write -> read produces identical `Inventory`
- **ObsPy-compatible**: output XML is parseable by `obspy.read_inventory()`
- **Embedded sensor DB**: `data/sensors.json` compiled into binary via `include_str!`

### TDD with ObsPy

Test vectors are generated by Python/ObsPy scripts, ensuring compatibility with the de facto reference implementation:

```bash
cd pyscripts && uv run python -m pyscripts.generate_vectors
cargo test
```

## Development

```bash
cargo build                     # build
cargo test                      # all tests (109 tests)
cargo clippy -- -D warnings     # lint (strict)
cargo fmt -- --check            # format check
cargo doc --no-deps --open      # browse docs locally
```

## References

- [FDSN StationXML 1.2 Schema](https://www.fdsn.org/xml/station/fdsn-station-1.2.xsd)
- [FDSN StationXML Documentation](https://docs.fdsn.org/projects/stationxml/en/latest/)
- [SeisComP SC3ML Inventory](https://www.seiscomp.de/doc/base/concepts/inventory.html)
- [ObsPy Inventory](https://docs.obspy.org/packages/autogen/obspy.core.inventory.inventory.Inventory.html)

## Sister Projects

- [miniseed-rs](https://github.com/luhtfiimanal/miniseed-rs) -- Pure Rust miniSEED v2/v3 decoder/encoder
- [seedlink-rs](https://github.com/luhtfiimanal/seedlink-rs) -- Pure Rust SeedLink client

## License

Apache-2.0

# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2025-02-23

### Added
- `Serialize` derive on `SensorEntry` for JSON API usage (#1)
- `Serialize, Deserialize` derives on all core inventory types: `Inventory`, `Network`, `Station`, `Channel`, `Response`, `Equipment`, `Site`, `Units`, and all response stage types (#2)

## [0.2.0] - 2025-02-23

### Added
- **SeisComP SC3ML 0.6--0.13** read and write support
- SC3ML reader with automatic `publicID` reference resolution
- SC3ML writer with sensor/datalogger/response deduplication
- Cross-format conversion: FDSN StationXML <-> SC3ML via shared inventory model
- Auto-detection of SC3ML format from root XML element
- Shared datetime parsing/formatting utilities (`src/datetime.rs`)
- 51 new tests (13 integration + 38 unit) -- total: 109

## [0.1.0] - 2025-02-22

### Added
- **FDSN StationXML 1.2** read and write support
- Format-agnostic core inventory model (`Inventory`, `Network`, `Station`, `Channel`, `Response`)
- Full instrument response: poles & zeros, FIR, coefficients, stage gains, sensitivity
- `InventoryBuilder` fluent API for constructing inventories
- Embedded sensor library (`data/sensors.json`) with common seismometers
- `AdcConversion` helpers for counts <-> voltage <-> physical unit conversion
- Auto-detect format from root XML element (`detect_format()`)
- `StationXmlFormat` trait for pluggable format backends
- 58 tests (8 integration + 47 unit + 3 doctests)

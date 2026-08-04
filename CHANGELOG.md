# Changelog

All notable changes to this project will be documented in this file.

## [0.2.4] - 2026-08-04

### Fixed
- Gain-only response stages (preamp, PGA, A/D conversion factor) can now declare their units. Previously they were written as a bare `<StageGain>`, which left them without `InputUnits`/`OutputUnits` — FDSN carries units only inside a transfer function element. That broke the unit chain of the response, so `evalresp` rejected the **entire channel** with *"units mismatch between stages"* and `obspy`'s `remove_response()` failed. Inventories affected by this looked correct but could not be used for deconvolution at all.
- SC3ML writer no longer drops the datalogger gain when the A/D stage is gain-only. It now finds the A/D stage by its units (output `COUNTS`, input something else) instead of requiring a `Coefficients` element.

### Added
- `ResponseStage::input_units` / `ResponseStage::output_units` — units for a stage that has no transfer function element. On write they are emitted as an empty `<PolesZeros>` (no poles, no zeros, `A0 = 1`, so `H(s) = 1` exactly) that carries them; the reader lifts it back onto the stage, so write -> read -> write is stable. An empty `<Coefficients>` cannot serve this role: ObsPy rejects the analog form and evalresp demands a decimation blockette for the digital form.
- `ResponseStage::gain_only()` constructor and `ResponseStage::resolved_units()`, which reads a stage's effective units whether they live on the element or on the stage.
- `Units::new()` and `Units::with_description()` constructors.
- `Default` derive on `ResponseStage`.
- `docs/guide/03-instrument-response.md`: section on gain-only stages and the unit-chain rule.

## [0.2.3] - 2026-07-30

### Changed
- Upgraded `quick-xml` to 0.41 (RUSTSEC-2026-0194 / RUSTSEC-2026-0195). No API changes.

## [0.2.2] - 2026-04-06

### Fixed
- SC3ML reader now populates `Decimation.input_sample_rate` for FIR and digital `Coefficients` stages by walking the digital filter chain forward from the pre-decimation rate. Previously, every parsed FIR stage had `input_sample_rate = 0.0`, which caused downstream consumers (e.g. `ppsd-rs`) to silently produce NaN response evaluations.

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

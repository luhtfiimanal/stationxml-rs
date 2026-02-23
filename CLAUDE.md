# CLAUDE.md — stationxml-rs

Pure Rust FDSN StationXML and SeisComP SC3ML reader/writer. Zero unsafe, zero C dependency. Apache 2.0.

Sister project of [miniseed-rs](https://github.com/luhtfiimanal/miniseed-rs) and [seedlink-rs](https://github.com/luhtfiimanal/seedlink-rs).

## CRITICAL

- **Diskusi dulu sebelum implementasi** — investigasi, jelaskan, diskusikan, baru code
- **Jangan push tanpa persetujuan user**
- **stdout workaround**: `script -q -c "cargo test" /dev/null` (Claude Code bug)
- **Zero unsafe** — no FFI, no transmute, no raw pointers

## Scope

**v0.1** — FDSN StationXML 1.2:
- Core inventory model (format-agnostic): `Inventory`, `Network`, `Station`, `Channel`, `Response`
- FDSN StationXML 1.2 read + write (via `quick-xml` + `serde`)
- Builder pattern for constructing inventories
- Sensor library (embedded JSON database of common seismometers)
- ADC conversion helpers (counts ↔ physical units)
- Auto-detect format from root XML element

**v0.2** — SeisComP SC3ML 0.13:
- SC3ML read + write (flat ID-based structure ↔ nested hierarchy)
- Cross-format conversion: FDSN ↔ SC3ML via core model

## Module Structure

```
src/
  lib.rs           -- re-exports, top-level read/write/detect functions
  error.rs         -- StationXmlError enum (thiserror)
  inventory.rs     -- core types: Inventory, Network, Station, Channel, Response, ...
  builder.rs       -- builder pattern API
  sensor.rs        -- SensorEntry, load_sensor_library(), find_sensor()
  conversion.rs    -- AdcConversion helpers (counts ↔ voltage ↔ physical)
  format.rs        -- StationXmlFormat trait, Format enum, detect_format()
  fdsn/
    mod.rs         -- pub struct Fdsn; impl StationXmlFormat
    types.rs       -- FDSN-specific XML serde structs (internal)
    reader.rs      -- XML → fdsn types → Inventory
    writer.rs      -- Inventory → fdsn types → XML
  sc3ml/           -- (v0.2)
    mod.rs
    types.rs
    reader.rs
    writer.rs

data/
  sensors.json     -- built-in sensor database

tests/
  fdsn_roundtrip.rs
  fixtures/
    fdsn_sample.xml
```

## Commands

```bash
cargo build                          # build
cargo test                           # test all
cargo test fdsn::                    # test single module
cargo clippy -- -D warnings          # lint (strict)
cargo fmt -- --check                 # format check

# pyscripts (TDD vector generation)
cd pyscripts && uv sync
cd pyscripts && uv run python -m pyscripts.generate_vectors
cd pyscripts && uv run ruff check src
cd pyscripts && uv run basedpyright src
```

## TDD Strategy

Python/ObsPy generates reference StationXML files → Rust tests assert roundtrip correctness.

1. `cd pyscripts && uv run python -m pyscripts.generate_vectors`
2. Write Rust test loading `tests/fixtures/*.xml` — RED
3. Implement Rust code — GREEN
4. Validate: `obspy.read_inventory()` can parse our output

Test fixtures in `tests/fixtures/` (checked in).
Generated test vectors in `pyscripts/test_vectors/` (gitignored, regenerate locally).

## Code Quality

- `cargo fmt` + `cargo clippy -- -D warnings` — pre-commit enforced
- `thiserror` for all error types
- No `unsafe` anywhere
- pyscripts: `basedpyright` strict + `ruff`

## Key Design Decisions

- **Format-agnostic core**: `Inventory` types don't depend on any XML format
- **Trait-based formats**: `StationXmlFormat` trait for each backend (Fdsn, Sc3ml)
- **Auto-detect**: `detect_format()` inspects root XML element
- **Roundtrip-safe**: read → write → read should produce identical `Inventory`
- **ObsPy-compatible**: output XML parseable by `obspy.read_inventory()`
- **Embedded sensor DB**: `data/sensors.json` compiled into binary via `include_str!`

## FDSN StationXML 1.2 Format

Root element: `<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" schemaVersion="1.2">`

Hierarchy: FDSNStationXML → Network → Station → Channel → Response

Key XML conventions:
- `#[serde(rename = "@code")]` → XML attributes
- `#[serde(rename = "$text")]` → text content
- Namespace `http://www.fdsn.org/xml/station/1` required in output

## SeisComP SC3ML 0.13 Format

Root element: `<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13" version="0.13">`

Key differences from FDSN:
- Flat structure: sensor, datalogger, response at top-level, referenced via `publicID`
- Attribute-heavy: lat/lon/elevation as attributes, not child elements
- `sensorLocation` = FDSN `location_code`, `stream` = FDSN `channel`
- Response stored separately (`responsePAZ`, `responseFIR`, `responsePolynomial`)

## References

- [FDSN StationXML 1.2 Schema](https://www.fdsn.org/xml/station/fdsn-station-1.2.xsd)
- [FDSN StationXML Documentation](https://docs.fdsn.org/projects/stationxml/en/latest/)
- [SeisComP SC3ML Schema](https://www.seiscomp.de/doc/base/concepts/inventory.html)
- [ObsPy Inventory](https://docs.obspy.org/packages/autogen/obspy.core.inventory.inventory.Inventory.html)
- [quick-xml serde guide](https://docs.rs/quick-xml/latest/quick_xml/de/index.html)

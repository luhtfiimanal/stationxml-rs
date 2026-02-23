# stationxml-rs — Seismic Station Metadata for Rust

## Tujuan

Crate Rust untuk membaca dan menulis metadata stasiun seismik dalam berbagai format XML. Core model yang format-agnostic, dengan multiple format backends.

**Prinsip utama:**
1. **Satu inventory model** — types internal yang merepresentasikan Network/Station/Channel/Response, tidak terikat format tertentu
2. **Multiple formats** — baca dari format apapun, tulis ke format apapun
3. **Roundtrip-safe** — baca file lalu tulis kembali tanpa kehilangan data
4. **Interoperable** — output harus compatible dengan ObsPy (`obspy.read_inventory()`) dan SeisComP

## Arsitektur

```
                    ┌──────────────────────┐
                    │   Inventory (core)   │
                    │  Network / Station   │
                    │  Channel / Response  │
                    └──────┬───────────────┘
                           │
              ┌────────────┼────────────────┐
              ▼            ▼                ▼
     ┌────────────┐  ┌──────────┐   ┌────────────┐
     │    FDSN    │  │  SC3ML   │   │  (future)  │
     │ StationXML │  │ SeisComP │   │  QuakeML   │
     │    1.2     │  │   0.13   │   │   etc.     │
     └────────────┘  └──────────┘   └────────────┘
```

**Read:** Format-specific XML → `Inventory` (core)
**Write:** `Inventory` (core) → Format-specific XML
**Convert:** Read format A → `Inventory` → Write format B

## Scope

### v0.1 — FDSN StationXML
- Full FDSN StationXML 1.2 schema (read + write)
- Core inventory model
- Builder pattern
- Sensor library bawaan (embedded JSON)
- Conversion helpers (ADC counts ↔ physical units)

### v0.2 — SeisComP SC3ML
- Read/write SeisComP SC3ML 0.13 format
- Convert FDSN ↔ SC3ML via core model

### Future
- SeisComP inventory XML (older format)
- IRIS NRL integration
- FDSN web service client (fetching remote inventories)

## Dependencies

- `quick-xml` dengan feature `serialize` — XML serde
- `serde` + `serde_json` — struct serialization
- `chrono` — datetime handling
- Minimal dependencies, no async runtime

## Core Types (format-agnostic)

Ini adalah internal model yang dipakai semua format backends. Semua field mengikuti konvensi FDSN tapi tidak terikat XML structure.

```rust
/// Top-level inventory — container untuk semua metadata
pub struct Inventory {
    pub source: String,
    pub sender: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub networks: Vec<Network>,
}

pub struct Network {
    pub code: String,              // e.g. "GE", "IU", "XX"
    pub description: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub stations: Vec<Station>,
}

pub struct Station {
    pub code: String,              // e.g. "PBUMI"
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,            // meters above sea level
    pub site: Site,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub creation_date: Option<DateTime<Utc>>,
    pub channels: Vec<Channel>,
}

pub struct Site {
    pub name: String,
    pub description: Option<String>,
    pub town: Option<String>,
    pub county: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
}

pub struct Channel {
    pub code: String,              // FDSN 3-char: "SHZ", "SHN", "SHE"
    pub location_code: String,     // e.g. "00", ""
    pub latitude: Option<f64>,     // override station lat if different
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub depth: f64,                // meters below surface
    pub azimuth: f64,              // degrees from north (0=N, 90=E)
    pub dip: f64,                  // degrees from horizontal (-90=up, 0=horiz, 90=down)
    pub sample_rate: f64,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sensor: Option<Equipment>,
    pub data_logger: Option<Equipment>,
    pub response: Option<Response>,
}

pub struct Equipment {
    pub equipment_type: Option<String>,    // "Geophone", "Datalogger"
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub installation_date: Option<DateTime<Utc>>,
    pub removal_date: Option<DateTime<Utc>>,
}

pub struct Response {
    pub instrument_sensitivity: Option<InstrumentSensitivity>,
    pub stages: Vec<ResponseStage>,
}

pub struct InstrumentSensitivity {
    pub value: f64,                // overall sensitivity (counts per input unit)
    pub frequency: f64,            // frequency at which sensitivity is valid
    pub input_units: Units,
    pub output_units: Units,
}

pub struct Units {
    pub name: String,              // e.g. "M/S", "V", "COUNTS"
    pub description: Option<String>,
}

pub struct ResponseStage {
    pub number: u32,               // stage number (1-based)
    pub stage_gain: Option<StageGain>,
    pub poles_zeros: Option<PolesZeros>,
    pub coefficients: Option<Coefficients>,
    pub decimation: Option<Decimation>,
}

pub struct StageGain {
    pub value: f64,
    pub frequency: f64,
}

// PolesZeros, Coefficients, Decimation — implement as needed
```

## Format Trait

Setiap format implement trait ini:

```rust
pub trait StationXmlFormat {
    /// Read from XML string → Inventory
    fn read_from_str(xml: &str) -> Result<Inventory, StationXmlError>;

    /// Read from bytes
    fn read_from_bytes(bytes: &[u8]) -> Result<Inventory, StationXmlError>;

    /// Write Inventory → XML string
    fn write_to_string(inventory: &Inventory) -> Result<String, StationXmlError>;
}
```

### FDSN StationXML 1.2

```rust
pub struct Fdsn;

impl StationXmlFormat for Fdsn {
    // ...
}
```

Internal XML structs (serde + quick-xml) di-map ke/dari core `Inventory`:

```rust
// Hanya untuk FDSN XML serialization, BUKAN public API
mod fdsn_xml {
    #[derive(Serialize, Deserialize)]
    #[serde(rename = "FDSNStationXML")]
    struct FDSNStationXML {
        #[serde(rename = "@xmlns")]
        xmlns: String,
        #[serde(rename = "@schemaVersion")]
        schema_version: String,
        #[serde(rename = "Source")]
        source: String,
        // ...
    }
}
```

### SeisComP SC3ML 0.13

```rust
pub struct Sc3ml;

impl StationXmlFormat for Sc3ml {
    // ...
}
```

SC3ML punya structure berbeda (flat, ID-based references):
```xml
<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13" version="0.13">
  <Inventory>
    <sensor publicID="Sensor/GS-11D" name="GS-11D" response="ResponsePAZ/GS-11D"/>
    <datalogger publicID="Datalogger/CS5532" name="CS5532"/>
    <network publicID="Network/XX" code="XX">
      <station publicID="Station/XX.PBUMI" code="PBUMI">
        <sensorLocation publicID="SensorLocation/XX.PBUMI.00" code="00">
          <stream publicID="Stream/XX.PBUMI.00.SHZ" code="SHZ"
                  sensor="Sensor/GS-11D" datalogger="Datalogger/CS5532"/>
        </sensorLocation>
      </station>
    </network>
    <responsePAZ publicID="ResponsePAZ/GS-11D" .../>
  </Inventory>
</seiscomp>
```

Conversion logic: flatten/unflatten antara SC3ML ID-based refs dan FDSN nested hierarchy.

## Top-level API

```rust
use stationxml::{Inventory, Format};

// === Read ===

// Auto-detect format from content
let inv = stationxml::read_from_file("station.xml")?;
let inv = stationxml::read_from_str(xml_string)?;

// Explicit format
let inv = stationxml::read_from_file_as::<Fdsn>("station.xml")?;
let inv = stationxml::read_from_file_as::<Sc3ml>("inventory.xml")?;

// === Write ===

// Write as specific format
stationxml::write_to_file::<Fdsn>("output.xml", &inv)?;
stationxml::write_to_file::<Sc3ml>("output_sc3.xml", &inv)?;

let xml_string = stationxml::write_to_string::<Fdsn>(&inv)?;

// === Convert ===

// FDSN → SC3ML (read one, write other)
let inv = stationxml::read_from_file_as::<Fdsn>("station.xml")?;
stationxml::write_to_file::<Sc3ml>("inventory_sc3.xml", &inv)?;
```

### Auto-detect

Format detection berdasarkan root element:
- `<FDSNStationXML ...>` → FDSN
- `<seiscomp ...>` → SC3ML

```rust
pub fn detect_format(xml: &str) -> Option<Format>;

pub enum Format {
    Fdsn,
    Sc3ml,
}
```

## Builder API

```rust
let inv = Inventory::builder()
    .source("Pena Bumi")
    .network("XX", |net| {
        net.description("Local Test Network")
           .station("PBUMI", |sta| {
               sta.latitude(-7.7714)
                  .longitude(110.3776)
                  .elevation(150.0)
                  .site_name("Yogyakarta Seismic Shelter")
                  .channel("SHZ", "00", |ch| {
                      ch.azimuth(0.0)
                        .dip(-90.0)
                        .sample_rate(100.0)
                        .sensor(Equipment { ... })
                  })
                  .channel("SHN", "00", |ch| {
                      ch.azimuth(0.0).dip(0.0).sample_rate(100.0)
                  })
                  .channel("SHE", "00", |ch| {
                      ch.azimuth(90.0).dip(0.0).sample_rate(100.0)
                  })
           })
    })
    .build();

// Write ke format manapun
let fdsn_xml = stationxml::write_to_string::<Fdsn>(&inv)?;
let sc3_xml = stationxml::write_to_string::<Sc3ml>(&inv)?;
```

## Sensor Library

Built-in JSON database of common seismometer/accelerometer specs.

```rust
pub struct SensorEntry {
    pub model: String,              // e.g. "GS-11D", "SM-6"
    pub manufacturer: String,       // e.g. "Geospace", "ION"
    pub sensor_type: String,        // "Geophone", "Accelerometer", "Broadband"
    pub sensitivity: f64,           // V/(m/s) or V/(m/s²)
    pub sensitivity_unit: String,   // "M/S", "M/S**2"
    pub frequency_range: (f64, f64), // Hz
    pub natural_period: Option<f64>, // seconds
    pub damping: Option<f64>,       // fraction of critical
}

/// Load built-in sensor library
pub fn load_sensor_library() -> Vec<SensorEntry>;

/// Find sensor by model name (case-insensitive)
pub fn find_sensor(model: &str) -> Option<&SensorEntry>;
```

Sensor list awal:

| Model | Manufacturer | Type | Sensitivity | Unit | Freq Range |
|-------|-------------|------|-------------|------|------------|
| GS-11D | Geospace | Geophone (Vertical) | 32.0 | M/S | 4.5–500 Hz |
| GS-11D 3C | Geospace | Geophone (3-Component) | 32.0 | M/S | 4.5–500 Hz |
| SM-6 | ION/Sensor Nederland | Geophone | 28.8 | M/S | 4.5–500 Hz |
| L-4C | Sercel/Mark Products | Geophone | 171.0 | M/S | 1.0–500 Hz |
| STS-2 | Streckeisen | Broadband | 1500.0 | M/S | 0.00833–50 Hz |
| CMG-3T | Güralp | Broadband | 1500.0 | M/S | 0.00833–50 Hz |
| CMG-40T | Güralp | Broadband | 800.0 | M/S | 0.033–50 Hz |
| Trillium 120 | Nanometrics | Broadband | 1202.5 | M/S | 0.00833–100 Hz |
| PE-6/B | Beijing Seis | Short Period | 200.0 | M/S | 1.0–100 Hz |

## Conversion Helpers

Helper functions untuk konversi ADC ↔ physical units.

```rust
/// Parameters for ADC count ↔ voltage conversion
pub struct AdcConversion {
    pub full_scale_voltage: f64,   // FSR in Volts (e.g. 5.0)
    pub max_count: f64,            // 2^(bits-1) - 1 (e.g. 8388607 for 24-bit)
    pub pga_gain: f64,             // external PGA gain (e.g. 1.0)
    pub adc_gain: f64,             // digital gain in ADC (e.g. 1.0)
}

impl AdcConversion {
    /// Raw ADC count → input voltage (before PGA)
    pub fn count_to_voltage(&self, count: f64) -> f64;

    /// Input voltage → raw ADC count
    pub fn voltage_to_count(&self, voltage: f64) -> f64;

    /// Raw ADC count → physical unit (using sensor sensitivity)
    pub fn count_to_physical(&self, count: f64, sensitivity: f64) -> f64;

    /// Overall sensitivity: counts per physical unit
    pub fn overall_sensitivity(&self, sensor_sensitivity: f64) -> f64;
}
```

Formula:
```
voltage = (count / max_count) * full_scale_voltage / (pga_gain * adc_gain)
physical = voltage / sensor_sensitivity
overall_sensitivity = max_count * pga_gain * adc_gain * sensor_sensitivity / full_scale_voltage
```

## Crate Structure

```
stationxml-rs/
├── Cargo.toml
├── README.md
├── LICENSE
├── PROMPT.md                  ← this file
├── src/
│   ├── lib.rs                 ← re-exports, top-level read/write/detect functions
│   ├── error.rs               ← StationXmlError enum
│   ├── inventory.rs           ← core types: Inventory, Network, Station, Channel, ...
│   ├── builder.rs             ← builder pattern API
│   ├── sensor.rs              ← SensorEntry, load_sensor_library()
│   ├── conversion.rs          ← AdcConversion helpers
│   ├── format.rs              ← StationXmlFormat trait, Format enum, detect_format()
│   ├── fdsn/
│   │   ├── mod.rs             ← pub struct Fdsn; impl StationXmlFormat
│   │   ├── types.rs           ← FDSN-specific XML serde structs (internal)
│   │   ├── reader.rs          ← XML → fdsn types → Inventory
│   │   └── writer.rs          ← Inventory → fdsn types → XML
│   └── sc3ml/                 ← v0.2
│       ├── mod.rs
│       ├── types.rs
│       ├── reader.rs
│       └── writer.rs
├── data/
│   └── sensors.json           ← built-in sensor database
└── tests/
    ├── fdsn_roundtrip.rs
    ├── sc3ml_roundtrip.rs
    ├── cross_format.rs        ← FDSN → SC3ML → FDSN roundtrip
    ├── obspy_compat.rs
    └── fixtures/
        ├── fdsn_sample.xml
        └── sc3ml_sample.xml
```

## XML Format Details

### FDSN StationXML 1.2

```xml
<?xml version="1.0" encoding="UTF-8"?>
<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1"
                schemaVersion="1.2">
  <Source>Pena Bumi</Source>
  <Created>2026-02-20T00:00:00Z</Created>
  <Network code="XX">
    <Station code="PBUMI">
      <Latitude>-7.7714</Latitude>
      <Longitude>110.3776</Longitude>
      <Elevation>150.0</Elevation>
      <Site><Name>Yogyakarta</Name></Site>
      <Channel code="SHZ" locationCode="00">
        <Azimuth>0.0</Azimuth>
        <Dip>-90.0</Dip>
        <SampleRate>100.0</SampleRate>
        <Sensor>
          <Type>Geophone</Type>
          <Model>GS-11D</Model>
          <Manufacturer>Geospace</Manufacturer>
        </Sensor>
        <Response>
          <InstrumentSensitivity>
            <Value>53721548.8</Value>
            <Frequency>15.0</Frequency>
            <InputUnits><Name>M/S</Name></InputUnits>
            <OutputUnits><Name>COUNTS</Name></OutputUnits>
          </InstrumentSensitivity>
        </Response>
      </Channel>
    </Station>
  </Network>
</FDSNStationXML>
```

quick-xml serde notes:
- `#[serde(rename = "@code")]` → XML attributes
- `#[serde(rename = "$text")]` → text content
- Namespace `http://www.fdsn.org/xml/station/1` harus ada di output

### SeisComP SC3ML 0.13

```xml
<?xml version="1.0" encoding="UTF-8"?>
<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13" version="0.13">
  <Inventory>
    <sensor publicID="Sensor/GS-11D" name="GS-11D"
            manufacturer="Geospace" type="Geophone"
            response="ResponsePAZ/GS-11D"/>
    <datalogger publicID="Datalogger/CS5532" name="CS5532"/>
    <network publicID="Network/XX" code="XX">
      <station publicID="Station/XX.PBUMI" code="PBUMI"
               latitude="-7.7714" longitude="110.3776" elevation="150.0">
        <sensorLocation publicID="SensorLocation/XX.PBUMI.00" code="00">
          <stream publicID="Stream/XX.PBUMI.00.SHZ" code="SHZ"
                  sensor="Sensor/GS-11D" datalogger="Datalogger/CS5532"
                  sampleRateNumerator="100" sampleRateDenominator="1"
                  azimuth="0.0" dip="-90.0"/>
        </sensorLocation>
      </station>
    </network>
    <responsePAZ publicID="ResponsePAZ/GS-11D" type="A" gain="32.0" gainFrequency="15.0"/>
  </Inventory>
</seiscomp>
```

Key differences dari FDSN:
- **Flat structure** — sensor, datalogger, response di top-level, direferensi via `publicID`
- **Attributes-heavy** — lat/lon/elevation sebagai attributes, bukan child elements
- `sensorLocation` = FDSN `location_code`, `stream` = FDSN `channel`
- Response disimpan terpisah (`responsePAZ`, `responseFIR`, `responsePolynomial`)

## Testing Strategy

1. **Unit tests**: Setiap struct serialize/deserialize correctly
2. **FDSN roundtrip**: Read FDSN XML → Inventory → write FDSN XML → read again → assert equal
3. **SC3ML roundtrip**: Same for SC3ML
4. **Cross-format**: Read FDSN → write SC3ML → read SC3ML → compare Inventory fields
5. **ObsPy compatibility**: Generate XML, verify parseable by `obspy.read_inventory()`
6. **Real fixtures**: Download StationXML dari IRIS/GFZ sebagai test data
7. **Sensor library**: All entries have sensitivity > 0, valid units

## Referensi

- [FDSN StationXML 1.2 Schema](https://www.fdsn.org/xml/station/fdsn-station-1.2.xsd)
- [FDSN StationXML Documentation](https://docs.fdsn.org/projects/stationxml/en/latest/)
- [SeisComP SC3ML Schema](https://www.seiscomp.de/doc/base/concepts/inventory.html)
- [ObsPy Inventory](https://docs.obspy.org/packages/autogen/obspy.core.inventory.inventory.Inventory.html)
- [IRIS NRL](https://ds.iris.edu/NRL/)
- [quick-xml serde guide](https://docs.rs/quick-xml/latest/quick_xml/de/index.html)

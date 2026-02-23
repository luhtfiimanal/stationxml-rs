# Channel Codes — The 3-Character Naming Convention

## Structure

A channel code is **3 characters** following the SEED naming convention (FDSN standard). Each character has a specific meaning:

```
  S  H  Z
  |  |  |
  |  |  +-- Orientation Code: measurement direction
  |  +----- Instrument Code: sensor type
  +-------- Band Code: sample rate & response band
```

## Character 1: Band Code

The band code indicates the **sample rate** and **frequency band** of the sensor:

| Code | Sample Rate | Period Range | Typical Use |
|------|------------|-------------|-------------|
| **F** | >= 1000 Hz | 0.1-5 s | Strong motion, engineering |
| **G** | >= 1000 Hz | >= 5 s | Uncommon |
| **D** | 250-999 Hz | < 0.1 s | High-frequency recording |
| **C** | 250-999 Hz | >= 5 s | Uncommon |
| **E** | 80-250 Hz | < 0.1 s | Short period, high sample |
| **S** | 10-80 Hz | < 0.1 s | **Short period** (most common for local monitoring) |
| **H** | 80-250 Hz | >= 5 s | **High broadband** (most common for regional/global) |
| **B** | 10-80 Hz | >= 5 s | **Broadband** |
| **M** | 1-10 Hz | >= 5 s | Mid-period |
| **L** | ~1 Hz | >= 5 s | Long period |
| **V** | ~0.1 Hz | >= 5 s | Very long period |
| **U** | ~0.01 Hz | >= 5 s | Ultra long period |

Examples:
- `SHZ` — **S** means sample rate 10-80 Hz, suitable for short period monitoring (local earthquakes)
- `BHZ` — **B** means broadband, sample rate 10-80 Hz, can record very low frequencies (teleseismic events)

## Character 2: Instrument Code

Indicates the **sensor type**:

| Code | Instrument | Description |
|------|-----------|-------------|
| **H** | High-gain seismometer | Standard velocity sensor (geophone, broadband) |
| **L** | Low-gain seismometer | Low-gain velocity sensor |
| **N** | Accelerometer | Measures acceleration (strong motion) |
| **G** | Gravimeter | Measures gravity |
| **M** | Mass position | Mass position in broadband sensor |
| **D** | Barometer/pressure | Atmospheric pressure |
| **F** | Magnetometer | Magnetic field |

Most common: **H** (seismometer/geophone) and **N** (accelerometer).

## Character 3: Orientation Code

Indicates the **measurement direction**:

| Code | Direction | Azimuth | Dip |
|------|----------|---------|-----|
| **Z** | Vertical (up) | 0 deg | **-90 deg** (pointing up) |
| **N** | North-South | **0 deg** (North) | 0 deg (horizontal) |
| **E** | East-West | **90 deg** (East) | 0 deg (horizontal) |
| **1** | Horizontal 1 | Varies | 0 deg |
| **2** | Horizontal 2 | Varies | 0 deg |

**Important note about dip convention:**
- `dip = -90` means sensor pointing **UP** (vertical Z component)
- `dip = 0` means sensor is **horizontal** (N, E components)
- `dip = 90` means sensor pointing **DOWN**

This often causes confusion because the FDSN convention defines negative dip as upward.

## Example: 3-Component Station

```
Station PBUMI (Yogyakarta)
|-- SHZ (00)  <-- Vertical, short period, 100 sps
|   azimuth=0, dip=-90
|-- SHN (00)  <-- North-South, short period, 100 sps
|   azimuth=0, dip=0
+-- SHE (00)  <-- East-West, short period, 100 sps
    azimuth=90, dip=0
```

This is called a **3-component (3C)** station. With 3 components, you can determine the **direction of incoming** earthquake waves.

## Location Code

In addition to the channel code, there is a **location code** (2 characters):

| Code | Meaning |
|------|---------|
| `""` (empty) | Default, only sensor at the location |
| `"00"` | Primary sensor |
| `"10"` | Secondary sensor (different sensor at same location) |
| `"20"`, `"30"`, ... | Additional sensors |

Example: a station can have both broadband and short period sensors:

```
Station PBUMI
|-- BHZ (00)  <-- Broadband STS-2, 20 sps
|-- BHN (00)
|-- BHE (00)
|-- SHZ (10)  <-- Short period GS-11D, 100 sps
|-- SHN (10)
+-- SHE (10)
```

The location code distinguishes data streams from different sensors at the same physical station.

## Mapping to Code

In `stationxml-rs`, the `Channel` struct captures these concepts directly:

```rust
pub struct Channel {
    pub code: String,           // "SHZ" -- 3-char channel code
    pub location_code: String,  // "00" -- 2-char location code
    pub azimuth: f64,           // 0.0 for Z/N, 90.0 for E
    pub dip: f64,               // -90.0 for Z, 0.0 for N/E
    pub sample_rate: f64,       // must match band code (S = 10-80 Hz)
    // ...
}
```

## Next

- [Instrument Response](03-instrument-response.md) — How to convert counts to physical units
- [FDSN vs SC3ML](04-fdsn-vs-sc3ml.md) — Comparing the two XML formats

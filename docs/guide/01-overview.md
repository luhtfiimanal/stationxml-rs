# Seismic Station Metadata — Overview

## What is a Seismic Station?

A seismic station is a physical installation that records ground vibrations 24/7. A typical station consists of:

```
+-------------------------------------+
|           Station "PBUMI"           |
|         (Yogyakarta, Indonesia)     |
|                                     |
|  +---------+     +--------------+   |
|  |  Sensor  |--->|  Datalogger  |---> Data (miniSEED)
|  | (GS-11D) |    |  (CS5532)    |      + Metadata (StationXML)
|  +---------+     +--------------+   |
|                                     |
|  Location: -7.7714, 110.3776       |
|  Elevation: 150m                    |
+-------------------------------------+
```

- **Sensor** (geophone/seismometer): converts ground motion into an electrical signal (Volts)
- **Datalogger** (ADC): converts Volts into digital numbers (counts)
- **Output**: waveform data in "counts" — raw numbers without physical units

## Why Metadata Matters

Data coming out of a datalogger is just **count values** — for example, `12345678`. Without metadata, we don't know:

1. **Where** is this sensor? (latitude, longitude, elevation)
2. **Which direction** is it measuring? (vertical? north-south? east-west?)
3. **How sensitive** is it? (1 count = how many m/s?)
4. **When** was it active? (start date, end date?)

Without this information, seismic data **cannot be used** for scientific analysis. Metadata is the "key" to converting raw counts into meaningful physical data.

## Metadata Hierarchy

Metadata is organized in a hierarchy:

```
Inventory (entire metadata collection)
 +-- Network (station network, 2-letter code)
      |   example: "GE" (GEOFON), "IU" (GSN), "XX" (temporary)
      |
      +-- Station (one physical location, 3-5 letter code)
           |   example: "PBUMI" -- has lat/lon/elevation
           |
           +-- Channel (one measurement component)
                |   example: "SHZ" -- 3 characters:
                |     S = band code (short period, ~10-80 Hz sample rate)
                |     H = instrument code (high-gain seismometer)
                |     Z = orientation code (vertical)
                |
                +-- Response (how to convert counts to physical units)
                     - Sensitivity: how many V/(m/s) does this sensor produce?
                     - Poles & Zeros: sensor transfer function
                     - Decimation: digital filters in the datalogger
```

## Who Uses Station Metadata?

| Who | What For |
|-----|----------|
| **Seismologists** | Earthquake location, waveform analysis — need sensor position & response |
| **Data centers** (IRIS, GFZ, BMKG) | Data distribution — must include metadata so data is usable |
| **Software** (ObsPy, SeisComP) | Remove instrument response — needs poles/zeros & sensitivity |
| **Station operators** | Documentation — what sensor, where, when installed |

## Data Flow

```
Sensor -> Datalogger -> miniSEED (waveform)
                     -> StationXML (metadata)  <-- THIS is what we build

User downloads both:
  miniSEED + StationXML -> ObsPy -> Remove response -> Data in m/s
```

## Metadata Formats

| Format | Created By | Used In |
|--------|-----------|---------|
| **FDSN StationXML** | FDSN (international standard) | IRIS, all FDSN data centers |
| **SC3ML** | GFZ (SeisComP) | SeisComP-based networks (including BMKG) |
| **Dataless SEED** | IRIS (legacy) | Old format, still widely found |
| **RESP** | IRIS (legacy) | Text format for response only |

This crate (`stationxml-rs`) focuses on reading/writing **FDSN StationXML** and **SC3ML**, and converting between them. This is important because many networks use SeisComP (SC3ML) operationally but international data centers require FDSN StationXML.

## Next

- [Channel Codes](02-channel-codes.md) — Understanding the 3-character naming convention
- [Instrument Response](03-instrument-response.md) — How to convert counts to physical units
- [FDSN vs SC3ML](04-fdsn-vs-sc3ml.md) — Comparing the two XML formats

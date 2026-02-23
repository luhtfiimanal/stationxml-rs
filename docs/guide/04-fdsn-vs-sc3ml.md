# FDSN StationXML vs SeisComP SC3ML

## Why Two Formats?

**FDSN StationXML** was created by FDSN (International Federation of Digital Seismograph Networks) as an international standard. All data centers in the FDSN federation (IRIS, GFZ, ORFEUS, etc.) must be able to serve data in this format.

**SC3ML** was created by GFZ (Helmholtz Centre Potsdam) as the internal format for **SeisComP** — open-source software for real-time seismic monitoring. SeisComP is widely used in Southeast Asia (including BMKG Indonesia), Latin America, Europe, and more.

The problem: many network operators use SeisComP for day-to-day operations (SC3ML), but must submit metadata to international data centers in FDSN StationXML format. **Manual conversion is error-prone** — which is why this crate exists.

## Fundamental Difference: Nested vs Flat

### FDSN StationXML = Nested (hierarchical)

All data is embedded within a parent-child hierarchy:

```xml
<FDSNStationXML>
  <Network code="XX">
    <Station code="PBUMI">
      <Channel code="SHZ" locationCode="00">
        <Sensor>                          <!-- sensor data INLINE -->
          <Type>Geophone</Type>
          <Model>GS-11D</Model>
        </Sensor>
        <Response>                        <!-- response data INLINE -->
          <InstrumentSensitivity>...</InstrumentSensitivity>
          <Stage number="1">
            <PolesZeros>...</PolesZeros>
          </Stage>
        </Response>
      </Channel>
    </Station>
  </Network>
</FDSNStationXML>
```

Each channel has its own **complete copy** of sensor and response data. If 10 channels use the same sensor, the sensor data is repeated 10 times.

### SC3ML = Flat (reference-based)

Sensors, dataloggers, and responses are stored **separately** at the top level, then referenced via `publicID`:

```xml
<seiscomp version="0.13">
  <Inventory>
    <!-- Top-level: sensor definition (shared) -->
    <sensor publicID="Sensor/GS-11D" name="GS-11D"
            manufacturer="Geospace" type="Geophone"
            response="ResponsePAZ/GS-11D"/>        <!-- reference to response -->

    <!-- Top-level: datalogger definition (shared) -->
    <datalogger publicID="Datalogger/CS5532" name="CS5532"/>

    <!-- Top-level: response definition (shared) -->
    <responsePAZ publicID="ResponsePAZ/GS-11D"
                 type="A" gain="32.0" gainFrequency="15.0"
                 zeros="(0,0)(0,0)"
                 poles="(-19.8,19.4)(-19.8,-19.4)"/>

    <!-- Hierarchy: network > station > sensorLocation > stream -->
    <network publicID="Network/XX" code="XX">
      <station publicID="Station/XX.PBUMI" code="PBUMI"
               latitude="-7.7714" longitude="110.3776" elevation="150.0">
        <sensorLocation publicID="SensorLocation/XX.PBUMI.00" code="00">
          <stream publicID="Stream/XX.PBUMI.00.SHZ" code="SHZ"
                  sensor="Sensor/GS-11D"              <!-- REFERENCE -->
                  datalogger="Datalogger/CS5532"       <!-- REFERENCE -->
                  azimuth="0.0" dip="-90.0"
                  sampleRateNumerator="100" sampleRateDenominator="1"/>
        </sensorLocation>
      </station>
    </network>
  </Inventory>
</seiscomp>
```

## Detailed Mapping

| Concept | FDSN StationXML | SC3ML |
|---------|----------------|-------|
| **Root element** | `<FDSNStationXML>` | `<seiscomp><Inventory>` |
| **Namespace** | `http://www.fdsn.org/xml/station/1` | `http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13` |
| **Network** | `<Network code="XX">` | `<network code="XX">` |
| **Station** | `<Station code="PBUMI">` | `<station code="PBUMI">` |
| **Location** | `locationCode` attr on Channel | `<sensorLocation code="00">` (separate element!) |
| **Channel** | `<Channel code="SHZ">` | `<stream code="SHZ">` |
| **Lat/Lon/Elev** | Child elements: `<Latitude>-7.7</Latitude>` | Attributes: `latitude="-7.7"` |
| **Sample rate** | `<SampleRate>100.0</SampleRate>` | `sampleRateNumerator="100" sampleRateDenominator="1"` |
| **Sensor info** | Inline `<Sensor>` in Channel | Top-level `<sensor>`, referenced by ID |
| **Datalogger** | Inline `<DataLogger>` in Channel | Top-level `<datalogger>`, referenced by ID |
| **Response** | Inline `<Response>` with `<Stage>` children | Top-level `<responsePAZ>`, `<responseFIR>`, etc. |
| **Dates** | `startDate="2026-01-01T00:00:00"` | `start="2026-01-01T00:00:00"` |

## Key Differences That Make Conversion Tricky

### 1. Location Code vs SensorLocation

In FDSN, location code is just an attribute on Channel:

```xml
<Channel code="SHZ" locationCode="00">
```

In SC3ML, location code becomes its own **wrapper element**:

```xml
<station>
  <sensorLocation code="00">    <!-- extra hierarchy level! -->
    <stream code="SHZ"/>
  </sensorLocation>
</station>
```

**FDSN to SC3ML:** Group channels by location_code, create one sensorLocation per group.

**SC3ML to FDSN:** Flatten — each stream in a sensorLocation becomes a Channel with a locationCode attribute.

### 2. Shared vs Duplicated Resources

In SC3ML, if 10 channels use the same sensor, the sensor is defined **once** and referenced 10 times. Efficient.

In FDSN, sensor data is **copied** into each channel. Redundant but self-contained.

**SC3ML to FDSN:** Resolve references — look up sensor/datalogger/response by publicID, copy data into each channel.

**FDSN to SC3ML:** Deduplicate — find identical sensor/response definitions, create shared entries, assign publicIDs.

### 3. Response Storage

**FDSN:** Response stages inline, with type information:

```xml
<Stage number="1">
  <PolesZeros>...</PolesZeros>
  <StageGain>...</StageGain>
</Stage>
```

**SC3ML:** Responses stored separately by type:

```xml
<responsePAZ publicID="ResponsePAZ/GS-11D" .../>    <!-- poles & zeros -->
<responseFIR publicID="ResponseFIR/filter1" .../>     <!-- FIR coefficients -->
<responsePolynomial .../>                              <!-- polynomial -->
```

The datalogger element has `<decimation>` children for the filter chain.

### 4. Sample Rate Representation

**FDSN:** Simple float: `<SampleRate>100.0</SampleRate>`

**SC3ML:** Ratio: `sampleRateNumerator="100" sampleRateDenominator="1"`

SC3ML supports non-integer sample rates more explicitly (e.g., 100/3 Hz).

**Conversion:** `sample_rate = numerator / denominator` and vice versa.

## Conversion Flow in stationxml-rs

```
         SC3ML                              FDSN
    +--------------+                  +--------------+
    | Flat XML:    |                  | Nested XML:  |
    | sensor, PAZ  |                  | Channel has  |
    | at top-level |                  | inline Sensor|
    | + references |                  | + Response   |
    +------+-------+                  +------+-------+
           |                                  |
           | sc3ml::reader                    | fdsn::reader
           | (resolve refs,                   | (straightforward
           |  flatten)                        |  nested -> struct)
           v                                  v
    +----------------------------------------------+
    |              Core Inventory                  |
    |  Network > Station > Channel > Response      |
    |  (format-agnostic, self-contained)           |
    +----------------------------------------------+
           |                                  |
           | sc3ml::writer                    | fdsn::writer
           | (deduplicate,                    | (straightforward
           |  create refs)                    |  struct -> nested)
           v                                  v
    +--------------+                  +--------------+
    |   SC3ML XML  |                  |   FDSN XML   |
    +--------------+                  +--------------+
```

The core `Inventory` model follows the **FDSN style** (nested, self-contained per channel) because:

1. More natural for programmatic manipulation
2. FDSN is the international standard
3. The SC3ML reader must resolve references anyway, producing nested data

The SC3ML writer must **deduplicate** — find identical sensor/response entries and create shared definitions. This is the most complex logic in the v0.2 implementation.

## PublicID Convention in SC3ML

SC3ML uses `publicID` strings as unique identifiers. Common conventions:

```
Sensor/{model}                    -> Sensor/GS-11D
Datalogger/{model}                -> Datalogger/CS5532
Network/{code}                    -> Network/XX
Station/{net}.{sta}               -> Station/XX.PBUMI
SensorLocation/{net}.{sta}.{loc}  -> SensorLocation/XX.PBUMI.00
Stream/{net}.{sta}.{loc}.{cha}    -> Stream/XX.PBUMI.00.SHZ
ResponsePAZ/{name}                -> ResponsePAZ/GS-11D
ResponseFIR/{name}                -> ResponseFIR/CS5532_100sps
```

The writer must generate reasonable publicIDs when converting from FDSN to SC3ML.

## Summary

| Aspect | FDSN StationXML | SC3ML |
|--------|----------------|-------|
| **Structure** | Nested hierarchy | Flat with references |
| **Data duplication** | High (per-channel copies) | Low (shared definitions) |
| **File size** | Larger | Smaller |
| **Self-contained** | Yes (each channel complete) | No (needs reference resolution) |
| **Human readability** | Easier (follow hierarchy) | Harder (must follow references) |
| **Standard** | International (FDSN) | Software-specific (SeisComP) |
| **Use case** | Data exchange, archival | Real-time operations |

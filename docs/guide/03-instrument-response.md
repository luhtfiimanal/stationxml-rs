# Instrument Response — From Counts to m/s

## The Core Problem

Seismic sensors record data in **counts** (integer values from the ADC). But scientists need data in physical units: **m/s** (velocity) or **m/s^2** (acceleration).

```
Ground motion (m/s) -> Sensor -> Voltage (V) -> ADC -> Counts (integer)

We want to go backwards:
Counts -> ??? -> m/s
```

The "???" is the **instrument response** — a transfer function that describes how the sensor + datalogger converts a physical signal into counts.

## Response Chain: Multi-Stage

The conversion from physical input to digital counts passes through multiple **stages**:

```
Stage 1: Sensor                    Stage 2: Digitizer (ADC)
+----------------------+          +----------------------+
|  Input: m/s          |          |  Input: V            |
|  Output: V           |          |  Output: COUNTS      |
|                      |          |                      |
|  Gain: 32.0 V/(m/s)  |  --->   |  Gain: 1678801.5     |
|  (sensor sensitivity) |          |  counts/V            |
|                      |          |                      |
|  Transfer Function:  |          |  Transfer Function:  |
|  Poles & Zeros       |          |  Coefficients (FIR)  |
+----------------------+          +----------------------+

Overall Sensitivity = Stage1.Gain x Stage2.Gain
                    = 32.0 x 1678801.5
                    = 53,721,648 counts/(m/s)
```

So:
- **1 m/s** ground motion produces **53.7 million counts** in the output
- **1 count** = 1/53721648 m/s ~ **1.86 x 10^-8 m/s** (extremely small!)

## Sensitivity — The Simplest Approach

For a quick conversion without worrying about frequency dependence:

```
velocity (m/s) = counts / overall_sensitivity
```

In StationXML, this is stored in `<InstrumentSensitivity>`:

```xml
<InstrumentSensitivity>
  <Value>53721548.8</Value>           <!-- counts per m/s -->
  <Frequency>15.0</Frequency>         <!-- valid at this frequency -->
  <InputUnits><Name>M/S</Name></InputUnits>
  <OutputUnits><Name>COUNTS</Name></OutputUnits>
</InstrumentSensitivity>
```

**Important:** Sensitivity is **only valid at one specific frequency** (in this example: 15 Hz). For accurate broadband analysis, you need the full transfer function.

## Poles & Zeros — The Full Transfer Function

A seismic sensor has a **frequency-dependent response** — it does not respond to all frequencies equally. Its transfer function is described using **Poles & Zeros** (P&Z):

```
            A0 x (s - z1)(s - z2)...(s - zn)
H(s) = -----------------------------------------
            (s - p1)(s - p2)...(s - pm)
```

Where:
- `s = j x 2*pi*f` (Laplace variable, `j` = imaginary unit)
- `z1, z2, ...` = **zeros** (frequencies where response = 0)
- `p1, p2, ...` = **poles** (frequencies where response approaches infinity)
- `A0` = normalization factor

### Example: GS-11D Geophone (velocity sensor)

```
Zeros: (0, 0), (0, 0)         <-- 2 zeros at origin
Poles: (-19.8, 19.4)           <-- complex conjugate pair
       (-19.8, -19.4)

Natural frequency: ~4.5 Hz
Damping: 0.6
```

What does this mean?
- **2 zeros at origin** = response rises with frequency (proportional to f^2) below the natural frequency
- **Poles** determine the sensor's natural frequency and damping
- **Below 4.5 Hz**, response drops drastically — a geophone cannot record distant earthquakes (low frequency)
- **Above 4.5 Hz**, response is flat — good for local earthquakes

```
Response Curve GS-11D (schematic):

Amplitude
    |          +---------------- flat region
    |         /
    |        /
    |       /
    |      /
    |     / <-- roll-off below natural freq
    |    /
    |   /
    +--/------------------------ Frequency
       1    4.5   10   50  100 Hz
           ^
       natural freq
```

Compare with **STS-2 Broadband**:
- Natural frequency: **0.00833 Hz** (120 second period!)
- Flat from 0.008 Hz to 50 Hz
- Can record earthquakes on the other side of the Earth

### In StationXML: Poles & Zeros

```xml
<Stage number="1">
  <PolesZeros>
    <InputUnits><Name>M/S</Name></InputUnits>
    <OutputUnits><Name>V</Name></OutputUnits>
    <PzTransferFunctionType>LAPLACE (RADIANS/SECOND)</PzTransferFunctionType>
    <NormalizationFactor>1.0</NormalizationFactor>
    <NormalizationFrequency>15.0</NormalizationFrequency>
    <Zero number="0">
      <Real>0.0</Real> <Imaginary>0.0</Imaginary>
    </Zero>
    <Zero number="1">
      <Real>0.0</Real> <Imaginary>0.0</Imaginary>
    </Zero>
    <Pole number="0">
      <Real>-19.8</Real> <Imaginary>19.4</Imaginary>
    </Pole>
    <Pole number="1">
      <Real>-19.8</Real> <Imaginary>-19.4</Imaginary>
    </Pole>
  </PolesZeros>
  <StageGain>
    <Value>32.0</Value>
    <Frequency>15.0</Frequency>
  </StageGain>
</Stage>
```

## ADC Stage — Coefficients & Decimation

Stage 2 (the digitizer) typically has:

```xml
<Stage number="2">
  <Coefficients>
    <InputUnits><Name>V</Name></InputUnits>
    <OutputUnits><Name>COUNTS</Name></OutputUnits>
    <CfTransferFunctionType>DIGITAL</CfTransferFunctionType>
  </Coefficients>
  <Decimation>
    <InputSampleRate>100.0</InputSampleRate>
    <Factor>1</Factor>
    <Offset>0</Offset>
    <Delay>0.0</Delay>
    <Correction>0.0</Correction>
  </Decimation>
  <StageGain>
    <Value>1678801.5</Value>
    <Frequency>15.0</Frequency>
  </StageGain>
</Stage>
```

### Understanding the ADC Gain

The ADC (Analog-to-Digital Converter) gain depends on:

```
ADC gain = max_count / full_scale_voltage
         = (2^(bits-1) - 1) / FSR
         = 8388607 / 5.0          (for 24-bit, 5V FSR)
         = 1677721.4 counts/V
```

With additional PGA (Programmable Gain Amplifier) and digital gain:

```
effective_gain = max_count * pga_gain * adc_gain / full_scale_voltage
```

## "Remove Instrument Response" — What ObsPy Does

When a scientist says "remove instrument response", what happens:

```
1. FFT the count data -> frequency domain
2. Divide by H(f) transfer function at each frequency
3. IFFT back -> time domain in m/s

Simplified:
  data_physical(f) = data_counts(f) / H(f)
```

This is why response metadata **must be accurate** — if the poles/zeros are wrong, the result is garbage.

## Overall Sensitivity Calculation

The overall sensitivity is the product of all stage gains:

```
overall_sensitivity = stage1_gain x stage2_gain x ... x stageN_gain
                    = sensor_sensitivity x ADC_gain
                    = 32.0 V/(m/s) x 1678801.5 counts/V
                    = 53,721,648 counts/(m/s)
```

In practice, with known ADC parameters:

```
overall_sensitivity = max_count x pga_gain x adc_gain x sensor_sensitivity / full_scale_voltage
                    = 8388607 x 1.0 x 1.0 x 32.0 / 5.0
                    = 53,687,084.8 counts/(m/s)
```

(Note: small differences due to rounding in max_count and ADC non-idealities.)

## Mapping to Code

In `stationxml-rs`, the response chain is represented as:

```rust
pub struct Response {
    pub instrument_sensitivity: Option<InstrumentSensitivity>,  // quick conversion
    pub stages: Vec<ResponseStage>,                              // full detail
}

pub struct ResponseStage {
    pub number: u32,                         // 1-based stage number
    pub stage_gain: Option<StageGain>,       // gain at reference frequency
    pub poles_zeros: Option<PolesZeros>,      // Stage 1 typically
    pub coefficients: Option<Coefficients>,   // Stage 2+ typically
    pub fir: Option<FIR>,                     // digital FIR filter
    pub decimation: Option<Decimation>,       // sample rate reduction
    pub input_units: Option<Units>,
    pub output_units: Option<Units>,
}
```

The `AdcConversion` helper simplifies the common case:

```rust
pub struct AdcConversion {
    pub full_scale_voltage: f64,  // e.g. 5.0V
    pub max_count: f64,           // 2^23 - 1 = 8388607 for 24-bit
    pub pga_gain: f64,            // pre-amplifier gain
    pub adc_gain: f64,            // digital gain
}

// Usage:
let adc = AdcConversion::new(5.0, 24, 1.0, 1.0);
let overall = adc.overall_sensitivity(32.0);  // 53,721,548.8
```

## Next

- [FDSN vs SC3ML](04-fdsn-vs-sc3ml.md) — Comparing the two XML formats

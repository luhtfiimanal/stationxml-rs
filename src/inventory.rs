//! Core inventory types — format-agnostic representation of seismic station metadata.
//!
//! These types represent the internal model used by all format backends (FDSN, SC3ML, etc.).
//! They follow FDSN naming conventions but are not tied to any specific XML structure.
//!
//! # Hierarchy
//!
//! ```text
//! Inventory
//!  └── Network
//!       └── Station
//!            └── Channel
//!                 └── Response
//!                      └── ResponseStage
//! ```

use chrono::{DateTime, Utc};

// ─── Top-level ───────────────────────────────────────────────────────

/// Top-level inventory — container for all station metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Inventory {
    /// Organization that generated this metadata (e.g. "IRIS", "Pena Bumi")
    pub source: String,
    /// Optional sender identifier
    pub sender: Option<String>,
    /// When this metadata document was created
    pub created: Option<DateTime<Utc>>,
    /// Networks contained in this inventory
    pub networks: Vec<Network>,
}

// ─── Network / Station ──────────────────────────────────────────────

/// A seismic network — a collection of stations operated together.
///
/// Network codes are typically 2 characters (e.g. "GE", "IU", "XX").
#[derive(Debug, Clone, PartialEq)]
pub struct Network {
    /// FDSN network code (e.g. "GE", "IU", "XX")
    pub code: String,
    /// Human-readable network description
    pub description: Option<String>,
    /// When this network epoch started
    pub start_date: Option<DateTime<Utc>>,
    /// When this network epoch ended (None = still active)
    pub end_date: Option<DateTime<Utc>>,
    /// Stations in this network
    pub stations: Vec<Station>,
}

/// A seismic station — one physical location with one or more sensors.
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    /// Station code (e.g. "PBUMI", "ANMO")
    pub code: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Geographic latitude in degrees (WGS84)
    pub latitude: f64,
    /// Geographic longitude in degrees (WGS84)
    pub longitude: f64,
    /// Elevation in meters above sea level
    pub elevation: f64,
    /// Site information (name, region, country, etc.)
    pub site: Site,
    /// When this station epoch started
    pub start_date: Option<DateTime<Utc>>,
    /// When this station epoch ended (None = still active)
    pub end_date: Option<DateTime<Utc>>,
    /// When this station was originally created
    pub creation_date: Option<DateTime<Utc>>,
    /// Channels (measurement components) at this station
    pub channels: Vec<Channel>,
}

/// Site information for a station — describes the physical location.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Site {
    /// Site name (e.g. "Yogyakarta Seismic Shelter")
    pub name: String,
    /// Optional detailed description
    pub description: Option<String>,
    /// Town or city
    pub town: Option<String>,
    /// County or district
    pub county: Option<String>,
    /// Region or state/province
    pub region: Option<String>,
    /// Country name
    pub country: Option<String>,
}

// ─── Channel ────────────────────────────────────────────────────────

/// A channel — one measurement component at a station.
///
/// Channel codes are 3 characters following the SEED naming convention:
/// - Band code (sample rate/response band): S, B, H, etc.
/// - Instrument code (sensor type): H (seismometer), N (accelerometer), etc.
/// - Orientation code (direction): Z (vertical), N (north), E (east), etc.
///
/// See `docs/guide/02-channel-codes.md` for the full breakdown.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    /// SEED channel code (e.g. "SHZ", "BHN", "HNE")
    pub code: String,
    /// Location code (e.g. "00", "10", "")
    pub location_code: String,
    /// Channel latitude in degrees (usually same as station)
    pub latitude: f64,
    /// Channel longitude in degrees (usually same as station)
    pub longitude: f64,
    /// Channel elevation in meters above sea level
    pub elevation: f64,
    /// Depth of sensor below surface in meters
    pub depth: f64,
    /// Azimuth in degrees from north (0=N, 90=E)
    pub azimuth: f64,
    /// Dip in degrees from horizontal (-90=up, 0=horizontal, 90=down)
    pub dip: f64,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// When this channel epoch started
    pub start_date: Option<DateTime<Utc>>,
    /// When this channel epoch ended (None = still active)
    pub end_date: Option<DateTime<Utc>>,
    /// Sensor (geophone, broadband, accelerometer, etc.)
    pub sensor: Option<Equipment>,
    /// Data logger / digitizer
    pub data_logger: Option<Equipment>,
    /// Instrument response (sensitivity, poles & zeros, etc.)
    pub response: Option<Response>,
}

/// Equipment description — sensor, datalogger, or other instrument.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Equipment {
    /// Equipment type (e.g. "Geophone", "Datalogger")
    pub equipment_type: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Manufacturer name (e.g. "Geospace", "Nanometrics")
    pub manufacturer: Option<String>,
    /// Vendor/distributor name
    pub vendor: Option<String>,
    /// Model name (e.g. "GS-11D", "Trillium 120")
    pub model: Option<String>,
    /// Serial number of this specific unit
    pub serial_number: Option<String>,
    /// When this equipment was installed
    pub installation_date: Option<DateTime<Utc>>,
    /// When this equipment was removed
    pub removal_date: Option<DateTime<Utc>>,
}

// ─── Response ───────────────────────────────────────────────────────

/// Full instrument response — describes how to convert counts to physical units.
///
/// Contains both a quick overall sensitivity and detailed per-stage information.
/// See `docs/guide/03-instrument-response.md` for background.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Response {
    /// Overall sensitivity (product of all stage gains).
    /// Used for quick counts-to-physical conversion at a single frequency.
    pub instrument_sensitivity: Option<InstrumentSensitivity>,
    /// Detailed per-stage response information.
    /// Stage 1 is typically the sensor, stage 2+ are digitizer/filters.
    pub stages: Vec<ResponseStage>,
}

/// Overall instrument sensitivity — a single-frequency approximation.
///
/// `value` is in units of `output_units / input_units` (e.g. counts per m/s).
/// Only valid at the specified `frequency`.
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentSensitivity {
    /// Sensitivity value (e.g. 53721548.8 counts/(m/s))
    pub value: f64,
    /// Frequency at which this sensitivity is valid (Hz)
    pub frequency: f64,
    /// Physical input units (e.g. M/S, M/S**2)
    pub input_units: Units,
    /// Digital output units (e.g. COUNTS)
    pub output_units: Units,
}

/// Physical or digital units.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Units {
    /// Unit name following SEED convention (e.g. "M/S", "V", "COUNTS")
    pub name: String,
    /// Optional human-readable description (e.g. "Velocity in meters per second")
    pub description: Option<String>,
}

// ─── Response stages ────────────────────────────────────────────────

/// One stage in the instrument response chain.
///
/// Each stage has a gain and optionally one transfer function type
/// (poles & zeros, coefficients, or FIR).
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseStage {
    /// Stage number (1-based). Stage 1 is typically the sensor.
    pub number: u32,
    /// Gain at a reference frequency for this stage
    pub stage_gain: Option<StageGain>,
    /// Poles & zeros transfer function (typically stage 1 — sensor)
    pub poles_zeros: Option<PolesZeros>,
    /// Coefficient transfer function
    pub coefficients: Option<Coefficients>,
    /// FIR filter
    pub fir: Option<FIR>,
    /// Decimation parameters (sample rate reduction)
    pub decimation: Option<Decimation>,
}

/// Gain of a single stage at a reference frequency.
#[derive(Debug, Clone, PartialEq)]
pub struct StageGain {
    /// Gain value (e.g. 32.0 V/(m/s) for a sensor, 1678801.5 counts/V for an ADC)
    pub value: f64,
    /// Frequency at which this gain is valid (Hz)
    pub frequency: f64,
}

// ─── Transfer functions ─────────────────────────────────────────────

/// Poles and zeros representation of a transfer function.
///
/// Describes the frequency response as:
/// ```text
/// H(s) = A0 * product(s - z_i) / product(s - p_j)
/// ```
/// where s = j*2*pi*f for Laplace (radians) or s = j*f for Laplace (Hz).
#[derive(Debug, Clone, PartialEq)]
pub struct PolesZeros {
    /// Input units for this stage (e.g. M/S for velocity)
    pub input_units: Units,
    /// Output units for this stage (e.g. V for voltage)
    pub output_units: Units,
    /// Transfer function type (Laplace in rad/s, Hz, or digital Z-transform)
    pub pz_transfer_function_type: PzTransferFunction,
    /// Normalization factor (A0) — scales the transfer function
    pub normalization_factor: f64,
    /// Frequency at which the normalization factor is computed (Hz)
    pub normalization_frequency: f64,
    /// Zeros of the transfer function (complex numbers)
    pub zeros: Vec<PoleZero>,
    /// Poles of the transfer function (complex numbers)
    pub poles: Vec<PoleZero>,
}

/// A single complex pole or zero.
#[derive(Debug, Clone, PartialEq)]
pub struct PoleZero {
    /// Stage-local index number
    pub number: u32,
    /// Real part of the complex value
    pub real: f64,
    /// Imaginary part of the complex value
    pub imaginary: f64,
}

/// Transfer function type for poles & zeros.
#[derive(Debug, Clone, PartialEq)]
pub enum PzTransferFunction {
    /// Laplace transform, angular frequency (radians/second)
    LaplaceRadians,
    /// Laplace transform, frequency in Hz
    LaplaceHertz,
    /// Digital (Z-transform)
    DigitalZTransform,
}

/// Coefficient-based transfer function.
#[derive(Debug, Clone, PartialEq)]
pub struct Coefficients {
    /// Input units for this stage
    pub input_units: Units,
    /// Output units for this stage
    pub output_units: Units,
    /// Transfer function type
    pub cf_transfer_function_type: CfTransferFunction,
    /// Numerator coefficients
    pub numerators: Vec<f64>,
    /// Denominator coefficients
    pub denominators: Vec<f64>,
}

/// Transfer function type for coefficients.
#[derive(Debug, Clone, PartialEq)]
pub enum CfTransferFunction {
    /// Analog, angular frequency (radians/second)
    AnalogRadians,
    /// Analog, frequency in Hz
    AnalogHertz,
    /// Digital
    Digital,
}

/// FIR (Finite Impulse Response) filter.
#[derive(Debug, Clone, PartialEq)]
pub struct FIR {
    /// Input units for this stage
    pub input_units: Units,
    /// Output units for this stage
    pub output_units: Units,
    /// Filter symmetry
    pub symmetry: Symmetry,
    /// Numerator coefficients
    pub numerator_coefficients: Vec<f64>,
}

/// FIR filter symmetry type.
#[derive(Debug, Clone, PartialEq)]
pub enum Symmetry {
    /// No symmetry — all coefficients specified
    None,
    /// Even symmetry — only first half specified
    Even,
    /// Odd symmetry — only first half specified
    Odd,
}

/// Decimation parameters — describes how sample rate is reduced at this stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Decimation {
    /// Input sample rate to this stage (Hz)
    pub input_sample_rate: f64,
    /// Decimation factor (output rate = input rate / factor)
    pub factor: u32,
    /// Sample offset for decimation
    pub offset: u32,
    /// Estimated delay introduced by this stage (seconds)
    pub delay: f64,
    /// Applied correction for the delay (seconds)
    pub correction: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inventory() {
        let inv = Inventory {
            source: "Test".into(),
            sender: None,
            created: None,
            networks: vec![],
        };
        assert_eq!(inv.source, "Test");
        assert!(inv.networks.is_empty());
    }

    #[test]
    fn full_inventory_construction() {
        let inv = Inventory {
            source: "Pena Bumi".into(),
            sender: Some("stationxml-rs".into()),
            created: None,
            networks: vec![Network {
                code: "XX".into(),
                description: Some("Local Test Network".into()),
                start_date: None,
                end_date: None,
                stations: vec![Station {
                    code: "PBUMI".into(),
                    description: None,
                    latitude: -7.7714,
                    longitude: 110.3776,
                    elevation: 150.0,
                    site: Site {
                        name: "Yogyakarta".into(),
                        ..Default::default()
                    },
                    start_date: None,
                    end_date: None,
                    creation_date: None,
                    channels: vec![Channel {
                        code: "SHZ".into(),
                        location_code: "00".into(),
                        latitude: -7.7714,
                        longitude: 110.3776,
                        elevation: 150.0,
                        depth: 0.0,
                        azimuth: 0.0,
                        dip: -90.0,
                        sample_rate: 100.0,
                        start_date: None,
                        end_date: None,
                        sensor: Some(Equipment {
                            equipment_type: Some("Geophone".into()),
                            model: Some("GS-11D".into()),
                            manufacturer: Some("Geospace".into()),
                            ..Default::default()
                        }),
                        data_logger: None,
                        response: Some(Response {
                            instrument_sensitivity: Some(InstrumentSensitivity {
                                value: 53721548.8,
                                frequency: 15.0,
                                input_units: Units {
                                    name: "M/S".into(),
                                    description: None,
                                },
                                output_units: Units {
                                    name: "COUNTS".into(),
                                    description: None,
                                },
                            }),
                            stages: vec![],
                        }),
                    }],
                }],
            }],
        };

        assert_eq!(inv.networks[0].code, "XX");
        let sta = &inv.networks[0].stations[0];
        assert_eq!(sta.code, "PBUMI");
        assert_eq!(sta.latitude, -7.7714);
        let ch = &sta.channels[0];
        assert_eq!(ch.code, "SHZ");
        assert_eq!(ch.dip, -90.0);
        let sens = ch
            .response
            .as_ref()
            .unwrap()
            .instrument_sensitivity
            .as_ref()
            .unwrap();
        assert!((sens.value - 53721548.8).abs() < 0.1);
    }

    #[test]
    fn site_default() {
        let site = Site::default();
        assert!(site.name.is_empty());
        assert!(site.country.is_none());
    }

    #[test]
    fn equipment_default() {
        let eq = Equipment::default();
        assert!(eq.model.is_none());
        assert!(eq.manufacturer.is_none());
    }

    #[test]
    fn response_default() {
        let resp = Response::default();
        assert!(resp.instrument_sensitivity.is_none());
        assert!(resp.stages.is_empty());
    }
}

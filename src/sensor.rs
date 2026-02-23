//! Built-in sensor library.
//!
//! Provides a database of common seismometer and accelerometer
//! specifications, loaded from an embedded JSON file.

use serde::Deserialize;
use std::sync::OnceLock;

const SENSORS_JSON: &str = include_str!("../data/sensors.json");

static SENSOR_DB: OnceLock<Vec<SensorEntry>> = OnceLock::new();

/// A sensor specification from the built-in database.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SensorEntry {
    /// Model name (e.g. "GS-11D", "STS-2")
    pub model: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Sensor type (e.g. "Geophone", "Broadband")
    pub sensor_type: String,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
    /// Sensitivity in V per (m/s) or V per (m/s^2)
    pub sensitivity: f64,
    /// Sensitivity unit: "M/S" or "M/S**2"
    pub sensitivity_unit: String,
    /// Operating frequency range as (low_hz, high_hz)
    pub frequency_range: (f64, f64),
    /// Natural period in seconds (for geophones)
    pub natural_period: Option<f64>,
    /// Damping ratio (fraction of critical damping)
    pub damping: Option<f64>,
}

/// Load the built-in sensor library.
///
/// Returns a slice of all sensor entries. The library is lazily initialized
/// and cached for the lifetime of the program.
pub fn load_sensor_library() -> &'static [SensorEntry] {
    SENSOR_DB
        .get_or_init(|| serde_json::from_str(SENSORS_JSON).expect("embedded sensors.json is valid"))
}

/// Find a sensor by model name (case-insensitive).
///
/// Returns `None` if no matching sensor is found.
pub fn find_sensor(model: &str) -> Option<&'static SensorEntry> {
    let model_lower = model.to_lowercase();
    load_sensor_library()
        .iter()
        .find(|s| s.model.to_lowercase() == model_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_library_count() {
        let sensors = load_sensor_library();
        assert_eq!(sensors.len(), 9);
    }

    #[test]
    fn find_gs11d() {
        let sensor = find_sensor("GS-11D").unwrap();
        assert_eq!(sensor.manufacturer, "Geospace");
        assert_eq!(sensor.sensitivity, 32.0);
        assert_eq!(sensor.sensitivity_unit, "M/S");
    }

    #[test]
    fn find_case_insensitive() {
        assert!(find_sensor("gs-11d").is_some());
        assert!(find_sensor("sts-2").is_some());
        assert!(find_sensor("STS-2").is_some());
    }

    #[test]
    fn find_nonexistent() {
        assert!(find_sensor("NonExistentSensor").is_none());
    }

    #[test]
    fn all_entries_valid() {
        for sensor in load_sensor_library() {
            assert!(sensor.sensitivity > 0.0, "sensitivity must be positive");
            assert!(!sensor.model.is_empty(), "model must not be empty");
            assert!(!sensor.manufacturer.is_empty());
            assert!(
                sensor.frequency_range.0 < sensor.frequency_range.1,
                "freq range must be low < high"
            );
        }
    }

    #[test]
    fn broadband_vs_geophone() {
        let sts2 = find_sensor("STS-2").unwrap();
        let gs11d = find_sensor("GS-11D").unwrap();

        // Broadband has much higher sensitivity
        assert!(sts2.sensitivity > gs11d.sensitivity);
        // Broadband has much wider frequency range (lower low-freq)
        assert!(sts2.frequency_range.0 < gs11d.frequency_range.0);
    }
}

//! Pure Rust FDSN StationXML and SeisComP SC3ML reader/writer.
//!
//! `stationxml-rs` provides a format-agnostic inventory model with pluggable
//! backends for different XML formats used in seismology.
//!
//! # Supported Formats
//!
//! | Format | Read | Write | Status |
//! |--------|------|-------|--------|
//! | FDSN StationXML 1.2 | Yes | Yes | v0.1 |
//! | SeisComP SC3ML 0.13 | -- | -- | v0.2 (planned) |

pub mod builder;
pub mod conversion;
pub mod error;
pub mod fdsn;
pub mod format;
pub mod inventory;
pub mod sensor;

pub use builder::InventoryBuilder;
pub use conversion::AdcConversion;
pub use error::{Result, StationXmlError};
pub use fdsn::Fdsn;
pub use format::{Format, StationXmlFormat, detect_format};
pub use inventory::*;
pub use sensor::{SensorEntry, find_sensor, load_sensor_library};

use std::path::Path;

/// Read from file with auto-format detection.
pub fn read_from_file(path: impl AsRef<Path>) -> Result<Inventory> {
    let content = std::fs::read_to_string(path)?;
    read_from_str(&content)
}

/// Read from string with auto-format detection.
pub fn read_from_str(xml: &str) -> Result<Inventory> {
    match detect_format(xml) {
        Some(Format::Fdsn) => Fdsn::read_from_str(xml),
        Some(Format::Sc3ml) => Err(StationXmlError::InvalidData(
            "SC3ML format not yet supported in v0.1".into(),
        )),
        None => Err(StationXmlError::UnknownFormat),
    }
}

/// Read from file with explicit format.
pub fn read_from_file_as<F: StationXmlFormat>(path: impl AsRef<Path>) -> Result<Inventory> {
    let content = std::fs::read_to_string(path)?;
    F::read_from_str(&content)
}

/// Write to file with explicit format.
pub fn write_to_file<F: StationXmlFormat>(
    path: impl AsRef<Path>,
    inventory: &Inventory,
) -> Result<()> {
    let xml = F::write_to_string(inventory)?;
    std::fs::write(path, xml)?;
    Ok(())
}

/// Write to string with explicit format.
pub fn write_to_string<F: StationXmlFormat>(inventory: &Inventory) -> Result<String> {
    F::write_to_string(inventory)
}

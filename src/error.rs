//! Error types for stationxml-rs.

use thiserror::Error;

/// All errors that can occur when reading/writing station metadata.
#[derive(Debug, Error)]
pub enum StationXmlError {
    /// Failed to deserialize XML (invalid structure, missing elements, etc.)
    #[error("XML parsing error: {0}")]
    XmlParse(#[from] quick_xml::DeError),

    /// Failed to serialize to XML
    #[error("XML serialization error: {0}")]
    XmlSerialize(#[from] quick_xml::SeError),

    /// Failed to parse JSON (sensor library)
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// File I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Could not detect XML format from root element
    #[error("unknown format: could not detect XML format from root element")]
    UnknownFormat,

    /// Data is present but invalid (bad datetime, out-of-range value, etc.)
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// A required field is missing from the input
    #[error("missing required field: {0}")]
    MissingField(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, StationXmlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_unknown_format() {
        let err = StationXmlError::UnknownFormat;
        assert!(err.to_string().contains("unknown format"));
    }

    #[test]
    fn error_display_invalid_data() {
        let err = StationXmlError::InvalidData("bad datetime value".into());
        assert!(err.to_string().contains("bad datetime value"));
    }

    #[test]
    fn error_display_missing_field() {
        let err = StationXmlError::MissingField("Latitude".into());
        assert!(err.to_string().contains("Latitude"));
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: StationXmlError = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }
}

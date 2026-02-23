//! Format trait and auto-detection.
//!
//! The [`StationXmlFormat`] trait is implemented by each format backend
//! (FDSN, SC3ML). [`detect_format`] inspects the root XML element to
//! determine which format a document uses.

use crate::error::Result;
use crate::inventory::Inventory;

/// Supported XML formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// FDSN StationXML 1.2
    Fdsn,
    /// SeisComP SC3ML 0.13
    Sc3ml,
}

/// Trait implemented by each format backend.
///
/// Each format (FDSN, SC3ML, etc.) provides read/write via this trait.
/// The type parameter is a zero-sized marker struct (e.g. `Fdsn`, `Sc3ml`).
pub trait StationXmlFormat {
    /// Deserialize XML string into an [`Inventory`].
    fn read_from_str(xml: &str) -> Result<Inventory>;

    /// Deserialize XML bytes into an [`Inventory`].
    fn read_from_bytes(bytes: &[u8]) -> Result<Inventory>;

    /// Serialize an [`Inventory`] to an XML string.
    fn write_to_string(inventory: &Inventory) -> Result<String>;
}

/// Detect the XML format by inspecting the root element name.
///
/// Uses quick-xml's event reader to skip over XML declarations, comments,
/// and whitespace, then matches on the first start element:
/// - `<FDSNStationXML ...>` → [`Format::Fdsn`]
/// - `<seiscomp ...>` → [`Format::Sc3ml`]
///
/// Returns `None` if the root element is not recognized.
pub fn detect_format(xml: &str) -> Option<Format> {
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e)) => {
                return match e.local_name().as_ref() {
                    b"FDSNStationXML" => Some(Format::Fdsn),
                    b"seiscomp" => Some(Format::Sc3ml),
                    _ => None,
                };
            }
            Ok(quick_xml::events::Event::Eof) => return None,
            Err(_) => return None,
            _ => continue, // skip Declaration, Comment, PI, etc.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_fdsn_with_declaration() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" schemaVersion="1.2">
  <Source>Test</Source>
</FDSNStationXML>"#;
        assert_eq!(detect_format(xml), Some(Format::Fdsn));
    }

    #[test]
    fn detect_fdsn_without_declaration() {
        let xml = r#"<FDSNStationXML schemaVersion="1.2"><Source>T</Source></FDSNStationXML>"#;
        assert_eq!(detect_format(xml), Some(Format::Fdsn));
    }

    #[test]
    fn detect_sc3ml() {
        let xml = r#"<?xml version="1.0"?>
<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13" version="0.13">
  <Inventory></Inventory>
</seiscomp>"#;
        assert_eq!(detect_format(xml), Some(Format::Sc3ml));
    }

    #[test]
    fn detect_with_comments() {
        let xml = r#"<?xml version="1.0"?>
<!-- This is a comment -->
<FDSNStationXML schemaVersion="1.2"><Source>T</Source></FDSNStationXML>"#;
        assert_eq!(detect_format(xml), Some(Format::Fdsn));
    }

    #[test]
    fn detect_unknown() {
        let xml = r#"<html><body>not station metadata</body></html>"#;
        assert_eq!(detect_format(xml), None);
    }

    #[test]
    fn detect_empty() {
        assert_eq!(detect_format(""), None);
    }

    #[test]
    fn detect_invalid_xml() {
        assert_eq!(detect_format("not xml at all"), None);
    }

    #[test]
    fn format_enum_copy() {
        let f = Format::Fdsn;
        let f2 = f; // Copy
        assert_eq!(f, f2);
    }
}

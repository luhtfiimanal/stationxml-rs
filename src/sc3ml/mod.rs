//! SeisComP SC3ML 0.13 format backend.
//!
//! Implements [`StationXmlFormat`] for reading and writing
//! SeisComP SC3ML documents (versions 0.6–0.13).

pub(crate) mod reader;
pub(crate) mod types;
pub(crate) mod writer;

use crate::error::Result;
use crate::format::StationXmlFormat;
use crate::inventory::Inventory;

/// SeisComP SC3ML 0.13 format marker.
///
/// Use this with [`StationXmlFormat`] methods to read/write SC3ML.
///
/// ```no_run
/// use stationxml_rs::{Sc3ml, StationXmlFormat};
///
/// let inv = Sc3ml::read_from_str("<seiscomp ...>...</seiscomp>").unwrap();
/// let xml = Sc3ml::write_to_string(&inv).unwrap();
/// ```
pub struct Sc3ml;

impl StationXmlFormat for Sc3ml {
    fn read_from_str(xml: &str) -> Result<Inventory> {
        reader::read_from_str(xml)
    }

    fn read_from_bytes(bytes: &[u8]) -> Result<Inventory> {
        reader::read_from_bytes(bytes)
    }

    fn write_to_string(inventory: &Inventory) -> Result<String> {
        writer::write_to_string(inventory)
    }
}

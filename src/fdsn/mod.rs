//! FDSN StationXML 1.2 format backend.
//!
//! Implements [`StationXmlFormat`] for reading and writing
//! FDSN StationXML 1.2 documents.

pub(crate) mod reader;
pub(crate) mod types;
pub(crate) mod writer;

use crate::error::Result;
use crate::format::StationXmlFormat;
use crate::inventory::Inventory;

/// FDSN StationXML 1.2 format marker.
///
/// Use this with [`StationXmlFormat`] methods to read/write FDSN StationXML.
///
/// ```no_run
/// use stationxml_rs::{Fdsn, StationXmlFormat};
///
/// let inv = Fdsn::read_from_str("<FDSNStationXML ...>...</FDSNStationXML>").unwrap();
/// let xml = Fdsn::write_to_string(&inv).unwrap();
/// ```
pub struct Fdsn;

impl StationXmlFormat for Fdsn {
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

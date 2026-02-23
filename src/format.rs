//! Format trait and auto-detection.
//!
//! The [`StationXmlFormat`] trait is implemented by each format backend
//! (FDSN, SC3ML). [`detect_format`] inspects the root XML element to
//! determine which format a document uses.

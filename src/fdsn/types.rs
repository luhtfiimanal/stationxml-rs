//! FDSN-specific XML serde structs (internal).
//!
//! These types map directly to the FDSN StationXML 1.2 schema
//! and are used only for serialization/deserialization.
//! They are NOT part of the public API.
//!
//! Field ordering matches the FDSN schema (xs:sequence) because
//! quick-xml serializes in struct field order.

use serde::{Deserialize, Serialize};

// ─── Root ────────────────────────────────────────────────────────────

/// Root element: `<FDSNStationXML>`
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "FDSNStationXML")]
pub(crate) struct FdsnStationXml {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "@schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "Source")]
    pub source: String,
    #[serde(rename = "Sender", default, skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
    #[serde(rename = "Module", default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(rename = "ModuleURI", default, skip_serializing_if = "Option::is_none")]
    pub module_uri: Option<String>,
    #[serde(rename = "Created")]
    pub created: String,
    #[serde(rename = "Network", default)]
    pub networks: Vec<FdsnNetwork>,
}

// ─── Network / Station ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Network")]
pub(crate) struct FdsnNetwork {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(
        rename = "@startDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_date: Option<String>,
    #[serde(rename = "@endDate", default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(
        rename = "@restrictedStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub restricted_status: Option<String>,
    // Child elements (order matches FDSN schema)
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(
        rename = "TotalNumberStations",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_number_stations: Option<u32>,
    #[serde(
        rename = "SelectedNumberStations",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub selected_number_stations: Option<u32>,
    #[serde(rename = "Station", default)]
    pub stations: Vec<FdsnStation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Station")]
pub(crate) struct FdsnStation {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(
        rename = "@startDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_date: Option<String>,
    #[serde(rename = "@endDate", default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(
        rename = "@restrictedStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub restricted_status: Option<String>,
    // Child elements (order matches FDSN schema)
    #[serde(rename = "Latitude")]
    pub latitude: FdsnFloatValue,
    #[serde(rename = "Longitude")]
    pub longitude: FdsnFloatValue,
    #[serde(rename = "Elevation")]
    pub elevation: FdsnFloatValue,
    #[serde(rename = "Site")]
    pub site: FdsnSite,
    #[serde(
        rename = "CreationDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub creation_date: Option<String>,
    #[serde(
        rename = "TotalNumberChannels",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_number_channels: Option<u32>,
    #[serde(
        rename = "SelectedNumberChannels",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub selected_number_channels: Option<u32>,
    #[serde(rename = "Channel", default)]
    pub channels: Vec<FdsnChannel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnSite {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "Town", default, skip_serializing_if = "Option::is_none")]
    pub town: Option<String>,
    #[serde(rename = "County", default, skip_serializing_if = "Option::is_none")]
    pub county: Option<String>,
    #[serde(rename = "Region", default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(rename = "Country", default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

// ─── Channel ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "Channel")]
pub(crate) struct FdsnChannel {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "@locationCode")]
    pub location_code: String,
    #[serde(
        rename = "@startDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_date: Option<String>,
    #[serde(rename = "@endDate", default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(
        rename = "@restrictedStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub restricted_status: Option<String>,
    // Child elements (order matches FDSN schema)
    #[serde(rename = "Latitude")]
    pub latitude: FdsnFloatValue,
    #[serde(rename = "Longitude")]
    pub longitude: FdsnFloatValue,
    #[serde(rename = "Elevation")]
    pub elevation: FdsnFloatValue,
    #[serde(rename = "Depth")]
    pub depth: FdsnFloatValue,
    #[serde(rename = "Azimuth")]
    pub azimuth: FdsnFloatValue,
    #[serde(rename = "Dip")]
    pub dip: FdsnFloatValue,
    #[serde(rename = "Type", default, skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<String>,
    #[serde(rename = "SampleRate")]
    pub sample_rate: FdsnFloatValue,
    #[serde(rename = "Sensor", default, skip_serializing_if = "Option::is_none")]
    pub sensor: Option<FdsnEquipment>,
    #[serde(
        rename = "DataLogger",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_logger: Option<FdsnEquipment>,
    #[serde(rename = "Response", default, skip_serializing_if = "Option::is_none")]
    pub response: Option<FdsnResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnEquipment {
    #[serde(rename = "Type", default, skip_serializing_if = "Option::is_none")]
    pub equipment_type: Option<String>,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(
        rename = "Manufacturer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer: Option<String>,
    #[serde(rename = "Vendor", default, skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(rename = "Model", default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(
        rename = "SerialNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub serial_number: Option<String>,
    #[serde(
        rename = "InstallationDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub installation_date: Option<String>,
    #[serde(
        rename = "RemovalDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub removal_date: Option<String>,
}

// ─── Float value helper ─────────────────────────────────────────────

/// Helper for FDSN FloatType elements that may have optional attributes.
///
/// Handles both `<Latitude>-7.7714</Latitude>` and
/// `<Latitude datum="WGS84" plusError="0.001">-7.7714</Latitude>`.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnFloatValue {
    #[serde(rename = "$text")]
    pub value: f64,
    #[serde(rename = "@unit", default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "@datum", default, skip_serializing_if = "Option::is_none")]
    pub datum: Option<String>,
    #[serde(
        rename = "@plusError",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub plus_error: Option<f64>,
    #[serde(
        rename = "@minusError",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minus_error: Option<f64>,
}

impl FdsnFloatValue {
    /// Create a simple float value without any attributes.
    pub fn new(value: f64) -> Self {
        Self {
            value,
            unit: None,
            datum: None,
            plus_error: None,
            minus_error: None,
        }
    }
}

// ─── Response ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnResponse {
    #[serde(
        rename = "InstrumentSensitivity",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub instrument_sensitivity: Option<FdsnInstrumentSensitivity>,
    #[serde(rename = "Stage", default)]
    pub stages: Vec<FdsnResponseStage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnInstrumentSensitivity {
    #[serde(rename = "Value")]
    pub value: f64,
    #[serde(rename = "Frequency")]
    pub frequency: f64,
    #[serde(rename = "InputUnits")]
    pub input_units: FdsnUnits,
    #[serde(rename = "OutputUnits")]
    pub output_units: FdsnUnits,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnUnits {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

// ─── Response stages ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnResponseStage {
    #[serde(rename = "@number")]
    pub number: u32,
    #[serde(
        rename = "PolesZeros",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub poles_zeros: Option<FdsnPolesZeros>,
    #[serde(
        rename = "Coefficients",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub coefficients: Option<FdsnCoefficients>,
    #[serde(rename = "FIR", default, skip_serializing_if = "Option::is_none")]
    pub fir: Option<FdsnFIR>,
    #[serde(
        rename = "Decimation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub decimation: Option<FdsnDecimation>,
    #[serde(rename = "StageGain", default, skip_serializing_if = "Option::is_none")]
    pub stage_gain: Option<FdsnStageGain>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnStageGain {
    #[serde(rename = "Value")]
    pub value: f64,
    #[serde(rename = "Frequency")]
    pub frequency: f64,
}

// ─── Poles & Zeros ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnPolesZeros {
    #[serde(rename = "InputUnits")]
    pub input_units: FdsnUnits,
    #[serde(rename = "OutputUnits")]
    pub output_units: FdsnUnits,
    #[serde(rename = "PzTransferFunctionType")]
    pub pz_transfer_function_type: String,
    #[serde(rename = "NormalizationFactor")]
    pub normalization_factor: f64,
    #[serde(rename = "NormalizationFrequency")]
    pub normalization_frequency: FdsnFloatValue,
    #[serde(rename = "Zero", default)]
    pub zeros: Vec<FdsnPoleZero>,
    #[serde(rename = "Pole", default)]
    pub poles: Vec<FdsnPoleZero>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnPoleZero {
    #[serde(rename = "@number")]
    pub number: u32,
    #[serde(rename = "Real")]
    pub real: FdsnFloatValue,
    #[serde(rename = "Imaginary")]
    pub imaginary: FdsnFloatValue,
}

// ─── Coefficients ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnCoefficients {
    #[serde(rename = "InputUnits")]
    pub input_units: FdsnUnits,
    #[serde(rename = "OutputUnits")]
    pub output_units: FdsnUnits,
    #[serde(rename = "CfTransferFunctionType")]
    pub cf_transfer_function_type: String,
    #[serde(rename = "Numerator", default)]
    pub numerators: Vec<FdsnFloatValue>,
    #[serde(rename = "Denominator", default)]
    pub denominators: Vec<FdsnFloatValue>,
}

// ─── FIR ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnFIR {
    #[serde(rename = "InputUnits")]
    pub input_units: FdsnUnits,
    #[serde(rename = "OutputUnits")]
    pub output_units: FdsnUnits,
    #[serde(rename = "Symmetry")]
    pub symmetry: String,
    #[serde(rename = "NumeratorCoefficient", default)]
    pub numerator_coefficients: Vec<FdsnFloatValue>,
}

// ─── Decimation ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FdsnDecimation {
    #[serde(rename = "InputSampleRate")]
    pub input_sample_rate: FdsnFloatValue,
    #[serde(rename = "Factor")]
    pub factor: u32,
    #[serde(rename = "Offset")]
    pub offset: u32,
    #[serde(rename = "Delay")]
    pub delay: FdsnFloatValue,
    #[serde(rename = "Correction")]
    pub correction: FdsnFloatValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_root() {
        let xml = r#"<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" schemaVersion="1.2">
  <Source>Test</Source>
  <Created>2026-01-01T00:00:00Z</Created>
</FDSNStationXML>"#;
        let doc: FdsnStationXml = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(doc.source, "Test");
        assert_eq!(doc.schema_version, "1.2");
        assert_eq!(
            doc.xmlns.as_deref(),
            Some("http://www.fdsn.org/xml/station/1")
        );
        assert!(doc.networks.is_empty());
    }

    #[test]
    fn deserialize_float_value_plain() {
        let xml = r#"<Latitude>-7.7714</Latitude>"#;
        let fv: FdsnFloatValue = quick_xml::de::from_str(xml).unwrap();
        assert!((fv.value - (-7.7714)).abs() < 1e-6);
        assert!(fv.datum.is_none());
    }

    #[test]
    fn deserialize_float_value_with_attrs() {
        let xml = r#"<Latitude datum="WGS84" plusError="0.001">-7.7714</Latitude>"#;
        let fv: FdsnFloatValue = quick_xml::de::from_str(xml).unwrap();
        assert!((fv.value - (-7.7714)).abs() < 1e-6);
        assert_eq!(fv.datum.as_deref(), Some("WGS84"));
        assert!((fv.plus_error.unwrap() - 0.001).abs() < 1e-6);
    }

    #[test]
    fn serialize_float_value_plain() {
        let fv = FdsnFloatValue::new(-7.7714);
        let xml = quick_xml::se::to_string(&fv).unwrap();
        assert!(xml.contains("-7.7714"));
        // Should NOT contain datum or other attrs
        assert!(!xml.contains("datum"));
    }

    #[test]
    fn deserialize_network_with_station() {
        let xml = r#"<FDSNStationXML schemaVersion="1.2">
  <Source>Test</Source>
  <Created>2026-01-01T00:00:00Z</Created>
  <Network code="XX">
    <Description>Test Network</Description>
    <Station code="PBUMI">
      <Latitude>-7.7714</Latitude>
      <Longitude>110.3776</Longitude>
      <Elevation>150.0</Elevation>
      <Site><Name>Yogyakarta</Name></Site>
    </Station>
  </Network>
</FDSNStationXML>"#;
        let doc: FdsnStationXml = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(doc.networks.len(), 1);
        assert_eq!(doc.networks[0].code, "XX");
        assert_eq!(doc.networks[0].stations.len(), 1);
        assert_eq!(doc.networks[0].stations[0].code, "PBUMI");
        assert!((doc.networks[0].stations[0].latitude.value - (-7.7714)).abs() < 1e-6);
    }

    #[test]
    fn deserialize_channel_with_response() {
        let xml = r#"<Channel code="SHZ" locationCode="00">
  <Latitude>-7.7714</Latitude>
  <Longitude>110.3776</Longitude>
  <Elevation>150.0</Elevation>
  <Depth>0.0</Depth>
  <Azimuth>0.0</Azimuth>
  <Dip>-90.0</Dip>
  <SampleRate>100.0</SampleRate>
  <Sensor>
    <Type>Geophone</Type>
    <Model>GS-11D</Model>
    <Manufacturer>Geospace</Manufacturer>
  </Sensor>
  <Response>
    <InstrumentSensitivity>
      <Value>53721548.8</Value>
      <Frequency>15.0</Frequency>
      <InputUnits><Name>M/S</Name></InputUnits>
      <OutputUnits><Name>COUNTS</Name></OutputUnits>
    </InstrumentSensitivity>
  </Response>
</Channel>"#;
        let ch: FdsnChannel = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(ch.code, "SHZ");
        assert_eq!(ch.location_code, "00");
        assert!((ch.dip.value - (-90.0)).abs() < 1e-6);
        assert!((ch.sample_rate.value - 100.0).abs() < 1e-6);

        let sensor = ch.sensor.as_ref().unwrap();
        assert_eq!(sensor.model.as_deref(), Some("GS-11D"));
        assert_eq!(sensor.equipment_type.as_deref(), Some("Geophone"));

        let resp = ch.response.as_ref().unwrap();
        let sens = resp.instrument_sensitivity.as_ref().unwrap();
        assert!((sens.value - 53721548.8).abs() < 0.1);
        assert_eq!(sens.input_units.name, "M/S");
    }
}

//! SC3ML-specific XML serde structs (internal).
//!
//! These types map directly to the SeisComP SC3ML schema (versions 0.6–0.13)
//! and are used only for serialization/deserialization.
//! They are NOT part of the public API.
//!
//! SC3ML uses lowercase element names (unlike FDSN's PascalCase).
//! Top-level definitions (sensor, datalogger, responsePAZ, responseFIR)
//! are referenced by `publicID` from stream elements.

use serde::{Deserialize, Serialize};

// ─── Root ────────────────────────────────────────────────────────────

/// Root element: `<seiscomp xmlns="..." version="0.13">`
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "seiscomp")]
pub(crate) struct Sc3mlRoot {
    #[serde(rename = "@xmlns", default, skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "Inventory")]
    pub inventory: Sc3mlInventory,
}

/// `<Inventory>` wrapper — contains all definitions and the network hierarchy.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlInventory {
    #[serde(rename = "sensor", default)]
    pub sensors: Vec<Sc3mlSensor>,
    #[serde(rename = "datalogger", default)]
    pub dataloggers: Vec<Sc3mlDatalogger>,
    #[serde(rename = "responsePAZ", default)]
    pub response_paz: Vec<Sc3mlResponsePaz>,
    #[serde(rename = "responseFIR", default)]
    pub response_fir: Vec<Sc3mlResponseFir>,
    #[serde(rename = "network", default)]
    pub networks: Vec<Sc3mlNetwork>,
}

// ─── Top-level definitions ───────────────────────────────────────────

/// `<sensor publicID="..." name="..." response="...">`
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlSensor {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@response", default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "model", default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(
        rename = "manufacturer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub sensor_type: Option<String>,
    #[serde(rename = "unit", default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "remark", default, skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

/// `<datalogger publicID="..." name="...">`
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlDatalogger {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "gain", default, skip_serializing_if = "Option::is_none")]
    pub gain: Option<f64>,
    #[serde(
        rename = "maxClockDrift",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_clock_drift: Option<f64>,
    #[serde(rename = "decimation", default)]
    pub decimations: Vec<Sc3mlDecimation>,
    #[serde(rename = "remark", default, skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

/// `<decimation sampleRateNumerator="40" sampleRateDenominator="1">`
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlDecimation {
    #[serde(rename = "@sampleRateNumerator")]
    pub sample_rate_numerator: u32,
    #[serde(rename = "@sampleRateDenominator")]
    pub sample_rate_denominator: u32,
    #[serde(
        rename = "analogueFilterChain",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub analogue_filter_chain: Option<String>,
    #[serde(
        rename = "digitalFilterChain",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub digital_filter_chain: Option<String>,
}

// ─── Response definitions ────────────────────────────────────────────

/// `<responsePAZ publicID="..." name="...">`
///
/// Poles and zeros response. `type` is:
/// - `A` = Laplace (radians/second)
/// - `B` = Laplace (Hz)
/// - `D` = Digital (Z-transform)
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlResponsePaz {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub paz_type: Option<String>,
    #[serde(rename = "gain", default, skip_serializing_if = "Option::is_none")]
    pub gain: Option<f64>,
    #[serde(
        rename = "gainFrequency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub gain_frequency: Option<f64>,
    #[serde(
        rename = "normalizationFactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub normalization_factor: Option<f64>,
    #[serde(
        rename = "normalizationFrequency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub normalization_frequency: Option<f64>,
    #[serde(
        rename = "numberOfZeros",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub number_of_zeros: Option<u32>,
    #[serde(
        rename = "numberOfPoles",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub number_of_poles: Option<u32>,
    #[serde(rename = "zeros", default, skip_serializing_if = "Option::is_none")]
    pub zeros: Option<String>,
    #[serde(rename = "poles", default, skip_serializing_if = "Option::is_none")]
    pub poles: Option<String>,
    #[serde(rename = "remark", default, skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

/// `<responseFIR publicID="..." name="...">`
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlResponseFir {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "gain", default, skip_serializing_if = "Option::is_none")]
    pub gain: Option<f64>,
    #[serde(
        rename = "gainFrequency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub gain_frequency: Option<f64>,
    #[serde(
        rename = "decimationFactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub decimation_factor: Option<u32>,
    #[serde(rename = "delay", default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<f64>,
    #[serde(
        rename = "correction",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub correction: Option<f64>,
    #[serde(
        rename = "numberOfCoefficients",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub number_of_coefficients: Option<u32>,
    #[serde(rename = "symmetry", default, skip_serializing_if = "Option::is_none")]
    pub symmetry: Option<String>,
    #[serde(
        rename = "coefficients",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub coefficients: Option<String>,
    #[serde(rename = "remark", default, skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

// ─── Hierarchy ───────────────────────────────────────────────────────

/// `<network publicID="..." code="...">`
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlNetwork {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "start", default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(rename = "end", default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "station", default)]
    pub stations: Vec<Sc3mlStation>,
}

/// `<station publicID="..." code="...">`
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlStation {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "start", default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(rename = "end", default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "latitude")]
    pub latitude: f64,
    #[serde(rename = "longitude")]
    pub longitude: f64,
    #[serde(rename = "elevation")]
    pub elevation: f64,
    #[serde(rename = "place", default, skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
    #[serde(rename = "country", default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(rename = "sensorLocation", default)]
    pub sensor_locations: Vec<Sc3mlSensorLocation>,
}

/// `<sensorLocation publicID="..." code="...">`
///
/// Groups streams by location code (equivalent to FDSN `locationCode`).
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlSensorLocation {
    #[serde(rename = "@publicID")]
    pub public_id: String,
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "start", default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(rename = "end", default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(rename = "latitude", default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(rename = "longitude", default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(rename = "elevation", default, skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
    #[serde(rename = "stream", default)]
    pub streams: Vec<Sc3mlStream>,
}

/// `<stream code="..." datalogger="..." sensor="...">`
///
/// A single measurement channel. `sensor` and `datalogger` attributes
/// reference top-level definitions by their `publicID`.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sc3mlStream {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(
        rename = "@datalogger",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub datalogger: Option<String>,
    #[serde(rename = "@sensor", default, skip_serializing_if = "Option::is_none")]
    pub sensor: Option<String>,
    #[serde(rename = "start", default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(rename = "end", default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(rename = "sampleRateNumerator", default)]
    pub sample_rate_numerator: u32,
    #[serde(rename = "sampleRateDenominator", default)]
    pub sample_rate_denominator: u32,
    #[serde(rename = "depth", default)]
    pub depth: f64,
    #[serde(rename = "azimuth", default)]
    pub azimuth: f64,
    #[serde(rename = "dip", default)]
    pub dip: f64,
    #[serde(rename = "gain", default, skip_serializing_if = "Option::is_none")]
    pub gain: Option<f64>,
    #[serde(
        rename = "gainFrequency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub gain_frequency: Option<f64>,
    #[serde(rename = "gainUnit", default, skip_serializing_if = "Option::is_none")]
    pub gain_unit: Option<String>,
    #[serde(
        rename = "dataloggerSerialNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub datalogger_serial_number: Option<String>,
    #[serde(
        rename = "sensorSerialNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sensor_serial_number: Option<String>,
    #[serde(
        rename = "dataloggerChannel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub datalogger_channel: Option<u32>,
    #[serde(
        rename = "sensorChannel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sensor_channel: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_root() {
        let xml = r#"<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13" version="0.13">
  <Inventory>
  </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(root.version.as_deref(), Some("0.13"));
        assert!(root.inventory.sensors.is_empty());
        assert!(root.inventory.networks.is_empty());
    }

    #[test]
    fn deserialize_sensor() {
        let xml = r#"<seiscomp version="0.13">
  <Inventory>
    <sensor publicID="Sensor#123" name="test" response="ResponsePAZ#456">
      <description>STS-2</description>
      <model>STS-2</model>
      <unit>M/S</unit>
    </sensor>
  </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(root.inventory.sensors.len(), 1);
        let s = &root.inventory.sensors[0];
        assert_eq!(s.public_id, "Sensor#123");
        assert_eq!(s.response.as_deref(), Some("ResponsePAZ#456"));
        assert_eq!(s.model.as_deref(), Some("STS-2"));
        assert_eq!(s.unit.as_deref(), Some("M/S"));
    }

    #[test]
    fn deserialize_datalogger_with_decimation() {
        let xml = r#"<seiscomp version="0.13">
  <Inventory>
    <datalogger publicID="DL#1" name="test">
      <gain>422552</gain>
      <maxClockDrift>0</maxClockDrift>
      <decimation sampleRateNumerator="40" sampleRateDenominator="1">
        <digitalFilterChain>FIR#1 FIR#2</digitalFilterChain>
      </decimation>
    </datalogger>
  </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        let dl = &root.inventory.dataloggers[0];
        assert_eq!(dl.public_id, "DL#1");
        assert!((dl.gain.unwrap() - 422552.0).abs() < 0.1);
        assert_eq!(dl.decimations.len(), 1);
        let dec = &dl.decimations[0];
        assert_eq!(dec.sample_rate_numerator, 40);
        assert_eq!(dec.sample_rate_denominator, 1);
        assert_eq!(dec.digital_filter_chain.as_deref(), Some("FIR#1 FIR#2"));
    }

    #[test]
    fn deserialize_response_paz() {
        let xml = r#"<seiscomp version="0.13">
  <Inventory>
    <responsePAZ publicID="PAZ#1">
      <type>A</type>
      <gain>1500</gain>
      <gainFrequency>1</gainFrequency>
      <normalizationFactor>2.3524e+17</normalizationFactor>
      <normalizationFrequency>1</normalizationFrequency>
      <numberOfZeros>2</numberOfZeros>
      <numberOfPoles>2</numberOfPoles>
      <zeros>(0,0) (0,0)</zeros>
      <poles>(-0.037,0.037) (-0.037,-0.037)</poles>
    </responsePAZ>
  </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        let paz = &root.inventory.response_paz[0];
        assert_eq!(paz.public_id, "PAZ#1");
        assert_eq!(paz.paz_type.as_deref(), Some("A"));
        assert!((paz.gain.unwrap() - 1500.0).abs() < 0.1);
        assert_eq!(paz.zeros.as_deref(), Some("(0,0) (0,0)"));
        assert_eq!(paz.poles.as_deref(), Some("(-0.037,0.037) (-0.037,-0.037)"));
    }

    #[test]
    fn deserialize_response_fir() {
        let xml = r#"<seiscomp version="0.13">
  <Inventory>
    <responseFIR publicID="FIR#1">
      <gain>1</gain>
      <decimationFactor>5</decimationFactor>
      <delay>0</delay>
      <correction>0</correction>
      <numberOfCoefficients>3</numberOfCoefficients>
      <symmetry>C</symmetry>
      <coefficients>0.1 0.2 0.3</coefficients>
    </responseFIR>
  </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        let fir = &root.inventory.response_fir[0];
        assert_eq!(fir.public_id, "FIR#1");
        assert_eq!(fir.decimation_factor, Some(5));
        assert_eq!(fir.symmetry.as_deref(), Some("C"));
        assert_eq!(fir.coefficients.as_deref(), Some("0.1 0.2 0.3"));
    }

    #[test]
    fn deserialize_network_hierarchy() {
        let xml = r#"<seiscomp version="0.13">
  <Inventory>
    <network publicID="Net/XX" code="XX">
      <start>2024-01-01T00:00:00.0000Z</start>
      <description>Test Network</description>
      <station publicID="Sta/PBUMI" code="PBUMI">
        <start>2024-06-01T00:00:00.0000Z</start>
        <latitude>-7.7714</latitude>
        <longitude>110.3776</longitude>
        <elevation>150</elevation>
        <sensorLocation publicID="Loc/00" code="00">
          <start>2024-06-01T00:00:00.0000Z</start>
          <stream code="SHZ" datalogger="DL#1" sensor="Sensor#1">
            <start>2024-06-01T00:00:00.0000Z</start>
            <sampleRateNumerator>100</sampleRateNumerator>
            <sampleRateDenominator>1</sampleRateDenominator>
            <depth>0</depth>
            <azimuth>0</azimuth>
            <dip>-90</dip>
            <gain>53687084.8</gain>
            <gainFrequency>15</gainFrequency>
            <gainUnit>M/S</gainUnit>
          </stream>
        </sensorLocation>
      </station>
    </network>
  </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(root.inventory.networks.len(), 1);
        let net = &root.inventory.networks[0];
        assert_eq!(net.code, "XX");
        assert_eq!(net.description.as_deref(), Some("Test Network"));

        let sta = &net.stations[0];
        assert_eq!(sta.code, "PBUMI");
        assert!((sta.latitude - (-7.7714)).abs() < 1e-6);

        let loc = &sta.sensor_locations[0];
        assert_eq!(loc.code, "00");

        let stream = &loc.streams[0];
        assert_eq!(stream.code, "SHZ");
        assert_eq!(stream.datalogger.as_deref(), Some("DL#1"));
        assert_eq!(stream.sensor.as_deref(), Some("Sensor#1"));
        assert_eq!(stream.sample_rate_numerator, 100);
        assert!((stream.dip - (-90.0)).abs() < 1e-6);
        assert!((stream.gain.unwrap() - 53687084.8).abs() < 0.1);
    }

    #[test]
    fn deserialize_real_sc3ml() {
        // Based on ObsPy's EB_response_sc3ml test fixture
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.7" version="0.7">
    <Inventory>
        <sensor publicID="Sensor#1" name="EBR.H" response="ResponsePAZ#1">
            <description>STS-2</description>
            <model>STS-2</model>
            <unit>M/S</unit>
        </sensor>
        <datalogger publicID="Datalogger#1" name="EBR.H">
            <gain>422552</gain>
            <maxClockDrift>0</maxClockDrift>
            <decimation sampleRateNumerator="40" sampleRateDenominator="1">
                <digitalFilterChain>ResponseFIR#1</digitalFilterChain>
            </decimation>
        </datalogger>
        <responsePAZ publicID="ResponsePAZ#1">
            <type>A</type>
            <gain>1500</gain>
            <gainFrequency>1</gainFrequency>
            <normalizationFactor>2.3524e+17</normalizationFactor>
            <normalizationFrequency>1</normalizationFrequency>
            <numberOfZeros>2</numberOfZeros>
            <numberOfPoles>2</numberOfPoles>
            <zeros>(0,0) (0,0)</zeros>
            <poles>(-0.037,0.037) (-0.037,-0.037)</poles>
        </responsePAZ>
        <responseFIR publicID="ResponseFIR#1">
            <gain>1</gain>
            <decimationFactor>5</decimationFactor>
            <delay>0</delay>
            <correction>0</correction>
            <numberOfCoefficients>3</numberOfCoefficients>
            <symmetry>C</symmetry>
            <coefficients>0.1 0.2 0.3</coefficients>
        </responseFIR>
        <network publicID="Network/EB" code="EB">
            <start>1980-01-01T00:00:00.0000Z</start>
            <description>SINGLE STATION</description>
            <station publicID="Station/EB/EBR" code="EBR">
                <start>2002-04-01T00:00:00.0000Z</start>
                <description>EBRO ROQUETAS, SPAIN</description>
                <latitude>40.8206</latitude>
                <longitude>0.4933</longitude>
                <elevation>40</elevation>
                <country>SPAIN</country>
                <sensorLocation publicID="SensorLocation#1" code="">
                    <start>2002-04-01T00:00:00.0000Z</start>
                    <latitude>40.8206</latitude>
                    <longitude>0.4933</longitude>
                    <elevation>40</elevation>
                    <stream code="BHZ" datalogger="Datalogger#1" sensor="Sensor#1">
                        <start>2002-04-01T00:00:00.0000Z</start>
                        <sampleRateNumerator>40</sampleRateNumerator>
                        <sampleRateDenominator>1</sampleRateDenominator>
                        <depth>0</depth>
                        <azimuth>0</azimuth>
                        <dip>-90</dip>
                        <gain>633828000</gain>
                        <gainFrequency>1</gainFrequency>
                        <gainUnit>M/S</gainUnit>
                    </stream>
                </sensorLocation>
            </station>
        </network>
    </Inventory>
</seiscomp>"#;
        let root: Sc3mlRoot = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(root.inventory.sensors.len(), 1);
        assert_eq!(root.inventory.dataloggers.len(), 1);
        assert_eq!(root.inventory.response_paz.len(), 1);
        assert_eq!(root.inventory.response_fir.len(), 1);
        assert_eq!(root.inventory.networks.len(), 1);
        assert_eq!(root.inventory.networks[0].stations[0].code, "EBR");
    }
}

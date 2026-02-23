//! SC3ML reader: XML → sc3ml types → Inventory.
//!
//! Resolves flat, reference-based SC3ML structure into the nested
//! `Inventory` hierarchy. Builds lookup maps for sensors, dataloggers,
//! and response definitions, then walks the network→station→sensorLocation→stream
//! tree to construct channels with resolved equipment and response info.

use std::collections::HashMap;

use crate::datetime::parse_datetime_opt;
use crate::error::{Result, StationXmlError};
use crate::inventory::*;

use super::types::*;

/// Parse SC3ML XML string into an [`Inventory`].
pub(crate) fn read_from_str(xml: &str) -> Result<Inventory> {
    let root: Sc3mlRoot = quick_xml::de::from_str(xml)?;
    sc3ml_to_inventory(root)
}

/// Parse SC3ML XML bytes into an [`Inventory`].
pub(crate) fn read_from_bytes(bytes: &[u8]) -> Result<Inventory> {
    let xml =
        std::str::from_utf8(bytes).map_err(|e| StationXmlError::InvalidData(e.to_string()))?;
    read_from_str(xml)
}

// ─── Response definition enum ────────────────────────────────────────

/// A resolved response definition (PAZ or FIR).
enum ResponseDef<'a> {
    Paz(&'a Sc3mlResponsePaz),
    Fir(&'a Sc3mlResponseFir),
}

// ─── Main conversion ─────────────────────────────────────────────────

fn sc3ml_to_inventory(root: Sc3mlRoot) -> Result<Inventory> {
    let inv = &root.inventory;

    // Build lookup maps for top-level definitions
    let sensors: HashMap<&str, &Sc3mlSensor> = inv
        .sensors
        .iter()
        .map(|s| (s.public_id.as_str(), s))
        .collect();

    let dataloggers: HashMap<&str, &Sc3mlDatalogger> = inv
        .dataloggers
        .iter()
        .map(|d| (d.public_id.as_str(), d))
        .collect();

    let mut responses: HashMap<&str, ResponseDef> = HashMap::new();
    for paz in &inv.response_paz {
        responses.insert(paz.public_id.as_str(), ResponseDef::Paz(paz));
    }
    for fir in &inv.response_fir {
        responses.insert(fir.public_id.as_str(), ResponseDef::Fir(fir));
    }

    let networks = inv
        .networks
        .iter()
        .map(|net| convert_network(net, &sensors, &dataloggers, &responses))
        .collect::<Result<Vec<_>>>()?;

    Ok(Inventory {
        source: "SeisComP".into(),
        sender: None,
        created: None,
        networks,
    })
}

// ─── Hierarchy conversion ────────────────────────────────────────────

fn convert_network(
    net: &Sc3mlNetwork,
    sensors: &HashMap<&str, &Sc3mlSensor>,
    dataloggers: &HashMap<&str, &Sc3mlDatalogger>,
    responses: &HashMap<&str, ResponseDef>,
) -> Result<Network> {
    let stations = net
        .stations
        .iter()
        .map(|sta| convert_station(sta, sensors, dataloggers, responses))
        .collect::<Result<Vec<_>>>()?;

    Ok(Network {
        code: net.code.clone(),
        description: net.description.clone(),
        start_date: parse_datetime_opt(&net.start)?,
        end_date: parse_datetime_opt(&net.end)?,
        stations,
    })
}

fn convert_station(
    sta: &Sc3mlStation,
    sensors: &HashMap<&str, &Sc3mlSensor>,
    dataloggers: &HashMap<&str, &Sc3mlDatalogger>,
    responses: &HashMap<&str, ResponseDef>,
) -> Result<Station> {
    // Flatten sensorLocation → channels
    let mut channels = Vec::new();
    for loc in &sta.sensor_locations {
        for stream in &loc.streams {
            let ch = convert_stream(stream, loc, sta, sensors, dataloggers, responses)?;
            channels.push(ch);
        }
    }

    // Use station description as site name if available, else use place or station code
    let site_name = sta
        .description
        .clone()
        .or_else(|| sta.place.clone())
        .unwrap_or_else(|| sta.code.clone());

    Ok(Station {
        code: sta.code.clone(),
        description: sta.description.clone(),
        latitude: sta.latitude,
        longitude: sta.longitude,
        elevation: sta.elevation,
        site: Site {
            name: site_name,
            country: sta.country.clone(),
            ..Default::default()
        },
        start_date: parse_datetime_opt(&sta.start)?,
        end_date: parse_datetime_opt(&sta.end)?,
        creation_date: None,
        channels,
    })
}

fn convert_stream(
    stream: &Sc3mlStream,
    loc: &Sc3mlSensorLocation,
    sta: &Sc3mlStation,
    sensors: &HashMap<&str, &Sc3mlSensor>,
    dataloggers: &HashMap<&str, &Sc3mlDatalogger>,
    responses: &HashMap<&str, ResponseDef>,
) -> Result<Channel> {
    // Use sensorLocation coordinates if available, else fall back to station
    let latitude = loc.latitude.unwrap_or(sta.latitude);
    let longitude = loc.longitude.unwrap_or(sta.longitude);
    let elevation = loc.elevation.unwrap_or(sta.elevation);

    // Compute sample rate
    let sample_rate = if stream.sample_rate_denominator > 0 {
        stream.sample_rate_numerator as f64 / stream.sample_rate_denominator as f64
    } else {
        0.0
    };

    // Resolve sensor equipment
    let sensor = stream
        .sensor
        .as_deref()
        .and_then(|id| sensors.get(id))
        .map(|s| convert_sensor_equipment(s, stream));

    // Resolve datalogger equipment
    let data_logger = stream
        .datalogger
        .as_deref()
        .and_then(|id| dataloggers.get(id))
        .map(|d| convert_datalogger_equipment(d, stream));

    // Build response
    let response = build_response(stream, sensors, dataloggers, responses)?;

    Ok(Channel {
        code: stream.code.clone(),
        location_code: loc.code.clone(),
        latitude,
        longitude,
        elevation,
        depth: stream.depth,
        azimuth: stream.azimuth,
        dip: stream.dip,
        sample_rate,
        start_date: parse_datetime_opt(&stream.start)?,
        end_date: parse_datetime_opt(&stream.end)?,
        sensor,
        data_logger,
        response,
    })
}

// ─── Equipment conversion ────────────────────────────────────────────

fn convert_sensor_equipment(sensor: &Sc3mlSensor, stream: &Sc3mlStream) -> Equipment {
    Equipment {
        equipment_type: sensor.sensor_type.clone().or(sensor.description.clone()),
        description: sensor.description.clone(),
        manufacturer: sensor.manufacturer.clone(),
        vendor: None,
        model: sensor.model.clone(),
        serial_number: stream.sensor_serial_number.clone(),
        installation_date: None,
        removal_date: None,
    }
}

fn convert_datalogger_equipment(dl: &Sc3mlDatalogger, stream: &Sc3mlStream) -> Equipment {
    Equipment {
        equipment_type: Some("Datalogger".into()),
        description: dl.description.clone(),
        manufacturer: None,
        vendor: None,
        model: dl.name.clone(),
        serial_number: stream.datalogger_serial_number.clone(),
        installation_date: None,
        removal_date: None,
    }
}

// ─── Response building ───────────────────────────────────────────────

fn build_response(
    stream: &Sc3mlStream,
    sensors: &HashMap<&str, &Sc3mlSensor>,
    dataloggers: &HashMap<&str, &Sc3mlDatalogger>,
    responses: &HashMap<&str, ResponseDef>,
) -> Result<Option<Response>> {
    let mut stages: Vec<ResponseStage> = Vec::new();
    let mut stage_number: u32 = 1;

    // Resolve sensor's response → responsePAZ (stage 1: sensor transfer function)
    let sensor_paz = stream
        .sensor
        .as_deref()
        .and_then(|id| sensors.get(id))
        .and_then(|s| s.response.as_deref())
        .and_then(|resp_id| responses.get(resp_id));

    // Resolve sensor unit for input_units
    let sensor_unit = stream
        .sensor
        .as_deref()
        .and_then(|id| sensors.get(id))
        .and_then(|s| s.unit.as_deref())
        .unwrap_or("M/S");

    if let Some(ResponseDef::Paz(paz)) = sensor_paz {
        let pz_stage = convert_paz_to_stage(paz, stage_number, sensor_unit, "V")?;
        stages.push(pz_stage);
        stage_number += 1;
    }

    // Resolve datalogger and its decimation filter chains
    let dl = stream
        .datalogger
        .as_deref()
        .and_then(|id| dataloggers.get(id));

    if let Some(dl) = dl {
        // Find matching decimation for this stream's sample rate
        let decim = dl.decimations.iter().find(|d| {
            d.sample_rate_numerator == stream.sample_rate_numerator
                && d.sample_rate_denominator == stream.sample_rate_denominator
        });

        if let Some(decim) = decim {
            // Analogue filter chain → PAZ stages
            if let Some(chain) = &decim.analogue_filter_chain {
                for ref_id in chain.split_whitespace() {
                    if let Some(ResponseDef::Paz(paz)) = responses.get(ref_id) {
                        let pz_stage = convert_paz_to_stage(paz, stage_number, "V", "V")?;
                        stages.push(pz_stage);
                        stage_number += 1;
                    }
                }
            }

            // Datalogger gain stage (V → COUNTS)
            if let Some(dl_gain) = dl.gain {
                let sample_rate = if stream.sample_rate_denominator > 0 {
                    stream.sample_rate_numerator as f64 / stream.sample_rate_denominator as f64
                } else {
                    0.0
                };

                stages.push(ResponseStage {
                    number: stage_number,
                    stage_gain: Some(StageGain {
                        value: dl_gain,
                        frequency: 0.0,
                    }),
                    poles_zeros: None,
                    coefficients: Some(Coefficients {
                        input_units: Units {
                            name: "V".into(),
                            description: None,
                        },
                        output_units: Units {
                            name: "COUNTS".into(),
                            description: None,
                        },
                        cf_transfer_function_type: CfTransferFunction::Digital,
                        numerators: vec![1.0],
                        denominators: vec![],
                    }),
                    fir: None,
                    decimation: Some(Decimation {
                        input_sample_rate: sample_rate,
                        factor: 1,
                        offset: 0,
                        delay: 0.0,
                        correction: 0.0,
                    }),
                });
                stage_number += 1;
            }

            // Digital filter chain → FIR stages
            if let Some(chain) = &decim.digital_filter_chain {
                for ref_id in chain.split_whitespace() {
                    if let Some(ResponseDef::Fir(fir)) = responses.get(ref_id) {
                        let fir_stage = convert_fir_to_stage(fir, stage_number)?;
                        stages.push(fir_stage);
                        stage_number += 1;
                    }
                }
            }
        } else if let Some(dl_gain) = dl.gain {
            // No matching decimation but datalogger has gain
            stages.push(ResponseStage {
                number: stage_number,
                stage_gain: Some(StageGain {
                    value: dl_gain,
                    frequency: 0.0,
                }),
                poles_zeros: None,
                coefficients: Some(Coefficients {
                    input_units: Units {
                        name: "V".into(),
                        description: None,
                    },
                    output_units: Units {
                        name: "COUNTS".into(),
                        description: None,
                    },
                    cf_transfer_function_type: CfTransferFunction::Digital,
                    numerators: vec![1.0],
                    denominators: vec![],
                }),
                fir: None,
                decimation: None,
            });
            stage_number += 1;
        }
    }

    // Build InstrumentSensitivity from stream's gain info
    let instrument_sensitivity = if let Some(gain_val) = stream.gain {
        let gain_unit = stream.gain_unit.as_deref().unwrap_or(sensor_unit);
        Some(InstrumentSensitivity {
            value: gain_val,
            frequency: stream.gain_frequency.unwrap_or(1.0),
            input_units: Units {
                name: gain_unit.to_string(),
                description: None,
            },
            output_units: Units {
                name: "COUNTS".into(),
                description: None,
            },
        })
    } else {
        None
    };

    // Return None if no response info at all
    let _ = stage_number; // suppress unused warning
    if instrument_sensitivity.is_none() && stages.is_empty() {
        return Ok(None);
    }

    Ok(Some(Response {
        instrument_sensitivity,
        stages,
    }))
}

// ─── PAZ → ResponseStage ────────────────────────────────────────────

fn convert_paz_to_stage(
    paz: &Sc3mlResponsePaz,
    number: u32,
    input_unit: &str,
    output_unit: &str,
) -> Result<ResponseStage> {
    let pz_type = paz.paz_type.as_deref().unwrap_or("A");

    let pz_transfer_function_type = match pz_type {
        "A" => PzTransferFunction::LaplaceRadians,
        "B" => PzTransferFunction::LaplaceHertz,
        "D" => PzTransferFunction::DigitalZTransform,
        _ => {
            return Err(StationXmlError::InvalidData(format!(
                "unknown PAZ type: '{pz_type}'"
            )));
        }
    };

    let zeros = paz
        .zeros
        .as_deref()
        .map(parse_complex_array)
        .transpose()?
        .unwrap_or_default();

    let poles = paz
        .poles
        .as_deref()
        .map(parse_complex_array)
        .transpose()?
        .unwrap_or_default();

    Ok(ResponseStage {
        number,
        stage_gain: paz.gain.map(|g| StageGain {
            value: g,
            frequency: paz.gain_frequency.unwrap_or(1.0),
        }),
        poles_zeros: Some(PolesZeros {
            input_units: Units {
                name: input_unit.into(),
                description: None,
            },
            output_units: Units {
                name: output_unit.into(),
                description: None,
            },
            pz_transfer_function_type,
            normalization_factor: paz.normalization_factor.unwrap_or(1.0),
            normalization_frequency: paz.normalization_frequency.unwrap_or(1.0),
            zeros,
            poles,
        }),
        coefficients: None,
        fir: None,
        decimation: None,
    })
}

// ─── FIR → ResponseStage ────────────────────────────────────────────

fn convert_fir_to_stage(fir: &Sc3mlResponseFir, number: u32) -> Result<ResponseStage> {
    let symmetry = match fir.symmetry.as_deref().unwrap_or("A") {
        "A" => Symmetry::None,
        "B" => Symmetry::Odd,
        "C" => Symmetry::Even,
        other => {
            return Err(StationXmlError::InvalidData(format!(
                "unknown FIR symmetry: '{other}'"
            )));
        }
    };

    let coefficients = fir
        .coefficients
        .as_deref()
        .map(parse_float_array)
        .transpose()?
        .unwrap_or_default();

    let decimation_factor = fir.decimation_factor.unwrap_or(1);
    let input_sample_rate = 0.0; // Not available in SC3ML FIR definition

    Ok(ResponseStage {
        number,
        stage_gain: fir.gain.map(|g| StageGain {
            value: g,
            frequency: fir.gain_frequency.unwrap_or(0.0),
        }),
        poles_zeros: None,
        coefficients: None,
        fir: Some(FIR {
            input_units: Units {
                name: "COUNTS".into(),
                description: None,
            },
            output_units: Units {
                name: "COUNTS".into(),
                description: None,
            },
            symmetry,
            numerator_coefficients: coefficients,
        }),
        decimation: Some(Decimation {
            input_sample_rate,
            factor: decimation_factor,
            offset: 0,
            delay: fir.delay.unwrap_or(0.0),
            correction: fir.correction.unwrap_or(0.0),
        }),
    })
}

// ─── Complex number parsing ─────────────────────────────────────────

/// Parse SC3ML complex number array: `"(0,0) (0,0) (-0.037,0.037)"`
///
/// Each complex number is in `(real,imag)` format, separated by whitespace.
fn parse_complex_array(s: &str) -> Result<Vec<PoleZero>> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut number: u32 = 0;

    // Find all (real,imag) pairs
    let mut chars = s.chars().peekable();
    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }

        // Expect '('
        match chars.next() {
            Some('(') => {}
            Some(c) => {
                return Err(StationXmlError::InvalidData(format!(
                    "expected '(' in complex number, got '{c}'"
                )));
            }
            None => break,
        }

        // Read until ','
        let real_str: String = chars.by_ref().take_while(|&c| c != ',').collect();
        let real: f64 = real_str.trim().parse().map_err(|_| {
            StationXmlError::InvalidData(format!("cannot parse real part: '{real_str}'"))
        })?;

        // Read until ')'
        let imag_str: String = chars.by_ref().take_while(|&c| c != ')').collect();
        let imaginary: f64 = imag_str.trim().parse().map_err(|_| {
            StationXmlError::InvalidData(format!("cannot parse imaginary part: '{imag_str}'"))
        })?;

        result.push(PoleZero {
            number,
            real,
            imaginary,
        });
        number += 1;
    }

    Ok(result)
}

/// Parse space-separated float values: `"0.1 0.2 0.3"`
fn parse_float_array(s: &str) -> Result<Vec<f64>> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(vec![]);
    }
    s.split_whitespace()
        .map(|tok| {
            tok.parse::<f64>()
                .map_err(|_| StationXmlError::InvalidData(format!("cannot parse float: '{tok}'")))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_complex_simple() {
        let result = parse_complex_array("(0,0) (0,0)").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].number, 0);
        assert!((result[0].real).abs() < 1e-10);
        assert!((result[0].imaginary).abs() < 1e-10);
    }

    #[test]
    fn parse_complex_with_values() {
        let result = parse_complex_array("(-0.037,0.037) (-0.037,-0.037)").unwrap();
        assert_eq!(result.len(), 2);
        assert!((result[0].real - (-0.037)).abs() < 1e-6);
        assert!((result[0].imaginary - 0.037).abs() < 1e-6);
        assert!((result[1].imaginary - (-0.037)).abs() < 1e-6);
    }

    #[test]
    fn parse_complex_scientific() {
        let result = parse_complex_array("(-5907,-3411) (-5907,3411)").unwrap();
        assert_eq!(result.len(), 2);
        assert!((result[0].real - (-5907.0)).abs() < 1e-6);
    }

    #[test]
    fn parse_complex_empty() {
        let result = parse_complex_array("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_complex_extra_whitespace() {
        let result = parse_complex_array("  (0,0)  (-1,2)  ").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_floats() {
        let result = parse_float_array("0.1 0.2 0.3").unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn parse_floats_scientific() {
        let result = parse_float_array("2.3524e+17 -3.37741e-07").unwrap();
        assert_eq!(result.len(), 2);
        assert!((result[0] - 2.3524e17).abs() < 1e10);
    }

    #[test]
    fn parse_floats_empty() {
        let result = parse_float_array("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn read_channel_level() {
        // Based on ObsPy's channel_level.sc3ml — no responsePAZ/FIR,
        // just sensor + datalogger with gain in stream
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<seiscomp xmlns="http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.9" version="0.9">
  <Inventory>
    <sensor publicID="Sensor#1" name="HGN.HZ">
      <description>STS-1</description>
      <model>STS-1</model>
      <unit>M/S</unit>
    </sensor>
    <datalogger publicID="Datalogger#1" name="HGN.BHZ">
      <description>HGN.BHZ</description>
    </datalogger>
    <network publicID="Network/NL" code="NL">
      <start>1980-01-01T00:00:00.0000Z</start>
      <description>Netherlands Seismic Network</description>
      <station publicID="Station/NL/HGN" code="HGN">
        <start>1993-01-01T00:00:00.0000Z</start>
        <description>HEIMANSGROEVE, NETHERLANDS</description>
        <latitude>50.764</latitude>
        <longitude>5.9317</longitude>
        <elevation>135</elevation>
        <country>The Netherlands</country>
        <sensorLocation publicID="SensorLocation#1" code="">
          <start>1993-11-03T00:00:00.0000Z</start>
          <latitude>50.764</latitude>
          <longitude>5.9317</longitude>
          <elevation>135</elevation>
          <stream code="BHZ" datalogger="Datalogger#1" sensor="Sensor#1">
            <start>1993-11-03T00:00:00.0000Z</start>
            <end>2003-10-24T00:00:00.0000Z</end>
            <sampleRateNumerator>40</sampleRateNumerator>
            <sampleRateDenominator>1</sampleRateDenominator>
            <depth>4</depth>
            <azimuth>0</azimuth>
            <dip>-90</dip>
            <gain>814301000</gain>
            <gainFrequency>1</gainFrequency>
            <gainUnit>M/S</gainUnit>
          </stream>
        </sensorLocation>
      </station>
    </network>
  </Inventory>
</seiscomp>"#;
        let inv = read_from_str(xml).unwrap();
        assert_eq!(inv.networks.len(), 1);
        let net = &inv.networks[0];
        assert_eq!(net.code, "NL");

        let sta = &net.stations[0];
        assert_eq!(sta.code, "HGN");
        assert!((sta.latitude - 50.764).abs() < 1e-6);
        assert_eq!(sta.site.country.as_deref(), Some("The Netherlands"));

        let ch = &sta.channels[0];
        assert_eq!(ch.code, "BHZ");
        assert_eq!(ch.location_code, "");
        assert!((ch.sample_rate - 40.0).abs() < 1e-6);
        assert!((ch.depth - 4.0).abs() < 1e-6);
        assert!((ch.dip - (-90.0)).abs() < 1e-6);

        // Sensor equipment
        let sensor = ch.sensor.as_ref().unwrap();
        assert_eq!(sensor.model.as_deref(), Some("STS-1"));

        // Response — gain only, no stages
        let resp = ch.response.as_ref().unwrap();
        let sens = resp.instrument_sensitivity.as_ref().unwrap();
        assert!((sens.value - 814301000.0).abs() < 0.1);
        assert!((sens.frequency - 1.0).abs() < 1e-6);
        assert_eq!(sens.input_units.name, "M/S");
    }

    #[test]
    fn read_with_response_paz_and_fir() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<seiscomp version="0.13">
  <Inventory>
    <sensor publicID="Sensor#1" response="ResponsePAZ#1">
      <description>STS-2</description>
      <model>STS-2</model>
      <unit>M/S</unit>
    </sensor>
    <datalogger publicID="Datalogger#1">
      <gain>422552</gain>
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
    <network publicID="Net/EB" code="EB">
      <start>1980-01-01T00:00:00.0000Z</start>
      <station publicID="Sta/EBR" code="EBR">
        <start>2002-04-01T00:00:00.0000Z</start>
        <latitude>40.8206</latitude>
        <longitude>0.4933</longitude>
        <elevation>40</elevation>
        <sensorLocation publicID="Loc#1" code="">
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
        let inv = read_from_str(xml).unwrap();
        let ch = &inv.networks[0].stations[0].channels[0];
        assert_eq!(ch.code, "BHZ");

        let resp = ch.response.as_ref().unwrap();

        // InstrumentSensitivity from stream gain
        let sens = resp.instrument_sensitivity.as_ref().unwrap();
        assert!((sens.value - 633828000.0).abs() < 0.1);

        // Stage 1: Poles & Zeros from sensor → responsePAZ
        assert!(resp.stages.len() >= 3);
        let s1 = &resp.stages[0];
        assert_eq!(s1.number, 1);
        let pz = s1.poles_zeros.as_ref().unwrap();
        assert_eq!(
            pz.pz_transfer_function_type,
            PzTransferFunction::LaplaceRadians
        );
        assert_eq!(pz.zeros.len(), 2);
        assert_eq!(pz.poles.len(), 2);
        assert!((pz.poles[0].real - (-0.037)).abs() < 1e-6);
        assert!((s1.stage_gain.as_ref().unwrap().value - 1500.0).abs() < 0.1);

        // Stage 2: Datalogger gain (V → COUNTS)
        let s2 = &resp.stages[1];
        assert_eq!(s2.number, 2);
        let cf = s2.coefficients.as_ref().unwrap();
        assert_eq!(cf.input_units.name, "V");
        assert_eq!(cf.output_units.name, "COUNTS");
        assert!((s2.stage_gain.as_ref().unwrap().value - 422552.0).abs() < 0.1);

        // Stage 3: FIR filter
        let s3 = &resp.stages[2];
        assert_eq!(s3.number, 3);
        let fir = s3.fir.as_ref().unwrap();
        assert_eq!(fir.symmetry, Symmetry::Even);
        assert_eq!(fir.numerator_coefficients.len(), 3);
        assert!((fir.numerator_coefficients[0] - 0.1).abs() < 1e-6);
        let dec = s3.decimation.as_ref().unwrap();
        assert_eq!(dec.factor, 5);
    }

    #[test]
    fn read_multiple_locations() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<seiscomp version="0.13">
  <Inventory>
    <sensor publicID="Sensor#1">
      <model>STS-2</model>
      <unit>M/S</unit>
    </sensor>
    <datalogger publicID="DL#1"/>
    <network publicID="Net/XX" code="XX">
      <station publicID="Sta/TEST" code="TEST">
        <latitude>0</latitude>
        <longitude>0</longitude>
        <elevation>0</elevation>
        <sensorLocation publicID="Loc/00" code="00">
          <stream code="BHZ" sensor="Sensor#1" datalogger="DL#1">
            <sampleRateNumerator>20</sampleRateNumerator>
            <sampleRateDenominator>1</sampleRateDenominator>
            <depth>0</depth>
            <azimuth>0</azimuth>
            <dip>-90</dip>
          </stream>
        </sensorLocation>
        <sensorLocation publicID="Loc/10" code="10">
          <stream code="HHZ" sensor="Sensor#1" datalogger="DL#1">
            <sampleRateNumerator>100</sampleRateNumerator>
            <sampleRateDenominator>1</sampleRateDenominator>
            <depth>5</depth>
            <azimuth>0</azimuth>
            <dip>-90</dip>
          </stream>
        </sensorLocation>
      </station>
    </network>
  </Inventory>
</seiscomp>"#;
        let inv = read_from_str(xml).unwrap();
        let sta = &inv.networks[0].stations[0];
        assert_eq!(sta.channels.len(), 2);
        assert_eq!(sta.channels[0].location_code, "00");
        assert_eq!(sta.channels[0].code, "BHZ");
        assert!((sta.channels[0].sample_rate - 20.0).abs() < 1e-6);
        assert_eq!(sta.channels[1].location_code, "10");
        assert_eq!(sta.channels[1].code, "HHZ");
        assert!((sta.channels[1].sample_rate - 100.0).abs() < 1e-6);
    }

    #[test]
    fn read_from_bytes_works() {
        let xml = r#"<?xml version="1.0"?>
<seiscomp version="0.13">
  <Inventory>
    <network publicID="Net/XX" code="XX">
      <station publicID="Sta/T" code="T">
        <latitude>0</latitude>
        <longitude>0</longitude>
        <elevation>0</elevation>
      </station>
    </network>
  </Inventory>
</seiscomp>"#;
        let inv = read_from_bytes(xml.as_bytes()).unwrap();
        assert_eq!(inv.networks[0].code, "XX");
    }

    #[test]
    fn read_zero_poles_and_zeros() {
        // Test responsePAZ with numberOfPoles=0 numberOfZeros=0
        let xml = r#"<?xml version="1.0"?>
<seiscomp version="0.13">
  <Inventory>
    <sensor publicID="S#1" response="PAZ#1">
      <unit>M/S</unit>
    </sensor>
    <datalogger publicID="DL#1">
      <gain>6553.6</gain>
      <decimation sampleRateNumerator="40" sampleRateDenominator="1">
        <analogueFilterChain>PAZ#2</analogueFilterChain>
      </decimation>
    </datalogger>
    <responsePAZ publicID="PAZ#1">
      <type>B</type>
      <gain>1383.04</gain>
      <gainFrequency>0.05</gainFrequency>
      <normalizationFactor>3.60287e+16</normalizationFactor>
      <normalizationFrequency>0.05</normalizationFrequency>
      <numberOfZeros>2</numberOfZeros>
      <numberOfPoles>2</numberOfPoles>
      <zeros>(0,0) (0,0)</zeros>
      <poles>(-0.00707,0.00707) (-0.00707,-0.00707)</poles>
    </responsePAZ>
    <responsePAZ publicID="PAZ#2">
      <type>B</type>
      <gain>33.3625</gain>
      <gainFrequency>0.05</gainFrequency>
      <normalizationFactor>1</normalizationFactor>
      <normalizationFrequency>0.05</normalizationFrequency>
      <numberOfZeros>0</numberOfZeros>
      <numberOfPoles>0</numberOfPoles>
    </responsePAZ>
    <network publicID="Net/XX" code="XX">
      <station publicID="Sta/T" code="T">
        <latitude>0</latitude>
        <longitude>0</longitude>
        <elevation>0</elevation>
        <sensorLocation publicID="Loc#1" code="10">
          <stream code="BHZ" sensor="S#1" datalogger="DL#1">
            <sampleRateNumerator>40</sampleRateNumerator>
            <sampleRateDenominator>1</sampleRateDenominator>
            <depth>7</depth>
            <azimuth>0</azimuth>
            <dip>-90</dip>
            <gain>1209570000</gain>
            <gainFrequency>0.05</gainFrequency>
            <gainUnit>m/s</gainUnit>
          </stream>
        </sensorLocation>
      </station>
    </network>
  </Inventory>
</seiscomp>"#;
        let inv = read_from_str(xml).unwrap();
        let ch = &inv.networks[0].stations[0].channels[0];
        let resp = ch.response.as_ref().unwrap();

        // Stage 1: sensor PAZ with poles and zeros
        let s1 = &resp.stages[0];
        let pz1 = s1.poles_zeros.as_ref().unwrap();
        assert_eq!(
            pz1.pz_transfer_function_type,
            PzTransferFunction::LaplaceHertz
        );
        assert_eq!(pz1.zeros.len(), 2);
        assert_eq!(pz1.poles.len(), 2);

        // Stage 2: analogue filter PAZ with 0 poles, 0 zeros
        let s2 = &resp.stages[1];
        let pz2 = s2.poles_zeros.as_ref().unwrap();
        assert!(pz2.zeros.is_empty());
        assert!(pz2.poles.is_empty());
        assert!((s2.stage_gain.as_ref().unwrap().value - 33.3625).abs() < 0.001);

        // Stage 3: datalogger gain
        let s3 = &resp.stages[2];
        assert!((s3.stage_gain.as_ref().unwrap().value - 6553.6).abs() < 0.1);
    }
}

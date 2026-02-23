//! SC3ML writer: Inventory → sc3ml types → XML.
//!
//! Converts the nested Inventory hierarchy into SC3ML's flat, reference-based
//! structure. Deduplicates sensors, dataloggers, and response definitions
//! by generating stable `publicID` identifiers.

use std::collections::HashMap;

use crate::datetime::format_datetime_opt;
use crate::error::Result;
use crate::inventory::*;

use super::types::*;

/// Serialize an [`Inventory`] to an SC3ML XML string.
pub(crate) fn write_to_string(inventory: &Inventory) -> Result<String> {
    let sc3ml = inventory_to_sc3ml(inventory);
    let body = quick_xml::se::to_string(&sc3ml)?;
    let mut xml = String::with_capacity(body.len() + 50);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(&body);
    Ok(xml)
}

// ─── Top-level conversion ────────────────────────────────────────────

/// Collected top-level definitions during hierarchy traversal.
struct Definitions {
    sensors: Vec<Sc3mlSensor>,
    dataloggers: Vec<Sc3mlDatalogger>,
    response_paz: Vec<Sc3mlResponsePaz>,
    response_fir: Vec<Sc3mlResponseFir>,
    /// Map: dedup key → publicID (for sensors)
    sensor_map: HashMap<String, String>,
    /// Map: dedup key → publicID (for dataloggers)
    datalogger_map: HashMap<String, String>,
    /// Map: dedup key → publicID (for PAZ responses)
    paz_map: HashMap<String, String>,
    /// Map: dedup key → publicID (for FIR responses)
    fir_map: HashMap<String, String>,
    /// Counter for unique IDs
    id_counter: u32,
}

impl Definitions {
    fn new() -> Self {
        Self {
            sensors: Vec::new(),
            dataloggers: Vec::new(),
            response_paz: Vec::new(),
            response_fir: Vec::new(),
            sensor_map: HashMap::new(),
            datalogger_map: HashMap::new(),
            paz_map: HashMap::new(),
            fir_map: HashMap::new(),
            id_counter: 1,
        }
    }

    fn next_id(&mut self) -> u32 {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }
}

fn inventory_to_sc3ml(inv: &Inventory) -> Sc3mlRoot {
    let mut defs = Definitions::new();

    // First pass: collect all definitions from channels
    let networks: Vec<Sc3mlNetwork> = inv
        .networks
        .iter()
        .map(|net| convert_network(net, &mut defs))
        .collect();

    Sc3mlRoot {
        xmlns: Some("http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13".into()),
        version: Some("0.13".into()),
        inventory: Sc3mlInventory {
            sensors: defs.sensors,
            dataloggers: defs.dataloggers,
            response_paz: defs.response_paz,
            response_fir: defs.response_fir,
            networks,
        },
    }
}

// ─── Hierarchy conversion ────────────────────────────────────────────

fn convert_network(net: &Network, defs: &mut Definitions) -> Sc3mlNetwork {
    let id = defs.next_id();
    Sc3mlNetwork {
        public_id: format!("Network/{}", net.code),
        code: net.code.clone(),
        start: format_datetime_opt(&net.start_date),
        end: format_datetime_opt(&net.end_date),
        description: net.description.clone(),
        stations: net
            .stations
            .iter()
            .map(|sta| convert_station(sta, &net.code, defs, id))
            .collect(),
    }
}

fn convert_station(
    sta: &Station,
    net_code: &str,
    defs: &mut Definitions,
    _net_id: u32,
) -> Sc3mlStation {
    // Group channels by location_code → sensorLocations
    let mut loc_groups: Vec<(String, Vec<&Channel>)> = Vec::new();
    for ch in &sta.channels {
        if let Some(group) = loc_groups
            .iter_mut()
            .find(|(code, _)| *code == ch.location_code)
        {
            group.1.push(ch);
        } else {
            loc_groups.push((ch.location_code.clone(), vec![ch]));
        }
    }

    let sensor_locations: Vec<Sc3mlSensorLocation> = loc_groups
        .into_iter()
        .map(|(loc_code, channels)| {
            convert_sensor_location(sta, net_code, &loc_code, &channels, defs)
        })
        .collect();

    Sc3mlStation {
        public_id: format!("Station/{net_code}/{}", sta.code),
        code: sta.code.clone(),
        start: format_datetime_opt(&sta.start_date),
        end: format_datetime_opt(&sta.end_date),
        description: sta.description.clone().or(Some(sta.site.name.clone())),
        latitude: sta.latitude,
        longitude: sta.longitude,
        elevation: sta.elevation,
        place: sta.site.town.clone().or(sta.site.region.clone()),
        country: sta.site.country.clone(),
        sensor_locations,
    }
}

fn convert_sensor_location(
    sta: &Station,
    net_code: &str,
    loc_code: &str,
    channels: &[&Channel],
    defs: &mut Definitions,
) -> Sc3mlSensorLocation {
    // Use first channel's coordinates (they should be identical for same location)
    let first = channels.first();
    let latitude = first.map(|ch| ch.latitude);
    let longitude = first.map(|ch| ch.longitude);
    let elevation = first.map(|ch| ch.elevation);

    let loc_id = defs.next_id();
    let streams: Vec<Sc3mlStream> = channels
        .iter()
        .map(|ch| convert_stream(ch, sta, net_code, loc_code, defs))
        .collect();

    Sc3mlSensorLocation {
        public_id: format!("SensorLocation/{net_code}/{}/{loc_code}/{loc_id}", sta.code),
        code: loc_code.into(),
        start: format_datetime_opt(&first.and_then(|ch| ch.start_date)),
        end: None,
        latitude,
        longitude,
        elevation,
        streams,
    }
}

fn convert_stream(
    ch: &Channel,
    sta: &Station,
    net_code: &str,
    _loc_code: &str,
    defs: &mut Definitions,
) -> Sc3mlStream {
    // Resolve or create sensor definition
    let sensor_public_id = ch
        .sensor
        .as_ref()
        .map(|eq| get_or_create_sensor(eq, ch, sta, net_code, defs));

    // Resolve or create datalogger definition
    let datalogger_public_id = ch
        .data_logger
        .as_ref()
        .map(|eq| get_or_create_datalogger(eq, ch, sta, net_code, defs));

    // Compute sample rate as numerator/denominator
    let (num, denom) = float_to_fraction(ch.sample_rate);

    // Build gain info from InstrumentSensitivity
    let (gain, gain_frequency, gain_unit) = ch
        .response
        .as_ref()
        .and_then(|r| r.instrument_sensitivity.as_ref())
        .map(|s| {
            (
                Some(s.value),
                Some(s.frequency),
                Some(s.input_units.name.clone()),
            )
        })
        .unwrap_or((None, None, None));

    Sc3mlStream {
        code: ch.code.clone(),
        datalogger: datalogger_public_id,
        sensor: sensor_public_id,
        start: format_datetime_opt(&ch.start_date),
        end: format_datetime_opt(&ch.end_date),
        sample_rate_numerator: num,
        sample_rate_denominator: denom,
        depth: ch.depth,
        azimuth: ch.azimuth,
        dip: ch.dip,
        gain,
        gain_frequency,
        gain_unit,
        datalogger_serial_number: ch
            .data_logger
            .as_ref()
            .and_then(|eq| eq.serial_number.clone()),
        sensor_serial_number: ch.sensor.as_ref().and_then(|eq| eq.serial_number.clone()),
        datalogger_channel: None,
        sensor_channel: None,
    }
}

// ─── Deduplication helpers ───────────────────────────────────────────

/// Generate a dedup key for a sensor based on model + manufacturer.
fn sensor_dedup_key(eq: &Equipment) -> String {
    format!(
        "{}|{}",
        eq.model.as_deref().unwrap_or(""),
        eq.manufacturer.as_deref().unwrap_or("")
    )
}

/// Generate a dedup key for a datalogger based on model + manufacturer.
fn datalogger_dedup_key(eq: &Equipment) -> String {
    format!(
        "{}|{}",
        eq.model.as_deref().unwrap_or(""),
        eq.manufacturer.as_deref().unwrap_or("")
    )
}

fn get_or_create_sensor(
    eq: &Equipment,
    ch: &Channel,
    _sta: &Station,
    _net_code: &str,
    defs: &mut Definitions,
) -> String {
    let key = sensor_dedup_key(eq);

    // Check for existing with matching response
    if let Some(public_id) = defs.sensor_map.get(&key) {
        return public_id.clone();
    }

    let id = defs.next_id();
    let public_id = format!(
        "Sensor/{}",
        eq.model
            .as_deref()
            .unwrap_or("unknown")
            .replace([' ', '/'], "_")
    );
    let public_id = format!("{public_id}_{id}");

    // Extract response PAZ from channel response stages (stage 1 is typically sensor)
    let response_paz_id = ch.response.as_ref().and_then(|resp| {
        resp.stages
            .iter()
            .find(|s| s.poles_zeros.is_some())
            .and_then(|stage| {
                stage
                    .poles_zeros
                    .as_ref()
                    .map(|pz| get_or_create_paz(pz, stage, defs))
            })
    });

    // Determine unit from sensor PZ input units or from gain
    let unit = ch
        .response
        .as_ref()
        .and_then(|r| {
            r.stages
                .first()
                .and_then(|s| s.poles_zeros.as_ref())
                .map(|pz| pz.input_units.name.clone())
        })
        .or_else(|| {
            ch.response
                .as_ref()
                .and_then(|r| r.instrument_sensitivity.as_ref())
                .map(|s| s.input_units.name.clone())
        });

    defs.sensors.push(Sc3mlSensor {
        public_id: public_id.clone(),
        name: None,
        response: response_paz_id,
        description: eq.description.clone(),
        model: eq.model.clone(),
        manufacturer: eq.manufacturer.clone(),
        sensor_type: eq.equipment_type.clone(),
        unit,
        remark: None,
    });

    defs.sensor_map.insert(key, public_id.clone());
    public_id
}

fn get_or_create_datalogger(
    eq: &Equipment,
    ch: &Channel,
    _sta: &Station,
    _net_code: &str,
    defs: &mut Definitions,
) -> String {
    // Don't dedup dataloggers — each channel may have different sample rates/filters
    let id = defs.next_id();
    let public_id = format!(
        "Datalogger/{}_{id}",
        eq.model
            .as_deref()
            .unwrap_or("unknown")
            .replace([' ', '/'], "_")
    );

    // Extract datalogger gain from response stages (typically stage with Coefficients V→COUNTS)
    let dl_gain = ch.response.as_ref().and_then(|resp| {
        resp.stages
            .iter()
            .find(|s| {
                s.coefficients
                    .as_ref()
                    .is_some_and(|cf| cf.output_units.name == "COUNTS")
            })
            .and_then(|s| s.stage_gain.as_ref().map(|g| g.value))
    });

    // Compute sample rate fraction
    let (num, denom) = float_to_fraction(ch.sample_rate);

    // Build decimation with filter chains
    let mut decimations = Vec::new();

    // Check for FIR stages → digital filter chain
    let fir_refs: Vec<String> = ch
        .response
        .as_ref()
        .map(|resp| {
            resp.stages
                .iter()
                .filter_map(|s| s.fir.as_ref().map(|fir| get_or_create_fir(fir, s, defs)))
                .collect()
        })
        .unwrap_or_default();

    let digital_filter_chain = if fir_refs.is_empty() {
        None
    } else {
        Some(fir_refs.join(" "))
    };

    if dl_gain.is_some() || digital_filter_chain.is_some() {
        decimations.push(Sc3mlDecimation {
            sample_rate_numerator: num,
            sample_rate_denominator: denom,
            analogue_filter_chain: None,
            digital_filter_chain,
        });
    }

    defs.dataloggers.push(Sc3mlDatalogger {
        public_id: public_id.clone(),
        name: eq.model.clone(),
        description: eq.description.clone(),
        gain: dl_gain,
        max_clock_drift: Some(0.0),
        decimations,
        remark: None,
    });

    defs.datalogger_map
        .insert(datalogger_dedup_key(eq), public_id.clone());
    public_id
}

fn get_or_create_paz(pz: &PolesZeros, stage: &ResponseStage, defs: &mut Definitions) -> String {
    // Build a dedup key from normalization + poles + zeros
    let key = format!(
        "{}|{}|{}|{:?}|{:?}",
        pz.normalization_factor,
        pz.normalization_frequency,
        format_pz_type(&pz.pz_transfer_function_type),
        pz.zeros
            .iter()
            .map(|z| (z.real.to_bits(), z.imaginary.to_bits()))
            .collect::<Vec<_>>(),
        pz.poles
            .iter()
            .map(|p| (p.real.to_bits(), p.imaginary.to_bits()))
            .collect::<Vec<_>>(),
    );

    if let Some(existing) = defs.paz_map.get(&key) {
        return existing.clone();
    }

    let id = defs.next_id();
    let public_id = format!("ResponsePAZ/{id}");

    let zeros_str = if pz.zeros.is_empty() {
        None
    } else {
        Some(format_complex_array(&pz.zeros))
    };
    let poles_str = if pz.poles.is_empty() {
        None
    } else {
        Some(format_complex_array(&pz.poles))
    };

    defs.response_paz.push(Sc3mlResponsePaz {
        public_id: public_id.clone(),
        name: None,
        paz_type: Some(format_pz_type(&pz.pz_transfer_function_type)),
        gain: stage.stage_gain.as_ref().map(|g| g.value),
        gain_frequency: stage.stage_gain.as_ref().map(|g| g.frequency),
        normalization_factor: Some(pz.normalization_factor),
        normalization_frequency: Some(pz.normalization_frequency),
        number_of_zeros: Some(pz.zeros.len() as u32),
        number_of_poles: Some(pz.poles.len() as u32),
        zeros: zeros_str,
        poles: poles_str,
        remark: None,
    });

    defs.paz_map.insert(key, public_id.clone());
    public_id
}

fn get_or_create_fir(fir: &FIR, stage: &ResponseStage, defs: &mut Definitions) -> String {
    // Build dedup key from symmetry + coefficients
    let key = format!(
        "{}|{:?}",
        format_symmetry(&fir.symmetry),
        fir.numerator_coefficients
            .iter()
            .map(|c| c.to_bits())
            .collect::<Vec<_>>(),
    );

    if let Some(existing) = defs.fir_map.get(&key) {
        return existing.clone();
    }

    let id = defs.next_id();
    let public_id = format!("ResponseFIR/{id}");

    let coefficients = if fir.numerator_coefficients.is_empty() {
        None
    } else {
        Some(format_float_array(&fir.numerator_coefficients))
    };

    defs.response_fir.push(Sc3mlResponseFir {
        public_id: public_id.clone(),
        name: None,
        gain: stage.stage_gain.as_ref().map(|g| g.value),
        gain_frequency: stage.stage_gain.as_ref().map(|g| g.frequency),
        decimation_factor: stage.decimation.as_ref().map(|d| d.factor),
        delay: stage.decimation.as_ref().map(|d| d.delay),
        correction: stage.decimation.as_ref().map(|d| d.correction),
        number_of_coefficients: Some(fir.numerator_coefficients.len() as u32),
        symmetry: Some(format_symmetry(&fir.symmetry)),
        coefficients,
        remark: None,
    });

    defs.fir_map.insert(key, public_id.clone());
    public_id
}

// ─── Formatting helpers ──────────────────────────────────────────────

fn format_pz_type(pz: &PzTransferFunction) -> String {
    match pz {
        PzTransferFunction::LaplaceRadians => "A".into(),
        PzTransferFunction::LaplaceHertz => "B".into(),
        PzTransferFunction::DigitalZTransform => "D".into(),
    }
}

fn format_symmetry(sym: &Symmetry) -> String {
    match sym {
        Symmetry::None => "A".into(),
        Symmetry::Odd => "B".into(),
        Symmetry::Even => "C".into(),
    }
}

/// Format complex numbers as SC3ML: `(real,imag) (real,imag)`
fn format_complex_array(pzs: &[PoleZero]) -> String {
    pzs.iter()
        .map(|pz| format!("({},{})", pz.real, pz.imaginary))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format float array as space-separated values.
fn format_float_array(values: &[f64]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert a float sample rate to numerator/denominator.
///
/// e.g., 100.0 → (100, 1), 0.1 → (1, 10), 40.0 → (40, 1)
fn float_to_fraction(rate: f64) -> (u32, u32) {
    if rate <= 0.0 {
        return (0, 1);
    }
    if rate >= 1.0 {
        // Common case: integer sample rates
        let rounded = rate.round() as u32;
        if (rate - rounded as f64).abs() < 1e-6 {
            return (rounded, 1);
        }
    }
    // For sub-hertz rates, find a reasonable fraction
    // Try denominators up to 1000
    for denom in 1..=1000u32 {
        let num = (rate * denom as f64).round() as u32;
        if ((num as f64 / denom as f64) - rate).abs() < 1e-6 {
            return (num, denom);
        }
    }
    // Fallback: use large denominator
    let denom = 1000u32;
    let num = (rate * denom as f64).round() as u32;
    (num, denom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_test_inventory() -> Inventory {
        Inventory {
            source: "Test".into(),
            sender: None,
            created: Some(Utc::now()),
            networks: vec![Network {
                code: "XX".into(),
                description: Some("Test Network".into()),
                start_date: None,
                end_date: None,
                stations: vec![Station {
                    code: "PBUMI".into(),
                    description: None,
                    latitude: -7.7714,
                    longitude: 110.3776,
                    elevation: 150.0,
                    site: Site {
                        name: "Yogyakarta".into(),
                        country: Some("Indonesia".into()),
                        ..Default::default()
                    },
                    start_date: None,
                    end_date: None,
                    creation_date: None,
                    channels: vec![
                        Channel {
                            code: "SHZ".into(),
                            location_code: "00".into(),
                            latitude: -7.7714,
                            longitude: 110.3776,
                            elevation: 150.0,
                            depth: 0.0,
                            azimuth: 0.0,
                            dip: -90.0,
                            sample_rate: 100.0,
                            start_date: None,
                            end_date: None,
                            sensor: Some(Equipment {
                                equipment_type: Some("Geophone".into()),
                                model: Some("GS-11D".into()),
                                manufacturer: Some("Geospace".into()),
                                serial_number: Some("1234".into()),
                                ..Default::default()
                            }),
                            data_logger: Some(Equipment {
                                equipment_type: Some("Datalogger".into()),
                                model: Some("PB-24".into()),
                                ..Default::default()
                            }),
                            response: Some(Response {
                                instrument_sensitivity: Some(InstrumentSensitivity {
                                    value: 53687084.8,
                                    frequency: 15.0,
                                    input_units: Units {
                                        name: "M/S".into(),
                                        description: None,
                                    },
                                    output_units: Units {
                                        name: "COUNTS".into(),
                                        description: None,
                                    },
                                }),
                                stages: vec![
                                    ResponseStage {
                                        number: 1,
                                        stage_gain: Some(StageGain {
                                            value: 32.0,
                                            frequency: 15.0,
                                        }),
                                        poles_zeros: Some(PolesZeros {
                                            input_units: Units {
                                                name: "M/S".into(),
                                                description: None,
                                            },
                                            output_units: Units {
                                                name: "V".into(),
                                                description: None,
                                            },
                                            pz_transfer_function_type:
                                                PzTransferFunction::LaplaceRadians,
                                            normalization_factor: 1.0,
                                            normalization_frequency: 15.0,
                                            zeros: vec![
                                                PoleZero {
                                                    number: 0,
                                                    real: 0.0,
                                                    imaginary: 0.0,
                                                },
                                                PoleZero {
                                                    number: 1,
                                                    real: 0.0,
                                                    imaginary: 0.0,
                                                },
                                            ],
                                            poles: vec![
                                                PoleZero {
                                                    number: 0,
                                                    real: -22.2111,
                                                    imaginary: 22.2111,
                                                },
                                                PoleZero {
                                                    number: 1,
                                                    real: -22.2111,
                                                    imaginary: -22.2111,
                                                },
                                            ],
                                        }),
                                        coefficients: None,
                                        fir: None,
                                        decimation: None,
                                    },
                                    ResponseStage {
                                        number: 2,
                                        stage_gain: Some(StageGain {
                                            value: 1677721.4,
                                            frequency: 15.0,
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
                                            input_sample_rate: 100.0,
                                            factor: 1,
                                            offset: 0,
                                            delay: 0.0,
                                            correction: 0.0,
                                        }),
                                    },
                                ],
                            }),
                        },
                        Channel {
                            code: "SHN".into(),
                            location_code: "00".into(),
                            latitude: -7.7714,
                            longitude: 110.3776,
                            elevation: 150.0,
                            depth: 0.0,
                            azimuth: 0.0,
                            dip: 0.0,
                            sample_rate: 100.0,
                            start_date: None,
                            end_date: None,
                            sensor: Some(Equipment {
                                equipment_type: Some("Geophone".into()),
                                model: Some("GS-11D".into()),
                                manufacturer: Some("Geospace".into()),
                                serial_number: Some("1235".into()),
                                ..Default::default()
                            }),
                            data_logger: None,
                            response: None,
                        },
                    ],
                }],
            }],
        }
    }

    #[test]
    fn write_produces_valid_xml() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains("seiscomp"));
        assert!(xml.contains("0.13"));
    }

    #[test]
    fn write_contains_sensor_definition() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        assert!(xml.contains("GS-11D"));
        assert!(xml.contains("Geospace"));
        assert!(xml.contains("Sensor/"));
    }

    #[test]
    fn write_contains_datalogger_definition() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        assert!(xml.contains("PB-24"));
        assert!(xml.contains("Datalogger/"));
    }

    #[test]
    fn write_contains_response_paz() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        assert!(xml.contains("responsePAZ"));
        assert!(xml.contains("ResponsePAZ/"));
    }

    #[test]
    fn write_contains_network_hierarchy() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        assert!(xml.contains(r#"code="XX""#));
        assert!(xml.contains(r#"code="PBUMI""#));
        assert!(xml.contains(r#"code="00""#));
        assert!(xml.contains(r#"code="SHZ""#));
        assert!(xml.contains(r#"code="SHN""#));
    }

    #[test]
    fn write_stream_has_gain() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        assert!(xml.contains("<gain>53687084.8</gain>"));
        assert!(xml.contains("<gainFrequency>15</gainFrequency>"));
        assert!(xml.contains("<gainUnit>M/S</gainUnit>"));
    }

    #[test]
    fn float_to_fraction_integer() {
        assert_eq!(float_to_fraction(100.0), (100, 1));
        assert_eq!(float_to_fraction(40.0), (40, 1));
        assert_eq!(float_to_fraction(1.0), (1, 1));
    }

    #[test]
    fn float_to_fraction_sub_hertz() {
        assert_eq!(float_to_fraction(0.1), (1, 10));
        assert_eq!(float_to_fraction(0.01), (1, 100));
    }

    #[test]
    fn float_to_fraction_zero() {
        assert_eq!(float_to_fraction(0.0), (0, 1));
    }

    #[test]
    fn format_complex_roundtrip() {
        let pzs = vec![
            PoleZero {
                number: 0,
                real: -0.037,
                imaginary: 0.037,
            },
            PoleZero {
                number: 1,
                real: -0.037,
                imaginary: -0.037,
            },
        ];
        let s = format_complex_array(&pzs);
        assert_eq!(s, "(-0.037,0.037) (-0.037,-0.037)");
    }

    #[test]
    fn format_float_array_basic() {
        let vals = vec![0.1, 0.2, 0.3];
        assert_eq!(format_float_array(&vals), "0.1 0.2 0.3");
    }

    #[test]
    fn sensor_dedup_works() {
        let inv = make_test_inventory();
        let xml = write_to_string(&inv).unwrap();
        // Two channels with same GS-11D sensor should produce only one sensor definition
        // (sensor dedup is by model+manufacturer)
        let sensor_count = xml.matches("<sensor ").count();
        // Should be 1 top-level sensor def + 1 in each stream attribute (doesn't count)
        assert_eq!(
            sensor_count, 1,
            "Expected 1 sensor definition, found {sensor_count} in: {xml}"
        );
    }
}

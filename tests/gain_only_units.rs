//! Units of gain-only response stages (preamp / PGA / A/D conversion factor).
//!
//! A stage that is nothing but a `<StageGain>` has nowhere to declare its units, which
//! breaks the input/output unit chain of the response. `evalresp` then refuses to
//! evaluate the *whole* channel ("units mismatch between stages"), so the response
//! cannot be used for deconvolution at all — the metadata looks fine but is worthless.
//!
//! These tests pin the fix: a gain-only stage carrying units is written as an empty
//! `<PolesZeros>` (no poles, no zeros, `A0 = 1`, so `H(s) = 1` exactly) that carries
//! `InputUnits`/`OutputUnits`, and reading it back restores the gain-only stage.
//!
//! The chain modelled here is a real digitizer:
//!
//! | stage | element      | units          |
//! |-------|--------------|----------------|
//! | 1     | poles & zeros| `M/S` -> `V`   |
//! | 2     | gain only    | `V` -> `V`     | PGA
//! | 3     | gain only    | `V` -> `COUNTS`| A/D
//! | 4, 5  | FIR          | `COUNTS` -> `COUNTS` |

use stationxml_rs::*;

/// Sensor poles & zeros stage: 4.5 Hz geophone, `M/S` -> `V`.
fn sensor_stage() -> ResponseStage {
    ResponseStage {
        number: 1,
        stage_gain: Some(StageGain {
            value: 1500.0,
            frequency: 1.0,
        }),
        poles_zeros: Some(PolesZeros {
            input_units: Units::with_description("M/S", "Velocity in meters per second"),
            output_units: Units::new("V"),
            pz_transfer_function_type: PzTransferFunction::LaplaceRadians,
            normalization_factor: 1.0,
            normalization_frequency: 1.0,
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
                    real: -19.79,
                    imaginary: 20.19,
                },
                PoleZero {
                    number: 1,
                    real: -19.79,
                    imaginary: -20.19,
                },
            ],
        }),
        ..Default::default()
    }
}

fn fir_stage(number: u32, input_rate: f64) -> ResponseStage {
    ResponseStage {
        number,
        stage_gain: Some(StageGain {
            value: 1.0,
            frequency: 1.0,
        }),
        fir: Some(FIR {
            input_units: Units::new("COUNTS"),
            output_units: Units::new("COUNTS"),
            symmetry: Symmetry::Odd,
            numerator_coefficients: vec![0.25, 0.5, 0.25],
        }),
        decimation: Some(Decimation {
            input_sample_rate: input_rate,
            factor: 4,
            offset: 0,
            delay: 0.001,
            correction: 0.001,
        }),
        ..Default::default()
    }
}

/// The five-stage chain a seismic digitizer emits: sensor, PGA, A/D, two FIRs.
fn digitizer_response() -> Response {
    Response {
        instrument_sensitivity: Some(InstrumentSensitivity {
            value: 1500.0 * 8.0 * 3_355_442.8,
            frequency: 1.0,
            input_units: Units::new("M/S"),
            output_units: Units::new("COUNTS"),
        }),
        stages: vec![
            sensor_stage(),
            // PGA: analog voltage gain, no filtering.
            ResponseStage::gain_only(2, 8.0, 1.0, Units::new("V"), Units::new("V")),
            // A/D: counts per volt.
            ResponseStage::gain_only(
                3,
                3_355_442.8,
                1.0,
                Units::new("V"),
                Units::with_description("COUNTS", "Digital counts"),
            ),
            fir_stage(4, 3200.0),
            fir_stage(5, 800.0),
        ],
    }
}

fn digitizer_inventory() -> Inventory {
    Inventory::builder()
        .source("Pena Bumi")
        .network("XX", |n| {
            n.station("PBUMI", |s| {
                s.latitude(-7.7714)
                    .longitude(110.3776)
                    .elevation(150.0)
                    .site_name("Bench")
                    .channel("HHZ", "00", |c| {
                        c.sample_rate(200.0)
                            .dip(-90.0)
                            .data_logger(Equipment {
                                model: Some("PB-24".into()),
                                ..Default::default()
                            })
                            .response(digitizer_response())
                    })
            })
        })
        .build()
}

#[test]
fn gain_only_stage_declares_units_in_xml() {
    let xml = write_to_string::<Fdsn>(&digitizer_inventory()).unwrap();

    // Every stage must declare units — this is exactly what evalresp walks.
    let stage_count = xml.matches("<Stage number=").count();
    assert_eq!(stage_count, 5, "expected 5 stages, got {stage_count}");
    assert_eq!(
        xml.matches("<InputUnits>").count(),
        // 5 stages + the overall InstrumentSensitivity.
        6,
        "every stage plus the sensitivity must declare InputUnits:\n{xml}"
    );
    assert_eq!(xml.matches("<OutputUnits>").count(), 6);

    // No stage may consist of a bare <StageGain> with no units around it.
    assert!(
        !xml.contains("<Stage number=\"2\"><StageGain>"),
        "stage 2 was written as a bare gain, units are lost:\n{xml}"
    );
    assert!(!xml.contains("<Stage number=\"3\"><StageGain>"));
}

#[test]
fn gain_only_stage_is_written_as_inert_poles_zeros() {
    let xml = write_to_string::<Fdsn>(&digitizer_inventory()).unwrap();

    // The unit carrier must be transparent: A0 = 1 and no poles or zeros, so it
    // contributes exactly its scalar gain and nothing else.
    let stage3 = xml
        .split("<Stage number=\"3\">")
        .nth(1)
        .and_then(|s| s.split("</Stage>").next())
        .expect("stage 3 present");
    assert!(stage3.contains("<PolesZeros>"), "{stage3}");
    assert!(stage3.contains("<Name>V</Name>"), "{stage3}");
    assert!(stage3.contains("<Name>COUNTS</Name>"), "{stage3}");
    assert!(stage3.contains("<NormalizationFactor>1</NormalizationFactor>"));
    assert!(!stage3.contains("<Pole "), "carrier must have no poles");
    assert!(!stage3.contains("<Zero "), "carrier must have no zeros");
    assert!(
        stage3.contains("LAPLACE (RADIANS/SECOND)"),
        "evalresp needs a transfer function type it understands: {stage3}"
    );
}

#[test]
fn unit_chain_is_continuous_across_all_stages() {
    let xml = write_to_string::<Fdsn>(&digitizer_inventory()).unwrap();
    let inv = read_from_str(&xml).unwrap();
    let resp = inv.networks[0].stations[0].channels[0]
        .response
        .as_ref()
        .unwrap();

    let units: Vec<(String, String)> = resp
        .stages
        .iter()
        .map(|s| {
            let (i, o) = s
                .resolved_units()
                .unwrap_or_else(|| panic!("stage {} declares no units", s.number));
            (i.name.clone(), o.name.clone())
        })
        .collect();

    assert_eq!(
        units,
        vec![
            ("M/S".to_string(), "V".to_string()),
            ("V".to_string(), "V".to_string()),
            ("V".to_string(), "COUNTS".to_string()),
            ("COUNTS".to_string(), "COUNTS".to_string()),
            ("COUNTS".to_string(), "COUNTS".to_string()),
        ]
    );

    // The property evalresp actually checks: output of stage N == input of stage N+1.
    for pair in units.windows(2) {
        assert_eq!(
            pair[0].1, pair[1].0,
            "unit chain broken between stages: {pair:?}"
        );
    }
}

#[test]
fn roundtrip_restores_gain_only_stage() {
    let inv = digitizer_inventory();
    let xml = write_to_string::<Fdsn>(&inv).unwrap();
    let back = read_from_str(&xml).unwrap();

    // (`created` is stamped by the writer, so compare the part under test.)
    assert_eq!(
        back.networks[0].stations[0].channels[0].response,
        inv.networks[0].stations[0].channels[0].response,
        "read(write(response)) must equal response"
    );

    let stages = &back.networks[0].stations[0].channels[0]
        .response
        .as_ref()
        .unwrap()
        .stages;

    // The carrier is lifted back onto the stage — it is a gain, not a filter.
    let pga = &stages[1];
    assert!(
        pga.poles_zeros.is_none(),
        "carrier must not survive as a filter"
    );
    assert_eq!(pga.input_units.as_ref().unwrap().name, "V");
    assert_eq!(pga.output_units.as_ref().unwrap().name, "V");
    assert_eq!(pga.stage_gain.as_ref().unwrap().value, 8.0);

    let adc = &stages[2];
    assert_eq!(adc.output_units.as_ref().unwrap().name, "COUNTS");
    assert_eq!(
        adc.output_units.as_ref().unwrap().description.as_deref(),
        Some("Digital counts"),
        "unit descriptions must survive the roundtrip"
    );

    // Writing again is stable.
    assert_eq!(write_to_string::<Fdsn>(&back).unwrap(), xml);
}

#[test]
fn stage_with_real_filter_keeps_element_units() {
    // A stage with a genuine transfer function keeps its units in the element; the
    // stage-level fields stay empty so there is exactly one source of truth.
    let inv = digitizer_inventory();
    let back = read_from_str(&write_to_string::<Fdsn>(&inv).unwrap()).unwrap();
    let stages = &back.networks[0].stations[0].channels[0]
        .response
        .as_ref()
        .unwrap()
        .stages;

    let sensor = &stages[0];
    assert!(sensor.input_units.is_none());
    assert!(sensor.output_units.is_none());
    assert_eq!(sensor.resolved_units().unwrap().0.name, "M/S");

    let fir = &stages[3];
    assert!(fir.input_units.is_none());
    assert_eq!(fir.resolved_units().unwrap().1.name, "COUNTS");
}

#[test]
fn poles_zeros_with_gain_only_a0_is_not_lifted() {
    // An empty poles & zeros with A0 != 1 is a real (flat) filter, not a unit carrier:
    // lifting it would silently drop the A0 factor.
    let mut resp = digitizer_response();
    resp.stages[1] = ResponseStage {
        number: 2,
        stage_gain: Some(StageGain {
            value: 8.0,
            frequency: 1.0,
        }),
        poles_zeros: Some(PolesZeros {
            input_units: Units::new("V"),
            output_units: Units::new("V"),
            pz_transfer_function_type: PzTransferFunction::LaplaceRadians,
            normalization_factor: 2.5,
            normalization_frequency: 1.0,
            zeros: vec![],
            poles: vec![],
        }),
        ..Default::default()
    };
    let inv = Inventory::builder()
        .source("Pena Bumi")
        .network("XX", |n| {
            n.station("PBUMI", |s| {
                s.site_name("Bench")
                    .channel("HHZ", "00", |c| c.sample_rate(200.0).response(resp.clone()))
            })
        })
        .build();

    let back = read_from_str(&write_to_string::<Fdsn>(&inv).unwrap()).unwrap();
    let stage = &back.networks[0].stations[0].channels[0]
        .response
        .as_ref()
        .unwrap()
        .stages[1];

    let pz = stage
        .poles_zeros
        .as_ref()
        .expect("A0 != 1 must stay a poles & zeros stage");
    assert_eq!(pz.normalization_factor, 2.5);
    assert!(stage.input_units.is_none());
}

#[test]
fn legacy_bare_gain_stage_still_reads() {
    // Files written before this fix have gain-only stages with no units at all.
    // They must keep parsing — just without units to resolve.
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" schemaVersion="1.2">
  <Source>Pena Bumi</Source>
  <Created>2026-01-01T00:00:00Z</Created>
  <Network code="XX">
    <Station code="PBUMI">
      <Latitude>0</Latitude><Longitude>0</Longitude><Elevation>0</Elevation>
      <Site><Name>Bench</Name></Site>
      <Channel code="HHZ" locationCode="00">
        <Latitude>0</Latitude><Longitude>0</Longitude>
        <Elevation>0</Elevation><Depth>0</Depth>
        <Azimuth>0</Azimuth><Dip>-90</Dip>
        <SampleRate>200</SampleRate>
        <Response>
          <Stage number="2">
            <StageGain><Value>8</Value><Frequency>1</Frequency></StageGain>
          </Stage>
        </Response>
      </Channel>
    </Station>
  </Network>
</FDSNStationXML>"#;

    let inv = read_from_str(xml).unwrap();
    let stage = &inv.networks[0].stations[0].channels[0]
        .response
        .as_ref()
        .unwrap()
        .stages[0];
    assert_eq!(stage.stage_gain.as_ref().unwrap().value, 8.0);
    assert!(stage.resolved_units().is_none());

    // And writing it back does not invent units.
    let xml_out = write_to_string::<Fdsn>(&inv).unwrap();
    assert!(!xml_out.contains("<PolesZeros>"));
}

#[test]
fn sc3ml_keeps_gain_only_datalogger_gain() {
    // SC3ML stores the A/D factor as the datalogger gain. It used to be found by
    // looking for a `Coefficients` element, so a gain-only A/D stage was dropped and
    // the exported datalogger had no gain at all.
    let xml = write_to_string::<Sc3ml>(&digitizer_inventory()).unwrap();
    assert!(
        xml.contains("3355442.8"),
        "datalogger gain missing from SC3ML:\n{xml}"
    );
}

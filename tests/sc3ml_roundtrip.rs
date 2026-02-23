//! Integration tests for SC3ML read/write and cross-format conversion.

use stationxml_rs::*;

const SC3ML_SAMPLE: &str = include_str!("fixtures/sc3ml_sample.xml");
const FDSN_SAMPLE: &str = include_str!("fixtures/fdsn_sample.xml");

// ─── Read tests ──────────────────────────────────────────────────────

#[test]
fn read_sc3ml_sample() {
    let inv = read_from_str(SC3ML_SAMPLE).unwrap();

    // Network
    assert_eq!(inv.networks.len(), 1);
    let net = &inv.networks[0];
    assert_eq!(net.code, "XX");
    assert_eq!(net.description.as_deref(), Some("Local Test Network"));
    assert!(net.start_date.is_some());

    // Station
    assert_eq!(net.stations.len(), 1);
    let sta = &net.stations[0];
    assert_eq!(sta.code, "PBUMI");
    assert!((sta.latitude - (-7.7714)).abs() < 1e-6);
    assert!((sta.longitude - 110.3776).abs() < 1e-6);
    assert!((sta.elevation - 150.0).abs() < 1e-6);
    assert_eq!(sta.site.country.as_deref(), Some("Indonesia"));

    // Channels
    assert_eq!(sta.channels.len(), 3);

    // SHZ channel
    let shz = &sta.channels[0];
    assert_eq!(shz.code, "SHZ");
    assert_eq!(shz.location_code, "00");
    assert!((shz.dip - (-90.0)).abs() < 1e-6);
    assert!((shz.sample_rate - 100.0).abs() < 1e-6);
    assert!((shz.depth - 0.0).abs() < 1e-6);

    // SHZ sensor
    let sensor = shz.sensor.as_ref().unwrap();
    assert_eq!(sensor.model.as_deref(), Some("GS-11D"));
    assert_eq!(sensor.manufacturer.as_deref(), Some("Geospace"));
    assert_eq!(sensor.serial_number.as_deref(), Some("1234"));

    // SHZ datalogger
    let dl = shz.data_logger.as_ref().unwrap();
    assert_eq!(dl.serial_number.as_deref(), Some("PB001"));

    // SHZ response
    let resp = shz.response.as_ref().unwrap();
    let sens = resp.instrument_sensitivity.as_ref().unwrap();
    assert!((sens.value - 53687084.8).abs() < 0.1);
    assert!((sens.frequency - 15.0).abs() < 1e-6);
    assert_eq!(sens.input_units.name, "M/S");
    assert_eq!(sens.output_units.name, "COUNTS");

    // SHZ response stages
    assert!(
        resp.stages.len() >= 2,
        "Expected at least 2 stages, got {}",
        resp.stages.len()
    );

    // Stage 1: PAZ (sensor)
    let s1 = &resp.stages[0];
    assert_eq!(s1.number, 1);
    let pz = s1.poles_zeros.as_ref().unwrap();
    assert_eq!(
        pz.pz_transfer_function_type,
        PzTransferFunction::LaplaceRadians
    );
    assert_eq!(pz.zeros.len(), 2);
    assert_eq!(pz.poles.len(), 2);
    assert!((pz.poles[0].real - (-22.2111)).abs() < 1e-4);
    assert!((pz.poles[0].imaginary - 22.2111).abs() < 1e-4);
    assert!((s1.stage_gain.as_ref().unwrap().value - 32.0).abs() < 1e-6);

    // Stage 2: Datalogger gain (V → COUNTS)
    let s2 = &resp.stages[1];
    assert_eq!(s2.number, 2);
    let cf = s2.coefficients.as_ref().unwrap();
    assert_eq!(cf.output_units.name, "COUNTS");
    assert!((s2.stage_gain.as_ref().unwrap().value - 1677721.4).abs() < 0.1);

    // SHN channel
    let shn = &sta.channels[1];
    assert_eq!(shn.code, "SHN");
    assert!((shn.dip - 0.0).abs() < 1e-6);
    assert!((shn.azimuth - 0.0).abs() < 1e-6);

    // SHE channel
    let she = &sta.channels[2];
    assert_eq!(she.code, "SHE");
    assert!((she.azimuth - 90.0).abs() < 1e-6);
}

#[test]
fn read_sc3ml_fir_stage() {
    let inv = read_from_str(SC3ML_SAMPLE).unwrap();
    let shz = &inv.networks[0].stations[0].channels[0];
    let resp = shz.response.as_ref().unwrap();

    // Find the FIR stage
    let fir_stage = resp.stages.iter().find(|s| s.fir.is_some());
    assert!(fir_stage.is_some(), "Expected a FIR stage in SHZ response");

    let fir = fir_stage.unwrap().fir.as_ref().unwrap();
    assert_eq!(fir.symmetry, Symmetry::Even);
    assert_eq!(fir.numerator_coefficients.len(), 5);
    assert!((fir.numerator_coefficients[2] - 0.4).abs() < 1e-6);

    let dec = fir_stage.unwrap().decimation.as_ref().unwrap();
    assert_eq!(dec.factor, 2);
}

// ─── Roundtrip tests ─────────────────────────────────────────────────

#[test]
fn roundtrip_sc3ml() {
    // Read SC3ML
    let inv1 = read_from_str(SC3ML_SAMPLE).unwrap();

    // Write as SC3ML
    let xml_out = write_to_string::<Sc3ml>(&inv1).unwrap();

    // Read back
    let inv2 = Sc3ml::read_from_str(&xml_out).unwrap();

    // Compare core structure
    assert_eq!(inv1.networks.len(), inv2.networks.len());
    let net1 = &inv1.networks[0];
    let net2 = &inv2.networks[0];
    assert_eq!(net1.code, net2.code);
    assert_eq!(net1.description, net2.description);

    let sta1 = &net1.stations[0];
    let sta2 = &net2.stations[0];
    assert_eq!(sta1.code, sta2.code);
    assert!((sta1.latitude - sta2.latitude).abs() < 1e-6);
    assert!((sta1.longitude - sta2.longitude).abs() < 1e-6);
    assert!((sta1.elevation - sta2.elevation).abs() < 1e-6);

    // Compare channels
    assert_eq!(sta1.channels.len(), sta2.channels.len());
    for (ch1, ch2) in sta1.channels.iter().zip(sta2.channels.iter()) {
        assert_eq!(ch1.code, ch2.code);
        assert_eq!(ch1.location_code, ch2.location_code);
        assert!((ch1.latitude - ch2.latitude).abs() < 1e-6);
        assert!((ch1.longitude - ch2.longitude).abs() < 1e-6);
        assert!((ch1.dip - ch2.dip).abs() < 1e-6);
        assert!((ch1.azimuth - ch2.azimuth).abs() < 1e-6);
        assert!((ch1.sample_rate - ch2.sample_rate).abs() < 1e-6);
        assert!((ch1.depth - ch2.depth).abs() < 1e-6);
    }

    // Compare sensitivity for SHZ
    let sens1 = sta1.channels[0]
        .response
        .as_ref()
        .unwrap()
        .instrument_sensitivity
        .as_ref()
        .unwrap();
    let sens2 = sta2.channels[0]
        .response
        .as_ref()
        .unwrap()
        .instrument_sensitivity
        .as_ref()
        .unwrap();
    assert!((sens1.value - sens2.value).abs() < 0.1);
    assert!((sens1.frequency - sens2.frequency).abs() < 1e-6);
    assert_eq!(sens1.input_units.name, sens2.input_units.name);
}

// ─── Cross-format tests ─────────────────────────────────────────────

#[test]
fn cross_format_fdsn_to_sc3ml() {
    // Read FDSN fixture
    let inv_fdsn = read_from_str(FDSN_SAMPLE).unwrap();

    // Write as SC3ML
    let sc3ml_xml = write_to_string::<Sc3ml>(&inv_fdsn).unwrap();

    // Read back as SC3ML
    let inv_sc3ml = Sc3ml::read_from_str(&sc3ml_xml).unwrap();

    // Compare core fields
    assert_eq!(inv_fdsn.networks[0].code, inv_sc3ml.networks[0].code);

    let sta_fdsn = &inv_fdsn.networks[0].stations[0];
    let sta_sc3ml = &inv_sc3ml.networks[0].stations[0];
    assert_eq!(sta_fdsn.code, sta_sc3ml.code);
    assert!((sta_fdsn.latitude - sta_sc3ml.latitude).abs() < 1e-6);
    assert!((sta_fdsn.longitude - sta_sc3ml.longitude).abs() < 1e-6);
    assert!((sta_fdsn.elevation - sta_sc3ml.elevation).abs() < 1e-6);

    // Channel count preserved
    assert_eq!(sta_fdsn.channels.len(), sta_sc3ml.channels.len());

    // Channel codes preserved
    for (ch_f, ch_s) in sta_fdsn.channels.iter().zip(sta_sc3ml.channels.iter()) {
        assert_eq!(ch_f.code, ch_s.code);
        assert_eq!(ch_f.location_code, ch_s.location_code);
        assert!((ch_f.sample_rate - ch_s.sample_rate).abs() < 1e-6);
        assert!((ch_f.dip - ch_s.dip).abs() < 1e-6);
        assert!((ch_f.azimuth - ch_s.azimuth).abs() < 1e-6);
    }

    // Sensitivity preserved for SHZ
    let sens_f = sta_fdsn.channels[0]
        .response
        .as_ref()
        .unwrap()
        .instrument_sensitivity
        .as_ref()
        .unwrap();
    let sens_s = sta_sc3ml.channels[0]
        .response
        .as_ref()
        .unwrap()
        .instrument_sensitivity
        .as_ref()
        .unwrap();
    assert!((sens_f.value - sens_s.value).abs() < 0.1);
}

#[test]
fn cross_format_sc3ml_to_fdsn() {
    // Read SC3ML fixture
    let inv_sc3ml = read_from_str(SC3ML_SAMPLE).unwrap();

    // Write as FDSN
    let fdsn_xml = write_to_string::<Fdsn>(&inv_sc3ml).unwrap();

    // Read back as FDSN
    let inv_fdsn = Fdsn::read_from_str(&fdsn_xml).unwrap();

    // Compare core fields
    assert_eq!(inv_sc3ml.networks[0].code, inv_fdsn.networks[0].code);

    let sta_s = &inv_sc3ml.networks[0].stations[0];
    let sta_f = &inv_fdsn.networks[0].stations[0];
    assert_eq!(sta_s.code, sta_f.code);
    assert!((sta_s.latitude - sta_f.latitude).abs() < 1e-6);
    assert!((sta_s.longitude - sta_f.longitude).abs() < 1e-6);

    // Channel count preserved
    assert_eq!(sta_s.channels.len(), sta_f.channels.len());

    // Sensitivity preserved
    let sens_s = sta_s.channels[0]
        .response
        .as_ref()
        .unwrap()
        .instrument_sensitivity
        .as_ref()
        .unwrap();
    let sens_f = sta_f.channels[0]
        .response
        .as_ref()
        .unwrap()
        .instrument_sensitivity
        .as_ref()
        .unwrap();
    assert!((sens_s.value - sens_f.value).abs() < 0.1);
    assert_eq!(sens_s.input_units.name, sens_f.input_units.name);
}

// ─── Auto-detect ─────────────────────────────────────────────────────

#[test]
fn auto_detect_sc3ml() {
    let format = detect_format(SC3ML_SAMPLE);
    assert_eq!(format, Some(Format::Sc3ml));
}

#[test]
fn auto_detect_reads_sc3ml() {
    // Auto-detect should work for SC3ML via read_from_str
    let inv = read_from_str(SC3ML_SAMPLE).unwrap();
    assert_eq!(inv.networks[0].code, "XX");
}

// ─── File I/O ────────────────────────────────────────────────────────

#[test]
fn read_sc3ml_from_file() {
    let inv = read_from_file("tests/fixtures/sc3ml_sample.xml").unwrap();
    assert_eq!(inv.networks[0].code, "XX");
    assert_eq!(inv.networks[0].stations[0].channels.len(), 3);
}

#[test]
fn read_sc3ml_from_file_as() {
    let inv = read_from_file_as::<Sc3ml>("tests/fixtures/sc3ml_sample.xml").unwrap();
    assert_eq!(inv.networks[0].code, "XX");
}

// ─── Write verification ─────────────────────────────────────────────

#[test]
fn write_sc3ml_contains_declaration() {
    let inv = read_from_str(SC3ML_SAMPLE).unwrap();
    let xml_out = write_to_string::<Sc3ml>(&inv).unwrap();
    assert!(xml_out.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
}

#[test]
fn write_sc3ml_contains_namespace() {
    let inv = read_from_str(SC3ML_SAMPLE).unwrap();
    let xml_out = write_to_string::<Sc3ml>(&inv).unwrap();
    assert!(xml_out.contains("http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13"));
    assert!(xml_out.contains(r#"version="0.13""#));
}

#[test]
fn write_sc3ml_has_top_level_definitions() {
    let inv = read_from_str(SC3ML_SAMPLE).unwrap();
    let xml_out = write_to_string::<Sc3ml>(&inv).unwrap();
    // Should have sensor, datalogger, responsePAZ definitions
    assert!(xml_out.contains("<sensor "), "Missing sensor definitions");
    assert!(
        xml_out.contains("<datalogger "),
        "Missing datalogger definitions"
    );
    assert!(
        xml_out.contains("<responsePAZ "),
        "Missing responsePAZ definitions"
    );
}

// ─── Namespace version tolerance ─────────────────────────────────────

#[test]
fn read_sc3ml_version_09() {
    let xml = SC3ML_SAMPLE
        .replace(
            "http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.13",
            "http://geofon.gfz-potsdam.de/ns/seiscomp3-schema/0.9",
        )
        .replace(r#"version="0.13""#, r#"version="0.9""#);
    let inv = read_from_str(&xml).unwrap();
    assert_eq!(inv.networks[0].code, "XX");
}

//! Integration tests for FDSN StationXML read/write roundtrip.

use stationxml_rs::*;

const SAMPLE_XML: &str = include_str!("fixtures/fdsn_sample.xml");

#[test]
fn read_fdsn_sample() {
    let inv = read_from_str(SAMPLE_XML).unwrap();

    assert_eq!(inv.source, "Pena Bumi");
    assert_eq!(inv.sender.as_deref(), Some("stationxml-rs"));
    assert!(inv.created.is_some());

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
    assert_eq!(sta.site.name, "Yogyakarta Seismic Shelter");
    assert_eq!(sta.site.country.as_deref(), Some("Indonesia"));

    // Channels
    assert_eq!(sta.channels.len(), 3);

    // SHZ channel — has full response
    let shz = &sta.channels[0];
    assert_eq!(shz.code, "SHZ");
    assert_eq!(shz.location_code, "00");
    assert!((shz.dip - (-90.0)).abs() < 1e-6);
    assert!((shz.sample_rate - 100.0).abs() < 1e-6);

    // Sensor equipment
    let sensor = shz.sensor.as_ref().unwrap();
    assert_eq!(sensor.equipment_type.as_deref(), Some("Geophone"));
    assert_eq!(sensor.manufacturer.as_deref(), Some("Geospace"));
    assert_eq!(sensor.model.as_deref(), Some("GS-11D"));
    assert_eq!(sensor.serial_number.as_deref(), Some("1234"));

    // Data logger
    let dl = shz.data_logger.as_ref().unwrap();
    assert_eq!(dl.model.as_deref(), Some("PB-24"));

    // Response
    let resp = shz.response.as_ref().unwrap();
    let sens = resp.instrument_sensitivity.as_ref().unwrap();
    assert!((sens.value - 53687084.8).abs() < 0.1);
    assert!((sens.frequency - 15.0).abs() < 1e-6);
    assert_eq!(sens.input_units.name, "M/S");
    assert_eq!(sens.output_units.name, "COUNTS");

    // Stage 1 — Poles & Zeros
    assert_eq!(resp.stages.len(), 2);
    let stage1 = &resp.stages[0];
    assert_eq!(stage1.number, 1);
    let pz = stage1.poles_zeros.as_ref().unwrap();
    assert_eq!(pz.input_units.name, "M/S");
    assert_eq!(pz.output_units.name, "V");
    assert_eq!(pz.zeros.len(), 2);
    assert_eq!(pz.poles.len(), 2);
    assert!((pz.poles[0].real - (-22.2111)).abs() < 1e-4);
    assert!((pz.poles[0].imaginary - 22.2111).abs() < 1e-4);
    let gain1 = stage1.stage_gain.as_ref().unwrap();
    assert!((gain1.value - 32.0).abs() < 1e-6);

    // Stage 2 — Coefficients + Decimation
    let stage2 = &resp.stages[1];
    assert_eq!(stage2.number, 2);
    let cf = stage2.coefficients.as_ref().unwrap();
    assert_eq!(cf.input_units.name, "V");
    assert_eq!(cf.output_units.name, "COUNTS");
    assert_eq!(cf.numerators.len(), 1);
    let dec = stage2.decimation.as_ref().unwrap();
    assert!((dec.input_sample_rate - 100.0).abs() < 1e-6);
    assert_eq!(dec.factor, 1);

    // SHN — no response, different dip
    let shn = &sta.channels[1];
    assert_eq!(shn.code, "SHN");
    assert!((shn.dip - 0.0).abs() < 1e-6);
    assert!((shn.azimuth - 0.0).abs() < 1e-6);

    // SHE — azimuth 90
    let she = &sta.channels[2];
    assert_eq!(she.code, "SHE");
    assert!((she.azimuth - 90.0).abs() < 1e-6);
}

#[test]
fn roundtrip_fdsn() {
    // Read original
    let inv1 = read_from_str(SAMPLE_XML).unwrap();

    // Write back to XML
    let xml_out = write_to_string::<Fdsn>(&inv1).unwrap();

    // Read written XML
    let inv2 = read_from_str(&xml_out).unwrap();

    // Compare inventories — should be identical
    assert_eq!(inv1.source, inv2.source);
    assert_eq!(inv1.sender, inv2.sender);
    assert_eq!(inv1.networks.len(), inv2.networks.len());

    let net1 = &inv1.networks[0];
    let net2 = &inv2.networks[0];
    assert_eq!(net1.code, net2.code);
    assert_eq!(net1.description, net2.description);

    let sta1 = &net1.stations[0];
    let sta2 = &net2.stations[0];
    assert_eq!(sta1.code, sta2.code);
    assert!((sta1.latitude - sta2.latitude).abs() < 1e-10);
    assert!((sta1.longitude - sta2.longitude).abs() < 1e-10);
    assert!((sta1.elevation - sta2.elevation).abs() < 1e-10);
    assert_eq!(sta1.site, sta2.site);

    // Channels
    assert_eq!(sta1.channels.len(), sta2.channels.len());
    for (ch1, ch2) in sta1.channels.iter().zip(sta2.channels.iter()) {
        assert_eq!(ch1.code, ch2.code);
        assert_eq!(ch1.location_code, ch2.location_code);
        assert!((ch1.latitude - ch2.latitude).abs() < 1e-10);
        assert!((ch1.longitude - ch2.longitude).abs() < 1e-10);
        assert!((ch1.dip - ch2.dip).abs() < 1e-10);
        assert!((ch1.azimuth - ch2.azimuth).abs() < 1e-10);
        assert!((ch1.sample_rate - ch2.sample_rate).abs() < 1e-10);
        assert_eq!(ch1.sensor, ch2.sensor);
        assert_eq!(ch1.data_logger, ch2.data_logger);
    }

    // Full response roundtrip for SHZ
    let resp1 = sta1.channels[0].response.as_ref().unwrap();
    let resp2 = sta2.channels[0].response.as_ref().unwrap();
    assert_eq!(resp1.stages.len(), resp2.stages.len());
    assert_eq!(resp1.instrument_sensitivity, resp2.instrument_sensitivity);

    // Stage-level comparison
    for (s1, s2) in resp1.stages.iter().zip(resp2.stages.iter()) {
        assert_eq!(s1.number, s2.number);
        assert_eq!(s1.stage_gain, s2.stage_gain);
        assert_eq!(s1.poles_zeros, s2.poles_zeros);
        assert_eq!(s1.coefficients, s2.coefficients);
        assert_eq!(s1.decimation, s2.decimation);
    }
}

#[test]
fn roundtrip_preserves_equality() {
    // Stronger test: full PartialEq on Inventory
    let inv1 = read_from_str(SAMPLE_XML).unwrap();
    let xml_out = write_to_string::<Fdsn>(&inv1).unwrap();
    let inv2 = read_from_str(&xml_out).unwrap();

    // This uses the derived PartialEq — most thorough check
    assert_eq!(inv1, inv2);
}

#[test]
fn auto_detect_fdsn() {
    let format = detect_format(SAMPLE_XML);
    assert_eq!(format, Some(Format::Fdsn));
}

#[test]
fn write_contains_xml_declaration() {
    let inv = read_from_str(SAMPLE_XML).unwrap();
    let xml_out = write_to_string::<Fdsn>(&inv).unwrap();
    assert!(xml_out.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
}

#[test]
fn write_contains_namespace() {
    let inv = read_from_str(SAMPLE_XML).unwrap();
    let xml_out = write_to_string::<Fdsn>(&inv).unwrap();
    assert!(xml_out.contains(r#"xmlns="http://www.fdsn.org/xml/station/1""#));
    assert!(xml_out.contains(r#"schemaVersion="1.2""#));
}

#[test]
fn read_from_file_works() {
    let inv = read_from_file("tests/fixtures/fdsn_sample.xml").unwrap();
    assert_eq!(inv.source, "Pena Bumi");
    assert_eq!(inv.networks[0].stations[0].channels.len(), 3);
}

#[test]
fn read_from_file_as_fdsn() {
    let inv = read_from_file_as::<Fdsn>("tests/fixtures/fdsn_sample.xml").unwrap();
    assert_eq!(inv.source, "Pena Bumi");
}

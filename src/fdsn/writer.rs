//! FDSN StationXML writer: Inventory → fdsn types → XML.

use chrono::{SecondsFormat, Utc};

use crate::datetime::{format_datetime, format_datetime_opt};
use crate::error::Result;
use crate::inventory::*;

use super::types::*;

/// Serialize an [`Inventory`] to an FDSN StationXML string.
pub(crate) fn write_to_string(inventory: &Inventory) -> Result<String> {
    let fdsn = inventory_to_fdsn(inventory);
    let body = quick_xml::se::to_string(&fdsn)?;
    let mut xml = String::with_capacity(body.len() + 50);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(&body);
    Ok(xml)
}

// ─── Conversion functions ───────────────────────────────────────────

fn inventory_to_fdsn(inv: &Inventory) -> FdsnStationXml {
    FdsnStationXml {
        xmlns: Some("http://www.fdsn.org/xml/station/1".into()),
        schema_version: "1.2".into(),
        source: inv.source.clone(),
        sender: inv.sender.clone(),
        module: None,
        module_uri: None,
        created: inv
            .created
            .map(|dt| format_datetime(&dt))
            .unwrap_or_else(|| Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        networks: inv.networks.iter().map(convert_network).collect(),
    }
}

fn convert_network(net: &Network) -> FdsnNetwork {
    FdsnNetwork {
        code: net.code.clone(),
        start_date: format_datetime_opt(&net.start_date),
        end_date: format_datetime_opt(&net.end_date),
        restricted_status: None,
        description: net.description.clone(),
        total_number_stations: None,
        selected_number_stations: None,
        stations: net.stations.iter().map(convert_station).collect(),
    }
}

fn convert_station(sta: &Station) -> FdsnStation {
    FdsnStation {
        code: sta.code.clone(),
        start_date: format_datetime_opt(&sta.start_date),
        end_date: format_datetime_opt(&sta.end_date),
        restricted_status: None,
        latitude: FdsnFloatValue::new(sta.latitude),
        longitude: FdsnFloatValue::new(sta.longitude),
        elevation: FdsnFloatValue::new(sta.elevation),
        site: FdsnSite {
            name: sta.site.name.clone(),
            description: sta.site.description.clone(),
            town: sta.site.town.clone(),
            county: sta.site.county.clone(),
            region: sta.site.region.clone(),
            country: sta.site.country.clone(),
        },
        creation_date: format_datetime_opt(&sta.creation_date),
        total_number_channels: None,
        selected_number_channels: None,
        channels: sta.channels.iter().map(convert_channel).collect(),
    }
}

fn convert_channel(ch: &Channel) -> FdsnChannel {
    FdsnChannel {
        code: ch.code.clone(),
        location_code: ch.location_code.clone(),
        start_date: format_datetime_opt(&ch.start_date),
        end_date: format_datetime_opt(&ch.end_date),
        restricted_status: None,
        latitude: FdsnFloatValue::new(ch.latitude),
        longitude: FdsnFloatValue::new(ch.longitude),
        elevation: FdsnFloatValue::new(ch.elevation),
        depth: FdsnFloatValue::new(ch.depth),
        azimuth: FdsnFloatValue::new(ch.azimuth),
        dip: FdsnFloatValue::new(ch.dip),
        channel_type: None,
        sample_rate: FdsnFloatValue::new(ch.sample_rate),
        sensor: ch.sensor.as_ref().map(convert_equipment),
        data_logger: ch.data_logger.as_ref().map(convert_equipment),
        response: ch.response.as_ref().map(convert_response),
    }
}

fn convert_equipment(eq: &Equipment) -> FdsnEquipment {
    FdsnEquipment {
        equipment_type: eq.equipment_type.clone(),
        description: eq.description.clone(),
        manufacturer: eq.manufacturer.clone(),
        vendor: eq.vendor.clone(),
        model: eq.model.clone(),
        serial_number: eq.serial_number.clone(),
        installation_date: format_datetime_opt(&eq.installation_date),
        removal_date: format_datetime_opt(&eq.removal_date),
    }
}

fn convert_response(resp: &Response) -> FdsnResponse {
    FdsnResponse {
        instrument_sensitivity: resp
            .instrument_sensitivity
            .as_ref()
            .map(convert_sensitivity),
        stages: resp.stages.iter().map(convert_stage).collect(),
    }
}

fn convert_sensitivity(sens: &InstrumentSensitivity) -> FdsnInstrumentSensitivity {
    FdsnInstrumentSensitivity {
        value: sens.value,
        frequency: sens.frequency,
        input_units: convert_units(&sens.input_units),
        output_units: convert_units(&sens.output_units),
    }
}

fn convert_units(units: &Units) -> FdsnUnits {
    FdsnUnits {
        name: units.name.clone(),
        description: units.description.clone(),
    }
}

fn convert_stage(stage: &ResponseStage) -> FdsnResponseStage {
    FdsnResponseStage {
        number: stage.number,
        poles_zeros: stage.poles_zeros.as_ref().map(convert_poles_zeros),
        coefficients: stage.coefficients.as_ref().map(convert_coefficients),
        fir: stage.fir.as_ref().map(convert_fir),
        decimation: stage.decimation.as_ref().map(|d| FdsnDecimation {
            input_sample_rate: FdsnFloatValue::new(d.input_sample_rate),
            factor: d.factor,
            offset: d.offset,
            delay: FdsnFloatValue::new(d.delay),
            correction: FdsnFloatValue::new(d.correction),
        }),
        stage_gain: stage.stage_gain.as_ref().map(|g| FdsnStageGain {
            value: g.value,
            frequency: g.frequency,
        }),
    }
}

fn convert_poles_zeros(pz: &PolesZeros) -> FdsnPolesZeros {
    FdsnPolesZeros {
        input_units: convert_units(&pz.input_units),
        output_units: convert_units(&pz.output_units),
        pz_transfer_function_type: format_pz_transfer_function(&pz.pz_transfer_function_type),
        normalization_factor: pz.normalization_factor,
        normalization_frequency: FdsnFloatValue::new(pz.normalization_frequency),
        zeros: pz
            .zeros
            .iter()
            .map(|z| FdsnPoleZero {
                number: z.number,
                real: FdsnFloatValue::new(z.real),
                imaginary: FdsnFloatValue::new(z.imaginary),
            })
            .collect(),
        poles: pz
            .poles
            .iter()
            .map(|p| FdsnPoleZero {
                number: p.number,
                real: FdsnFloatValue::new(p.real),
                imaginary: FdsnFloatValue::new(p.imaginary),
            })
            .collect(),
    }
}

fn convert_coefficients(cf: &Coefficients) -> FdsnCoefficients {
    FdsnCoefficients {
        input_units: convert_units(&cf.input_units),
        output_units: convert_units(&cf.output_units),
        cf_transfer_function_type: format_cf_transfer_function(&cf.cf_transfer_function_type),
        numerators: cf
            .numerators
            .iter()
            .map(|&v| FdsnFloatValue::new(v))
            .collect(),
        denominators: cf
            .denominators
            .iter()
            .map(|&v| FdsnFloatValue::new(v))
            .collect(),
    }
}

fn convert_fir(fir: &FIR) -> FdsnFIR {
    FdsnFIR {
        input_units: convert_units(&fir.input_units),
        output_units: convert_units(&fir.output_units),
        symmetry: format_symmetry(&fir.symmetry),
        numerator_coefficients: fir
            .numerator_coefficients
            .iter()
            .map(|&v| FdsnFloatValue::new(v))
            .collect(),
    }
}

// ─── Enum formatting ────────────────────────────────────────────────

fn format_pz_transfer_function(pz: &PzTransferFunction) -> String {
    match pz {
        PzTransferFunction::LaplaceRadians => "LAPLACE (RADIANS/SECOND)".into(),
        PzTransferFunction::LaplaceHertz => "LAPLACE (HERTZ)".into(),
        PzTransferFunction::DigitalZTransform => "DIGITAL (Z-TRANSFORM)".into(),
    }
}

fn format_cf_transfer_function(cf: &CfTransferFunction) -> String {
    match cf {
        CfTransferFunction::AnalogRadians => "ANALOG (RADIANS/SECOND)".into(),
        CfTransferFunction::AnalogHertz => "ANALOG (HERTZ)".into(),
        CfTransferFunction::Digital => "DIGITAL".into(),
    }
}

fn format_symmetry(sym: &Symmetry) -> String {
    match sym {
        Symmetry::None => "NONE".into(),
        Symmetry::Even => "EVEN".into(),
        Symmetry::Odd => "ODD".into(),
    }
}

//! FDSN StationXML reader: XML → fdsn types → Inventory.

use crate::datetime::parse_datetime_opt;
use crate::error::{Result, StationXmlError};
use crate::inventory::*;

use super::types::*;

/// Parse FDSN StationXML string into an [`Inventory`].
pub(crate) fn read_from_str(xml: &str) -> Result<Inventory> {
    let fdsn: FdsnStationXml = quick_xml::de::from_str(xml)?;
    fdsn_to_inventory(fdsn)
}

/// Parse FDSN StationXML bytes into an [`Inventory`].
pub(crate) fn read_from_bytes(bytes: &[u8]) -> Result<Inventory> {
    let xml =
        std::str::from_utf8(bytes).map_err(|e| StationXmlError::InvalidData(e.to_string()))?;
    read_from_str(xml)
}

// ─── Conversion functions ───────────────────────────────────────────

fn fdsn_to_inventory(fdsn: FdsnStationXml) -> Result<Inventory> {
    Ok(Inventory {
        source: fdsn.source,
        sender: fdsn.sender,
        created: parse_datetime_opt(&Some(fdsn.created))?,
        networks: fdsn
            .networks
            .into_iter()
            .map(convert_network)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_network(net: FdsnNetwork) -> Result<Network> {
    Ok(Network {
        code: net.code,
        description: net.description,
        start_date: parse_datetime_opt(&net.start_date)?,
        end_date: parse_datetime_opt(&net.end_date)?,
        stations: net
            .stations
            .into_iter()
            .map(convert_station)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_station(sta: FdsnStation) -> Result<Station> {
    Ok(Station {
        code: sta.code,
        description: None,
        latitude: sta.latitude.value,
        longitude: sta.longitude.value,
        elevation: sta.elevation.value,
        site: Site {
            name: sta.site.name,
            description: sta.site.description,
            town: sta.site.town,
            county: sta.site.county,
            region: sta.site.region,
            country: sta.site.country,
        },
        start_date: parse_datetime_opt(&sta.start_date)?,
        end_date: parse_datetime_opt(&sta.end_date)?,
        creation_date: parse_datetime_opt(&sta.creation_date)?,
        channels: sta
            .channels
            .into_iter()
            .map(convert_channel)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_channel(ch: FdsnChannel) -> Result<Channel> {
    Ok(Channel {
        code: ch.code,
        location_code: ch.location_code,
        latitude: ch.latitude.value,
        longitude: ch.longitude.value,
        elevation: ch.elevation.value,
        depth: ch.depth.value,
        azimuth: ch.azimuth.value,
        dip: ch.dip.value,
        sample_rate: ch.sample_rate.value,
        start_date: parse_datetime_opt(&ch.start_date)?,
        end_date: parse_datetime_opt(&ch.end_date)?,
        sensor: ch.sensor.map(convert_equipment),
        data_logger: ch.data_logger.map(convert_equipment),
        response: ch.response.map(convert_response).transpose()?,
    })
}

fn convert_equipment(eq: FdsnEquipment) -> Equipment {
    Equipment {
        equipment_type: eq.equipment_type,
        description: eq.description,
        manufacturer: eq.manufacturer,
        vendor: eq.vendor,
        model: eq.model,
        serial_number: eq.serial_number,
        installation_date: None, // datetime parsing for equipment dates
        removal_date: None,
    }
}

fn convert_response(resp: FdsnResponse) -> Result<Response> {
    Ok(Response {
        instrument_sensitivity: resp.instrument_sensitivity.map(convert_sensitivity),
        stages: resp
            .stages
            .into_iter()
            .map(convert_stage)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_sensitivity(sens: FdsnInstrumentSensitivity) -> InstrumentSensitivity {
    InstrumentSensitivity {
        value: sens.value,
        frequency: sens.frequency,
        input_units: convert_units(sens.input_units),
        output_units: convert_units(sens.output_units),
    }
}

fn convert_units(units: FdsnUnits) -> Units {
    Units {
        name: units.name,
        description: units.description,
    }
}

fn convert_stage(stage: FdsnResponseStage) -> Result<ResponseStage> {
    Ok(ResponseStage {
        number: stage.number,
        stage_gain: stage.stage_gain.map(|g| StageGain {
            value: g.value,
            frequency: g.frequency,
        }),
        poles_zeros: stage.poles_zeros.map(convert_poles_zeros).transpose()?,
        coefficients: stage.coefficients.map(convert_coefficients).transpose()?,
        fir: stage.fir.map(convert_fir).transpose()?,
        decimation: stage.decimation.map(|d| Decimation {
            input_sample_rate: d.input_sample_rate.value,
            factor: d.factor,
            offset: d.offset,
            delay: d.delay.value,
            correction: d.correction.value,
        }),
    })
}

fn convert_poles_zeros(pz: FdsnPolesZeros) -> Result<PolesZeros> {
    Ok(PolesZeros {
        input_units: convert_units(pz.input_units),
        output_units: convert_units(pz.output_units),
        pz_transfer_function_type: parse_pz_transfer_function(&pz.pz_transfer_function_type)?,
        normalization_factor: pz.normalization_factor,
        normalization_frequency: pz.normalization_frequency.value,
        zeros: pz
            .zeros
            .into_iter()
            .map(|z| PoleZero {
                number: z.number,
                real: z.real.value,
                imaginary: z.imaginary.value,
            })
            .collect(),
        poles: pz
            .poles
            .into_iter()
            .map(|p| PoleZero {
                number: p.number,
                real: p.real.value,
                imaginary: p.imaginary.value,
            })
            .collect(),
    })
}

fn convert_coefficients(cf: FdsnCoefficients) -> Result<Coefficients> {
    Ok(Coefficients {
        input_units: convert_units(cf.input_units),
        output_units: convert_units(cf.output_units),
        cf_transfer_function_type: parse_cf_transfer_function(&cf.cf_transfer_function_type)?,
        numerators: cf.numerators.into_iter().map(|v| v.value).collect(),
        denominators: cf.denominators.into_iter().map(|v| v.value).collect(),
    })
}

fn convert_fir(fir: FdsnFIR) -> Result<FIR> {
    Ok(FIR {
        input_units: convert_units(fir.input_units),
        output_units: convert_units(fir.output_units),
        symmetry: parse_symmetry(&fir.symmetry)?,
        numerator_coefficients: fir
            .numerator_coefficients
            .into_iter()
            .map(|v| v.value)
            .collect(),
    })
}

// ─── Enum parsing ───────────────────────────────────────────────────

fn parse_pz_transfer_function(s: &str) -> Result<PzTransferFunction> {
    match s {
        "LAPLACE (RADIANS/SECOND)" => Ok(PzTransferFunction::LaplaceRadians),
        "LAPLACE (HERTZ)" => Ok(PzTransferFunction::LaplaceHertz),
        "DIGITAL (Z-TRANSFORM)" => Ok(PzTransferFunction::DigitalZTransform),
        _ => Err(StationXmlError::InvalidData(format!(
            "unknown PzTransferFunctionType: '{s}'"
        ))),
    }
}

fn parse_cf_transfer_function(s: &str) -> Result<CfTransferFunction> {
    match s {
        "ANALOG (RADIANS/SECOND)" => Ok(CfTransferFunction::AnalogRadians),
        "ANALOG (HERTZ)" => Ok(CfTransferFunction::AnalogHertz),
        "DIGITAL" => Ok(CfTransferFunction::Digital),
        _ => Err(StationXmlError::InvalidData(format!(
            "unknown CfTransferFunctionType: '{s}'"
        ))),
    }
}

fn parse_symmetry(s: &str) -> Result<Symmetry> {
    match s {
        "NONE" => Ok(Symmetry::None),
        "EVEN" => Ok(Symmetry::Even),
        "ODD" => Ok(Symmetry::Odd),
        _ => Err(StationXmlError::InvalidData(format!(
            "unknown Symmetry: '{s}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_sample_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FDSNStationXML xmlns="http://www.fdsn.org/xml/station/1" schemaVersion="1.2">
  <Source>Pena Bumi</Source>
  <Created>2026-02-20T00:00:00Z</Created>
  <Network code="XX">
    <Description>Local Test Network</Description>
    <Station code="PBUMI">
      <Latitude>-7.7714</Latitude>
      <Longitude>110.3776</Longitude>
      <Elevation>150.0</Elevation>
      <Site><Name>Yogyakarta</Name></Site>
      <Channel code="SHZ" locationCode="00">
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
      </Channel>
    </Station>
  </Network>
</FDSNStationXML>"#;
        let inv = read_from_str(xml).unwrap();
        assert_eq!(inv.source, "Pena Bumi");
        assert_eq!(inv.networks.len(), 1);
        assert_eq!(inv.networks[0].code, "XX");
        assert_eq!(
            inv.networks[0].description.as_deref(),
            Some("Local Test Network")
        );

        let sta = &inv.networks[0].stations[0];
        assert_eq!(sta.code, "PBUMI");
        assert!((sta.latitude - (-7.7714)).abs() < 1e-6);
        assert!((sta.longitude - 110.3776).abs() < 1e-6);
        assert!((sta.elevation - 150.0).abs() < 1e-6);
        assert_eq!(sta.site.name, "Yogyakarta");

        let ch = &sta.channels[0];
        assert_eq!(ch.code, "SHZ");
        assert_eq!(ch.location_code, "00");
        assert!((ch.dip - (-90.0)).abs() < 1e-6);
        assert!((ch.sample_rate - 100.0).abs() < 1e-6);

        let sensor = ch.sensor.as_ref().unwrap();
        assert_eq!(sensor.equipment_type.as_deref(), Some("Geophone"));
        assert_eq!(sensor.model.as_deref(), Some("GS-11D"));

        let resp = ch.response.as_ref().unwrap();
        let sens = resp.instrument_sensitivity.as_ref().unwrap();
        assert!((sens.value - 53721548.8).abs() < 0.1);
        assert_eq!(sens.input_units.name, "M/S");
        assert_eq!(sens.output_units.name, "COUNTS");
    }

    #[test]
    fn read_with_response_stages() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FDSNStationXML schemaVersion="1.2">
  <Source>Test</Source>
  <Created>2026-01-01T00:00:00Z</Created>
  <Network code="XX">
    <Station code="TEST">
      <Latitude>0.0</Latitude>
      <Longitude>0.0</Longitude>
      <Elevation>0.0</Elevation>
      <Site><Name>Test Site</Name></Site>
      <Channel code="SHZ" locationCode="00">
        <Latitude>0.0</Latitude>
        <Longitude>0.0</Longitude>
        <Elevation>0.0</Elevation>
        <Depth>0.0</Depth>
        <Azimuth>0.0</Azimuth>
        <Dip>-90.0</Dip>
        <SampleRate>100.0</SampleRate>
        <Response>
          <InstrumentSensitivity>
            <Value>53721548.8</Value>
            <Frequency>15.0</Frequency>
            <InputUnits><Name>M/S</Name></InputUnits>
            <OutputUnits><Name>COUNTS</Name></OutputUnits>
          </InstrumentSensitivity>
          <Stage number="1">
            <PolesZeros>
              <InputUnits><Name>M/S</Name></InputUnits>
              <OutputUnits><Name>V</Name></OutputUnits>
              <PzTransferFunctionType>LAPLACE (RADIANS/SECOND)</PzTransferFunctionType>
              <NormalizationFactor>1.0</NormalizationFactor>
              <NormalizationFrequency>15.0</NormalizationFrequency>
              <Zero number="0"><Real>0.0</Real><Imaginary>0.0</Imaginary></Zero>
              <Zero number="1"><Real>0.0</Real><Imaginary>0.0</Imaginary></Zero>
              <Pole number="0"><Real>-19.8</Real><Imaginary>19.4</Imaginary></Pole>
              <Pole number="1"><Real>-19.8</Real><Imaginary>-19.4</Imaginary></Pole>
            </PolesZeros>
            <StageGain><Value>32.0</Value><Frequency>15.0</Frequency></StageGain>
          </Stage>
          <Stage number="2">
            <Coefficients>
              <InputUnits><Name>V</Name></InputUnits>
              <OutputUnits><Name>COUNTS</Name></OutputUnits>
              <CfTransferFunctionType>DIGITAL</CfTransferFunctionType>
            </Coefficients>
            <Decimation>
              <InputSampleRate>100.0</InputSampleRate>
              <Factor>1</Factor>
              <Offset>0</Offset>
              <Delay>0.0</Delay>
              <Correction>0.0</Correction>
            </Decimation>
            <StageGain><Value>1678801.5</Value><Frequency>15.0</Frequency></StageGain>
          </Stage>
        </Response>
      </Channel>
    </Station>
  </Network>
</FDSNStationXML>"#;
        let inv = read_from_str(xml).unwrap();
        let resp = inv.networks[0].stations[0].channels[0]
            .response
            .as_ref()
            .unwrap();
        assert_eq!(resp.stages.len(), 2);

        // Stage 1: Poles & Zeros
        let s1 = &resp.stages[0];
        assert_eq!(s1.number, 1);
        let pz = s1.poles_zeros.as_ref().unwrap();
        assert_eq!(
            pz.pz_transfer_function_type,
            PzTransferFunction::LaplaceRadians
        );
        assert_eq!(pz.zeros.len(), 2);
        assert_eq!(pz.poles.len(), 2);
        assert!((pz.poles[0].real - (-19.8)).abs() < 1e-6);
        assert!((pz.poles[0].imaginary - 19.4).abs() < 1e-6);
        assert!((s1.stage_gain.as_ref().unwrap().value - 32.0).abs() < 1e-6);

        // Stage 2: Coefficients + Decimation
        let s2 = &resp.stages[1];
        assert_eq!(s2.number, 2);
        let cf = s2.coefficients.as_ref().unwrap();
        assert_eq!(cf.cf_transfer_function_type, CfTransferFunction::Digital);
        let dec = s2.decimation.as_ref().unwrap();
        assert!((dec.input_sample_rate - 100.0).abs() < 1e-6);
        assert_eq!(dec.factor, 1);
        assert!((s2.stage_gain.as_ref().unwrap().value - 1678801.5).abs() < 0.1);
    }

    #[test]
    fn read_from_bytes_works() {
        let xml = r#"<?xml version="1.0"?>
<FDSNStationXML schemaVersion="1.2">
  <Source>Test</Source>
  <Created>2026-01-01T00:00:00Z</Created>
</FDSNStationXML>"#;
        let inv = read_from_bytes(xml.as_bytes()).unwrap();
        assert_eq!(inv.source, "Test");
    }
}

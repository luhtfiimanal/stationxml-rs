//! ADC conversion helpers.
//!
//! Convert between raw ADC counts, voltage, and physical units
//! using sensor sensitivity and digitizer parameters.
//!
//! # Formulas
//!
//! ```text
//! voltage = (count / max_count) * full_scale_voltage / (pga_gain * adc_gain)
//! physical = voltage / sensor_sensitivity
//! overall_sensitivity = max_count * pga_gain * adc_gain * sensor_sensitivity / full_scale_voltage
//! ```
//!
//! See `docs/guide/03-instrument-response.md` for background.

/// Parameters for ADC count / voltage conversion.
///
/// Describes the digitizer characteristics needed to convert between
/// raw ADC counts and voltage (or physical units with a sensor sensitivity).
#[derive(Debug, Clone, PartialEq)]
pub struct AdcConversion {
    /// Full-scale range in Volts (e.g. 5.0 for a +/-2.5V ADC)
    pub full_scale_voltage: f64,
    /// Maximum count value: 2^(bits-1) - 1 (e.g. 8388607 for 24-bit)
    pub max_count: f64,
    /// External PGA (Programmable Gain Amplifier) gain (e.g. 1.0)
    pub pga_gain: f64,
    /// Internal digital gain in ADC (e.g. 1.0)
    pub adc_gain: f64,
}

impl AdcConversion {
    /// Create from ADC bit depth and gains.
    ///
    /// `max_count` is computed as 2^(bits-1) - 1.
    pub fn new(full_scale_voltage: f64, bits: u32, pga_gain: f64, adc_gain: f64) -> Self {
        Self {
            full_scale_voltage,
            max_count: (1_i64 << (bits - 1)) as f64 - 1.0,
            pga_gain,
            adc_gain,
        }
    }

    /// Convert raw ADC count to input voltage (before PGA).
    pub fn count_to_voltage(&self, count: f64) -> f64 {
        (count / self.max_count) * self.full_scale_voltage / (self.pga_gain * self.adc_gain)
    }

    /// Convert input voltage to raw ADC count.
    pub fn voltage_to_count(&self, voltage: f64) -> f64 {
        voltage * self.max_count * self.pga_gain * self.adc_gain / self.full_scale_voltage
    }

    /// Convert raw ADC count to physical unit using sensor sensitivity.
    ///
    /// `sensitivity` is in V/(m/s) for velocity sensors.
    pub fn count_to_physical(&self, count: f64, sensitivity: f64) -> f64 {
        self.count_to_voltage(count) / sensitivity
    }

    /// Compute overall sensitivity in counts per physical unit.
    ///
    /// This is the value that goes into `<InstrumentSensitivity><Value>`.
    pub fn overall_sensitivity(&self, sensor_sensitivity: f64) -> f64 {
        self.max_count * self.pga_gain * self.adc_gain * sensor_sensitivity
            / self.full_scale_voltage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs5532_24bit() -> AdcConversion {
        // CS5532 24-bit ADC, 5V full-scale, no PGA, no digital gain
        AdcConversion::new(5.0, 24, 1.0, 1.0)
    }

    #[test]
    fn max_count_24bit() {
        let adc = cs5532_24bit();
        assert_eq!(adc.max_count, 8388607.0);
    }

    #[test]
    fn count_to_voltage_full_scale() {
        let adc = cs5532_24bit();
        let v = adc.count_to_voltage(adc.max_count);
        assert!((v - 5.0).abs() < 1e-6);
    }

    #[test]
    fn count_to_voltage_zero() {
        let adc = cs5532_24bit();
        assert_eq!(adc.count_to_voltage(0.0), 0.0);
    }

    #[test]
    fn voltage_to_count_roundtrip() {
        let adc = cs5532_24bit();
        let voltage = 2.5;
        let count = adc.voltage_to_count(voltage);
        let v_back = adc.count_to_voltage(count);
        assert!((v_back - voltage).abs() < 1e-6);
    }

    #[test]
    fn overall_sensitivity_gs11d() {
        let adc = cs5532_24bit();
        // GS-11D sensitivity: 32.0 V/(m/s)
        let overall = adc.overall_sensitivity(32.0);
        // Expected: 8388607 * 1 * 1 * 32.0 / 5.0 = 53687084.8
        assert!((overall - 53687084.8).abs() < 0.1);
    }

    #[test]
    fn count_to_physical() {
        let adc = cs5532_24bit();
        let sensitivity = 32.0; // V/(m/s)
        // 1000 counts -> voltage -> physical
        let physical = adc.count_to_physical(1000.0, sensitivity);
        let expected_voltage = 1000.0 / 8388607.0 * 5.0;
        let expected_physical = expected_voltage / 32.0;
        assert!((physical - expected_physical).abs() < 1e-12);
    }

    #[test]
    fn with_pga_gain() {
        let adc = AdcConversion::new(5.0, 24, 2.0, 1.0);
        // With PGA=2, effective voltage is halved for same count
        let v_no_pga = cs5532_24bit().count_to_voltage(1000.0);
        let v_with_pga = adc.count_to_voltage(1000.0);
        assert!((v_with_pga - v_no_pga / 2.0).abs() < 1e-10);
    }
}

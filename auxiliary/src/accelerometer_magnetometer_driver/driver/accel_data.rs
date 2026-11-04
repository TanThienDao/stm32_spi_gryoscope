//! =====================================================================
//! Struct for representing accelerometer data readings in milli-g (mg).
//!
//! The `AccelData` struct is used to store the converted accelerometer
//! readings along the X, Y, and Z axes in milli-g (mg).
//! It provides a convenient way to represent the acceleration values
//! after converting from raw ADC readings using the appropriate scale factor
//! based on the accelerometer's configuration.
//! todo this class was not used yet.
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use crate::accelerometer_magnetometer_driver::driver::accel_config::AccelConfig;

/// Represents converted accelerometer readings.
#[derive(Debug, Clone, Copy)]
pub struct AccelData {
    /// Acceleration along the X-axis in milli-g (mg)
    pub x_mg: f32,
    /// Acceleration along the Y-axis in milli-g (mg)
    pub y_mg: f32,
    /// Acceleration along the Z-axis in milli-g (mg)
    pub z_mg: f32,
}

impl AccelData {
    /// Create a new `AccelData` instance from raw ADC values and a scale factor.
    ///
    /// # Arguments
    /// * `raw_x` - Raw ADC value for the X-axis
    /// * `raw_y` - Raw ADC value for the Y-axis
    /// * `raw_z` - Raw ADC value for the Z-axis
    /// * `scale_factor_mg` - Scale factor in mg/LSB for converting raw values to milli-g
    ///
    /// # Returns
    /// An instance of `AccelData` with converted values in milli-g.
    pub fn from_raw(config: &AccelConfig, raw_x: i16, raw_y: i16, raw_z: i16) -> Self {
        let scale_factor_mg = config.scale_factor_mg();
        AccelData {
            x_mg: raw_x as f32 * scale_factor_mg,
            y_mg: raw_y as f32 * scale_factor_mg,
            z_mg: raw_z as f32 * scale_factor_mg,
        }
    }

    /// Convert to grams (g) by dividing the milli-g values by 1000.
    ///
    /// # Parameters
    /// * `self` - The `AccelData` instance containing the acceleration values in milli-g (mg)
    ///
    /// # Returns
    /// A tuple containing the acceleration values in grams (g) for the X, Y, and Z axes.
    ///
    pub fn to_g(&self) -> (f32, f32, f32) {
        (self.x_mg / 1000.0, self.y_mg / 1000.0, self.z_mg / 1000.0)
    }

    /// Convert to meters per second squared (m/s²)
    /// by multiplying the milli-g values by 9.80665 / 1000.
    ///
    /// # Parameters
    /// * `self` - The `AccelData` instance containing the acceleration values in milli-g (mg)
    ///
    /// # Returns
    /// A tuple containing the acceleration values in meters per second squared (m/s²) for the X, Y, and Z axes.
    ///
    pub fn to_m_s2(&self) -> (f32, f32, f32) {
        const GRAVITY: f32 = 9.80665; // Standard gravity in m/s²
        let (x_g, y_g, z_g) = self.to_g();
        (x_g * GRAVITY, y_g * GRAVITY, z_g * GRAVITY)
    }
}

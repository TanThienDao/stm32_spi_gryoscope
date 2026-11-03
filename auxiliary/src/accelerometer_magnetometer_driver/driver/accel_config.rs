//! =====================================================================
//! Configuration of the accelerometer driver.
//! This module defines the accelerometer range selection and operating mode,
//! along with their corresponding scaling factors base on the LSM303AGR datasheet.
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use super::super::registers::accelerometer_registers::*;
/// Accelerometer full-scale range  selection  and scaling factor. The scaling factor.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AccelRange {
    /// ±2g, scale factor
    G2,
    /// ±4g  scale factor
    G4,
    /// ±8g  scale factor
    G8,
    /// ±16g scale factor
    G16,
}

impl AccelRange {
    /// Get the register value for CTRL_REG4_A FULL_SCALE bits [5:4]
    pub fn ctrl_reg4_bits(&self) -> u8 {
        match self {
            AccelRange::G2 => ACCEL_FS_2G,
            AccelRange::G4 => ACCEL_FS_4G,
            AccelRange::G8 => ACCEL_FS_8G,
            AccelRange::G16 => ACCEL_FS_16G,
        }
    }
}

/// Accelerometer operating mode affecting data resolution and scale factor.
///
/// The LSM303AGR accelerometer supports three operating modes: Normal, High-Resolution, and Low-Power.
/// - HR bit (bit 3 of CTRL_REG4): High-Resolution mode (12-bit output)
/// - LP bit (bit 3 of CTRL_REG1): Low-Power mode (8-bit output)
/// - Neither set = Normal mode (10-bit output, default)
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AccelOperatingMode {
    /// High-Resolution mode: 12-bit output, set ACCEL_HR_ENABLE in CTRL_REG4_A
    /// Best accuracy, consumes more power
    HighResolution,

    /// Normal mode (default): 10-bit output
    /// Balanced between accuracy and power consumption
    Normal,

    /// Low-Power mode: 8-bit output, set ACCEL_LP_ENABLE in CTRL_REG1_A
    /// Lower resolution but reduced power consumption
    LowPower,
}
///
/// Usage Example:
///
/// High-resolution mode at ±2g
/// let config = AccelConfig::new(AccelRange::G2, AccelOperatingMode::HighResolution);
/// let scale = config.scale_factor_mg(); // Returns 0.98 mg/LSB
///
/// Normal mode at ±16g
/// let config = AccelConfig::new(AccelRange::G16, AccelOperatingMode::Normal);
/// let scale = config.scale_factor_mg(); // Returns 46.9 mg/LSB
///
/// Low-power mode at ±4g
/// let config = AccelConfig::new(AccelRange::G4, AccelOperatingMode::LowPower);
/// let scale = config.scale_factor_mg(); // Returns 31.26 mg/LSB
impl AccelOperatingMode {
    /// Get the control register bits needed to set this operating mode.
    ///
    /// Returns a tuple of (CTRL_REG1_bits, CTRL_REG4_bits) to configure the mode.
    /// The caller should OR these values with other bits in the respective registers.
    pub fn ctrl_reg_bits(&self) -> (u8, u8) {
        match self {
            AccelOperatingMode::HighResolution => (0x00, ACCEL_HR_ENABLE),
            AccelOperatingMode::Normal => (0x00, 0x00),
            AccelOperatingMode::LowPower => (ACCEL_LP_ENABLE, 0x00),
        }
    }
}
/// Accelerometer output data rate selection
///
/// Description: The output data rate (ODR) determines how frequently the accelerometer updates its
///      data registers with new measurements. The LSM303AGR supports various ODR settings, which.
///      can be configured in CTRL_REG1_A. The ODR affects both the responsiveness of the sensor and
///      its power consumption.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AccelDataRate {
    PowerDown,
    Hz1,
    Hz10,
    Hz25,
    Hz50,
    Hz100,
    Hz200,
    Hz400,
    LowPower1_6KHz,
    LowPower5_376KHz,
}
impl AccelDataRate {
    /// Get the register value for CTRL_REG1_A ODR bits [7:4]
    pub fn ctrl_reg1_bits(&self) -> u8 {
        match self {
            AccelDataRate::PowerDown => ACCEL_ODR_POWER_DOWN,
            AccelDataRate::Hz1 => ACCEL_ODR_1_HZ,
            AccelDataRate::Hz10 => ACCEL_ODR_10_HZ,
            AccelDataRate::Hz25 => ACCEL_ODR_25_HZ,
            AccelDataRate::Hz50 => ACCEL_ODR_50_HZ,
            AccelDataRate::Hz100 => ACCEL_ODR_100_HZ,
            AccelDataRate::Hz200 => ACCEL_ODR_200_HZ,
            AccelDataRate::Hz400 => ACCEL_ODR_400_HZ,
            AccelDataRate::LowPower1_6KHz => ACCEL_ODR_LOW_POWER_1_6_KHZ,
            AccelDataRate::LowPower5_376KHz => ACCEL_ODR_LOW_POWER_5_376_KHZ,
        }
    }
}

/// Accelerometer scale factor configuration.
/// Combines range and operating mode to determine mg/LSB conversion factor.
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AccelConfig {
    /// Full-scale range selection (±2g, ±4g, ±8g, ±16g)
    pub range: AccelRange,
    /// Operating mode selection (High-Resolution, Normal, Low-Power)
    pub mode: AccelOperatingMode,
    /// Output data rate selection (PowerDown, 1Hz, 10Hz, etc.)
    pub data_rate: AccelDataRate,
}

impl AccelConfig {
    /// Create a new accelerometer configuration.
    pub fn new(range: AccelRange, mode: AccelOperatingMode, data_rate: AccelDataRate) -> Self {
        AccelConfig {
            range,
            mode,
            data_rate,
        }
    }
    /// Get the combined register bits for CTRL_REG1_A (data rate, mode, and axis enable).
    /// This method combines the data rate bits, mode bits, and enables all axes.
    /// The caller should OR this value with any additional bits needed for specific configurations.
    pub fn get_ctrl_reg1_bits(&self) -> u8 {
        self.data_rate.ctrl_reg1_bits() | self.mode.ctrl_reg_bits().0 | ACCEL_ALL_AXES
    }

    /// Get the combined register bits for CTRL_REG4_A (range and mode).
    /// This method combines the range bits and mode bits.
    /// The caller should OR this value with any additional bits needed for specific configurations.
    pub fn get_ctrl_reg4_bits(&self) -> u8 {
        self.range.ctrl_reg4_bits() | self.mode.ctrl_reg_bits().1
    }

    /// Get the scale factor in mg/LSB for converting raw ADC values to milligrammage.
    ///
    /// # Arguments
    /// * `range` - The full-scale range setting
    /// * `mode` - The operating mode (High-Res, Normal, or Low-Power)
    /// * `data_rate` - The output data rate selection (PowerDown, 1Hz, 10Hz, etc.)
    ///
    /// # Returns
    /// Scale factor in mg/LSB (milligrams per Least Significant Bit)
    ///
    /// # Example
    /// ```ignore
    /// let config = AccelConfig::new(AccelRange::G2, AccelOperatingMode::HighResolution, AccelDataRate::Hz100);
    /// let scale = config.scale_factor_mg();
    /// let raw_value: i16 = 1000; // Example raw ADC reading
    /// let mg = raw_value as f32 * scale; // Convert to milligrams
    /// ```
    ///
    /// # Data Sheet Reference
    /// LSM303AGR Table 3: Linear acceleration sensitivity (LA_So)
    /// - High-Resolution mode (12-bit): ±7% tolerance
    /// - Normal mode (10-bit): ±7% tolerance
    /// - Low-Power mode (8-bit): ±7% tolerance
    pub fn scale_factor_mg(&self) -> f32 {
        match (self.range, self.mode) {
            // ========== High-Resolution Mode (12-bit output) ==========
            // Best accuracy, typical values from datasheet
            (AccelRange::G2, AccelOperatingMode::HighResolution) => 0.98, // 1 mg/LSB nominal
            (AccelRange::G4, AccelOperatingMode::HighResolution) => 1.95, // 2 mg/LSB nominal
            (AccelRange::G8, AccelOperatingMode::HighResolution) => 3.9,  // 4 mg/LSB nominal
            (AccelRange::G16, AccelOperatingMode::HighResolution) => 11.72, // 12 mg/LSB nominal

            // ========== Normal Mode (10-bit output, default) ==========
            // Balanced resolution and power consumption
            (AccelRange::G2, AccelOperatingMode::Normal) => 3.9, // 4 mg/LSB nominal
            (AccelRange::G4, AccelOperatingMode::Normal) => 7.82, // 8 mg/LSB nominal
            (AccelRange::G8, AccelOperatingMode::Normal) => 15.63, // 16 mg/LSB nominal
            (AccelRange::G16, AccelOperatingMode::Normal) => 46.9, // 48 mg/LSB nominal

            // ========== Low-Power Mode (8-bit output) ==========
            // Lower resolution, reduced power consumption
            (AccelRange::G2, AccelOperatingMode::LowPower) => 15.63, // 16 mg/LSB nominal
            (AccelRange::G4, AccelOperatingMode::LowPower) => 31.26, // 32 mg/LSB nominal
            (AccelRange::G8, AccelOperatingMode::LowPower) => 62.52, // 64 mg/LSB nominal
            (AccelRange::G16, AccelOperatingMode::LowPower) => 187.58, // 192 mg/LSB nominal
        }
    }

    /// Get register bits for CTRL_REG4 (full-scale range configuration).
    pub fn ctrl_reg4_bits(&self) -> u8 {
        let range_bits = self.range.ctrl_reg4_bits();
        let (mode_bits, _) = self.mode.ctrl_reg_bits();
        range_bits | mode_bits
    }

    /// Get register bits for CTRL_REG1 (operating mode configuration).
    ///
    /// This returns only the mode bits. You'll need to OR this with ODR and axis enable bits.
    pub fn ctrl_reg1_mode_bits(&self) -> u8 {
        let (_, mode_bits) = self.mode.ctrl_reg_bits();
        mode_bits
    }
}

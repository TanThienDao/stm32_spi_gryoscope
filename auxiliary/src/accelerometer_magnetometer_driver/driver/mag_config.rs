//! ======================================================================
//! Configuration of the magnetometer driver.
//! This struct holds the configuration parameters for the magnetometer driver.
//!
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use super::super::registers::magnetometer_registers::*;

pub const MAG_SENSITIVITY: f32 = 1.5; // mgauss/LSB for ±50 gauss todo can move this or adjust in accel data let that handle instead.
/// Magnetometer full-scale range selection and scaling factor.
#[derive(Debug, Clone, Copy)]
pub enum MagRange {
    /// ±50 gauss, scale factor
    Gauss50,
}
impl MagRange {
    /// Get the register value for CFG_REG_B FULL_SCALE bits [6:5]
    pub fn cfg_reg_b_bits(&self) -> u8 {
        match self {
            MagRange::Gauss50 => 10, // 50 gauss
        }
    }
}
/// Magnetometer data rate selection.
#[derive(Debug, Clone, Copy)]
pub enum MagDataRate {
    Hz10, // Default data rate, 10 Hz
    Hz20,
    Hz50,
    Hz100,
}

impl MagDataRate {
    /// Get the register value for CFG_REG_A_M ODR bits [3:2]
    pub fn cfg_reg_a_m_odr_bits(&self) -> u8 {
        match self {
            MagDataRate::Hz10 => MAG_ODR_10_HZ,   // 10 Hz
            MagDataRate::Hz20 => MAG_ODR_20_HZ,   // 20 Hz
            MagDataRate::Hz50 => MAG_ODR_50_HZ,   // 50 Hz
            MagDataRate::Hz100 => MAG_ODR_100_HZ, // 100 Hz
        }
    }
}

/// Magnetometer operating mode selection.
#[derive(Debug, Clone, Copy)]
pub enum MagOperatingMode {
    /// Continuous-conversion mode (default)
    /// The magnetometer continuously measures the magnetic field
    /// and updates the output registers.
    Continuous,
    /// Single-conversion mode
    /// The magnetometer performs a single measurement
    /// Set DRDY high
    /// and then enters idle mode.
    Single,
    /// Idle mode
    /// The magnetometer is inactive and consumes minimal power.
    Idle,
}

impl MagOperatingMode {
    /// Get the register value for CFG_REG_A_M MD bits [1:0]
    pub fn cfg_reg_a_m_md_bits(&self) -> u8 {
        match self {
            MagOperatingMode::Continuous => MAG_MODE_CONTINUOUS, // Continuous-conversion mode
            MagOperatingMode::Single => MAG_MODE_SINGLE,         // Single-conversion mode
            MagOperatingMode::Idle => MAG_MODE_IDLE,             // Idle mode
        }
    }
}

/// Magnetometer Power Mode selection.
#[derive(Debug, Clone, Copy)]
pub enum MagPowerMode {
    /// High resolution mode, Max ODR 100 Hz
    HighResolution,
    /// Low-power mode, Max ODR 50 Hz
    LowPower,
}
impl MagPowerMode {
    /// Get the register value for CFG_REG_A_M LP bits 3
    pub fn cfg_reg_a_m_power_bits(&self) -> u8 {
        match self {
            MagPowerMode::HighResolution => MAG_LP_DISABLE, // High resolution mode
            MagPowerMode::LowPower => MAG_LP_ENABLE,        // Low-power mode
        }
    }
    pub fn set_mag_power_mode(&self, mode: MagPowerMode) -> u8 {
        match mode {
            MagPowerMode::HighResolution => MAG_LPF_ENABLE, // High resolution mode
            MagPowerMode::LowPower => MAG_LP_DISABLE,       // Low-power mode
        }
    }
    // todo implement the set mag power mode function after driver.
}

/// Magnetometer configuration for the LSM303AGR driver.
/// This struct holds the configuration parameters for the magnetometer driver.
#[derive(Debug, Clone, Copy)]
pub struct MagConfig {
    /// Full-scale range selection (±50 gauss)
    pub range: MagRange,
    /// Data rate selection (10 Hz, 20 Hz, 50 Hz, 100 Hz)
    pub data_rate: MagDataRate,
    /// Operating mode selection (Continuous, Single, Idle)
    pub operating_mode: MagOperatingMode,
    /// Power mode selection (High-Resolution, Low-Power)
    pub power_mode: MagPowerMode,
}

impl MagConfig {
    /// Create a new magnetometer configuration.
    pub fn new(
        operating_mode: MagOperatingMode,
        data_rate: MagDataRate,
        power_mode: MagPowerMode,
    ) -> Self {
        MagConfig {
            range: MagRange::Gauss50, // Default to ±50 gauss
            operating_mode,
            data_rate,
            power_mode,
        }
    }

    /// Get the register value for CFG_REG_A_M based on the configuration.
    pub fn cfg_reg_a_m_bits(&self) -> u8 {
        self.data_rate.cfg_reg_a_m_odr_bits()
            | self.operating_mode.cfg_reg_a_m_md_bits()
            | self.power_mode.cfg_reg_a_m_power_bits()
    }
}

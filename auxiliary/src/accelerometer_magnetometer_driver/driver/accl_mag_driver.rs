//! =====================================================================
//! LSM303ARG Accelerometer and Magnetometer Driver
//!
//! Generic over I2C peripheral type to work with any HAL.
//!
//! # Type Parameters
//! * `I2C` - The I2C peripheral type implementing the `embedded_hal::blocking::i2c::WriteRead` and `embedded_hal::blocking::i2c::Write` traits.
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use crate::accelerometer_magnetometer_driver::driver::accel_config::{
    AccelConfig, AccelDataRate, AccelOperatingMode, AccelRange,
};
use crate::accelerometer_magnetometer_driver::driver::mag_config::{
    MagConfig, MagDataRate, MagOperatingMode, MagPowerMode, MAG_SENSITIVITY,
};
use crate::accelerometer_magnetometer_driver::registers::{
    accelerometer_registers::*, magnetometer_registers::*,
};
use embedded_hal::blocking::i2c::{Write, WriteRead};

/// Represents errors that can occur in the LSM303AGR driver.
#[derive(Debug, Clone, Copy)]
pub enum DriverError<I2CError> {
    /// I2C communication error
    Bus(I2CError),
    /// Invalid configuration parameter
    InvalidConfig,
}

impl<I2CError> From<I2CError> for DriverError<I2CError> {
    fn from(err: I2CError) -> Self {
        DriverError::Bus(err)
    }
}
/// Represents the LSM303AGR driver, generic over the I2C peripheral type.
///
/// # Type Parameters
/// * `I2C` - The I2C peripheral type implementing the
///      `embedded_hal::blocking::i2c
#[derive(Debug)]
pub struct Lsm303agr<I2C> {
    i2c: I2C,
    accel_config: AccelConfig, // Store for scaling factor accelerometer configuration
    mag_config: MagConfig,     // Store for scaling factor magnetometer configuration
}

impl<I2C> Lsm303agr<I2C> {
    /// Create a new instance of the LSM303AGR driver.
    ///
    /// # Arguments
    /// * `i2c` - The I2C peripheral implementing the required traits.
    /// * `accel_config` - The desired accelerometer configuration.
    /// * `mag_config` - The desired magnetometer configuration.
    ///
    /// # Returns
    /// A new instance of the LSM303AGR driver.
    pub fn new(
        i2c: I2C,
        // Accelerometer configuration
        accel_config: AccelConfig,
        // Magnetometer configuration
        mag_config: MagConfig,
    ) -> Self {
        Lsm303agr {
            i2c,
            accel_config,
            mag_config,
        }
    }

    /// Write to an accelerometer register.
    ///
    ///   Description In I2C communication, writing to a register typically involves sending
    ///      the register address followed by the data byte to the device.
    ///      This method encapsulates that process for the accelerometer.
    ///
    /// # Arguments
    /// * `reg_address` - The address of the accelerometer register to write
    /// * `data` - The data byte to write to the register
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DriverError<I2C::Error>)` on I2C communication error
    pub fn write_accel(&mut self, reg_address: u8, data: u8) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        let payload = [reg_address, data];
        self.i2c.write(ACCEL_ADDRESS, &payload)?;
        Ok(())
    }

    /// Read from an accelerometer register.
    ///
    /// Description
    ///     In I2C communication, reading from a register typically
    ///         involves first writing the register address to the device,
    ///         followed by reading the data from that register.
    ///     This method encapsulates that process for the accelerometer.
    ///     The method takes the register address to read from
    ///         and a mutable buffer to store the read data.
    ///
    /// # Arguments
    /// * `reg_address` - The address of the accelerometer register to read
    /// * `buffer` - The buffer to store the read data
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DriverError<I2C::Error>)` on I2C communication error
    pub fn read_accel(
        &mut self,
        reg_address: u8,
        buffer: &mut [u8],
    ) -> Result<u8, DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        self.i2c.write_read(ACCEL_ADDRESS, &[reg_address], buffer)?;
        Ok(buffer[0])
    }

    /// Write to a magnetometer register.
    ///
    ///  Description
    ///      Similar above, but for the magnetometer.
    ///
    ///  Arguments:
    /// * `reg_address` - The address of the magnetometer register to write
    /// * `data` - The data byte to write to the register
    /// Returns:
    /// * `Ok(())` on success
    /// * `Err(DriverError<I2C::Error>)` on I2C communication error
    pub fn write_mag(&mut self, reg_address: u8, data: u8) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        let payload = [reg_address, data];
        self.i2c.write(MAG_ADDRESS, &payload)?;

        Ok(())
    }

    /// Read from a magnetometer register.
    ///
    /// Description
    ///      Similar above, but for the magnetometer.
    ///
    /// Arguments:
    /// * `reg_address` - The address of the magnetometer register to read
    /// * `buffer` - The buffer to store the read data
    /// Returns:
    /// * `Ok(())` on success
    /// * `Err(DriverError<I2C::Error>)` on I2C communication error
    pub fn read_mag(
        &mut self,
        reg_address: u8,
        buffer: &mut [u8],
    ) -> Result<u8, DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        self.i2c.write_read(MAG_ADDRESS, &[reg_address], buffer)?;

        Ok(buffer[0])
    }
    /// Read the WHO_AM_I_A register of the accelerometer to verify its identity.
    ///
    /// Description
    ///  The WHO_AM_I register is a standard register in many sensors that contains
    /// a fixed value used to identify the device. This method reads that register from the
    /// accelerometer and returns its value.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok(u8)` containing the value of the WHO_AM_I register on success
    ///     Expecting a specific value (e.g., 0x33 for LSM303AGR) to confirm the device identity.
    /// * `Err(DriverError<I2C::Error>)` on I2C communication error
    pub fn who_am_i_accel(&mut self) -> Result<u8, DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let mut buffer = [0u8; 1];
        self.read_accel(ACCEL_WHO_AM_I, &mut buffer)?;
        Ok(buffer[0])
    }
    /// Read the WHO_AM_I_M register of the magnetometer to verify its identity.
    ///
    /// Description
    ///      Similar to the accelerometer, this method reads the WHO_AM_I register
    ///     from the magnetometer and returns its value.
    ///     Expecting a specific value (e.g., 0x40 for LSM303AGR)
    ///     to confirm the device identity.
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok(u8)` containing the value of the WHO_AM_I register on success
    /// * `Err(DriverError<I2C::Error>)` on I2C communication error
    pub fn who_am_i_mag(&mut self) -> Result<u8, DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let mut buffer = [0u8; 1];
        self.read_mag(MAG_WHO_AM_I, &mut buffer)?;
        Ok(buffer[0])
    }

    /// Verify the identity of both the accelerometer and magnetometer by reading their WHO_AM_I
    /// register.
    ///
    /// Description
    /// This method reads the WHO_AM_I registers of both the accelerometer and
    /// magnetometer, compares the returned values to the expected values, and returns a tuple
    /// indicating whether each device's identity is verified.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok((u8, u8))` - A tuple where the first element is the accelerometer ID
    ///      and the second element is the magnetometer ID.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while reading
    ///      either WHO_AM_I register.
    ///
    pub fn verify_device_identity(&mut self) -> Result<(u8, u8), DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let accel_id = self.who_am_i_accel()?;
        let mag_id = self.who_am_i_mag()?;
        Ok((accel_id, mag_id)) // Return the IDs of the accelerometer and magnetometer
    }

    /// Initialize the LSM303AGR device with the provided configurations for both accelerometer and magnetometer.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok(())` - If the initialization is successful
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing
    ///     to the configuration registers.
    pub fn init(&mut self) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        // Initialize the accelerometer with the provided configuration
        let ctrl_reg1_a = self.accel_config.get_ctrl_reg1_bits();
        let ctrl_reg4_a = self.accel_config.get_ctrl_reg4_bits();
        self.write_accel(ACCEL_CTRL_REG1, ctrl_reg1_a)?;
        self.write_accel(ACCEL_CTRL_REG4, ctrl_reg4_a)?;

        // Initialize the magnetometer with the provided configuration
        let cfg_reg_a_m = self.mag_config.cfg_reg_a_m_bits();
        self.write_mag(MAG_CFG_REG_A, cfg_reg_a_m)?;

        Ok(())
    }

    /// Read raw accelerometer data from the LSM303AGR device.
    ///
    /// Description
    ///     Read raw accelerometer data (6 bytes: X_L, X_H, Y_L, Y_H, Z_L, Z_H)
    ///                                          0x28|0x29|0x2A|0x2B|0x2C|0x2D
    ///     Uses auto-increment feature to read all axes in one I2C transaction.
    ///      28 +1 -> 29, 2A +1 -> 2B, 2C +1 -> 2D
    ///     todo verify if the auto-increment works
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok((i16, i16, i16))` - A tuple containing the raw X, Y, and Z accelerometer readings.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while reading
    ///     the accelerometer data registers.
    pub fn read_accel_raw(&mut self) -> Result<(i16, i16, i16), DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let mut buffer = [0u8; 6];
        // ✅ Enable auto-increment with bit 7
        self.i2c
            .write_read(ACCEL_ADDRESS, &[ACCEL_OUT_X_L | 0x80], &mut buffer)?;
        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);
        Ok((x, y, z))
    }
    /// Read accelerometer data in g (gravity) units from the LSM303AGR device.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok((f32, f32, f32))` - A tuple containing the X, Y, and Z accelerometer readings
    ///     in g units.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while reading
    ///     the accelerometer data registers.
    pub fn read_accel_g(&mut self) -> Result<(f32, f32, f32), DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let (x_raw, y_raw, z_raw) = self.read_accel_raw()?;
        let scale_factor = self.accel_config.scale_factor_mg();
        Ok((
            x_raw as f32 * scale_factor,
            y_raw as f32 * scale_factor,
            z_raw as f32 * scale_factor,
        ))
    }
    /// Read raw magnetometer data from the LSM303AGR device.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok((i16, i16, i16))` - A tuple containing the raw X, Y, and Z magnetometer readings.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while reading
    ///     the magnetometer data registers.
    pub fn read_mag_raw(&mut self) -> Result<(i16, i16, i16), DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let mut buffer = [0u8; 6];
        // ✅ Enable auto-increment with bit 7
        self.i2c
            .write_read(MAG_ADDRESS, &[MAG_OUT_X_L | 0x80], &mut buffer)?;
        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);
        Ok((x, y, z))
    }

    /// Read magnetometer data in gauss units from the LSM303AGR device.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok((f32, f32, f32))` - A tuple containing the X, Y, and Z magnetometer readings in gauss units.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while reading the
    ///         magnetometer data registers.
    pub fn read_mag_gauss(&mut self) -> Result<(f32, f32, f32), DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let sensitivity = MAG_SENSITIVITY; // mgauss/LSB for ±50 gauss
        let (x_raw, y_raw, z_raw) = self.read_mag_raw()?;
        Ok((
            x_raw as f32 * sensitivity,
            y_raw as f32 * sensitivity,
            z_raw as f32 * sensitivity,
        ))
    }

    /// Set the accelerometer range and update the configuration register.
    /// CTRL_REG4_A FULL_SCALE bits [5:4]
    ///
    /// Description
    ///   G2 = ±2g, G4 = ±4g, G8 = ±8g, G16 = ±16g
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// * `range` - The desired accelerometer range (e.g., ±2g, ±4g, ±8g)
    /// Returns:
    /// * `Ok(())` - If the range is successfully set
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing to
    ///     the configuration register.
    pub fn set_accel_range(&mut self, range: AccelRange) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        self.accel_config.range = range;
        let ctrl_reg4_a = self.accel_config.range.ctrl_reg4_bits();
        self.write_accel(ACCEL_CTRL_REG4, ctrl_reg4_a)?;
        Ok(())
    }
    /// Set the accelerometer data rate and update the configuration register. (ODR)
    /// CTRL_REG1_A ODR bits.
    ///
    /// Description
    ///     PowerDown, Hz1, Hz10, Hz25, Hz50, Hz100,
    ///     Hz200, Hz400, Hz1620(lower power), Hz5376 (lower power)
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver.
    /// * `data_rate` - The desired accelerometer data rate (e.g., 10Hz, 50Hz, 100Hz).
    /// Returns:
    /// * `Ok(())` - If the data rate is successfully set.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing to
    ///     the configuration register.
    pub fn set_accel_data_rate(
        &mut self,
        data_rate: AccelDataRate,
    ) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        self.accel_config.data_rate = data_rate;
        let ctrl_reg1_a = self.accel_config.data_rate.ctrl_reg1_bits();
        self.write_accel(ACCEL_CTRL_REG1, ctrl_reg1_a)?;
        Ok(())
    }
    /// Set the accelerometer operating mode and update the configuration registers.
    /// CTRL_REG1_A and CTRL_REG4_A bits
    ///
    /// Description
    ///   HighResolution, Normal, LowPower
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// * `mode` - The desired accelerometer operating mode (e.g., Normal, Low Power)
    /// Returns:
    /// * `Ok(())` - If the mode is successfully set
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing to
    ///     the configuration registers.
    pub fn set_accel_mode(
        &mut self,
        mode: AccelOperatingMode,
    ) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        self.accel_config.mode = mode;
        let (ctrl_reg1_a, ctrl_reg4_a) = self.accel_config.mode.ctrl_reg_bits();
        self.write_accel(ACCEL_CTRL_REG1, ctrl_reg1_a)?;
        self.write_accel(ACCEL_CTRL_REG4, ctrl_reg4_a)?;
        Ok(())
    }

    /// Set the magnetometer data rate and update the configuration register. (ODR)
    /// CFG_REG_A_M ODR bits [3:2]
    ///
    /// Description
    ///    Hz10, Hz20, Hz50, Hz100
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// * `data_rate` - The desired magnetometer data rate (e.g., 10Hz, 50Hz, 100Hz)
    /// Returns:
    /// * `Ok(())` - If the data rate is successfully set
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing to
    ///      the configuration register.
    pub fn set_mag_data_rate(
        &mut self,
        data_rate: MagDataRate,
    ) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        self.mag_config.data_rate = data_rate;
        let cfg_reg_a_m = self.mag_config.data_rate.cfg_reg_a_m_odr_bits();
        self.write_mag(MAG_CFG_REG_A, cfg_reg_a_m)?;
        Ok(())
    }

    /// Set the magnetometer operating mode and update the configuration register. (MD bits)
    /// CFG_REG_A_M MD bits [1:0]
    ///
    /// Description
    ///   Continuous, Single, Idle
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// * `mode` - The desired magnetometer operating mode (e.g., Continuous,
    /// Returns:
    /// * `Ok(())` - If the mode is successfully set
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing to
    ///  the configuration register.
    pub fn set_mag_operating_mode(
        &mut self,
        mode: MagOperatingMode,
    ) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        self.mag_config.operating_mode = mode;
        let cfg_reg_a_m = self.mag_config.operating_mode.cfg_reg_a_m_md_bits();
        self.write_mag(MAG_CFG_REG_A, cfg_reg_a_m)?;
        Ok(())
    }

    /// Set the magnetometer power mode and update the configuration register. (Power Mode bits)
    /// CFG_REG_A_M LP bits 3
    /// Description
    ///   HighResolution, LowPower
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// * `mode` - The desired magnetometer power mode (e.g., HighResolution, LowPower)
    /// Returns:
    /// * `Ok(())` - If the power mode is successfully set
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while writing to
    ///      the configuration register.
    pub fn set_mag_power_mode(&mut self, mode: MagPowerMode) -> Result<(), DriverError<I2C::Error>>
    where
        I2C: Write,
    {
        self.mag_config.power_mode = mode;
        let cfg_reg_a_m = self.mag_config.power_mode.cfg_reg_a_m_power_bits();
        self.write_mag(MAG_CFG_REG_A, cfg_reg_a_m)?;
        Ok(())
    }
}

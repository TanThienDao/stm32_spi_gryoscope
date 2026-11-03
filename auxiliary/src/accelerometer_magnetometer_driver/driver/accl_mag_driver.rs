//! =====================================================================
//! LSM303ARG Accelerometer and Magnetometer Driver
//!
//! Generic over I2C peripheral type to work with any HAL.
//!
//! # Type Parameters
//! * `I2C` - The I2C peripheral type implementing the `embedded_hal::blocking::i2c::WriteRead` and `embedded_hal::blocking::i2c::Write` traits.
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use crate::accelerometer_magnetometer_driver::driver::accel_config::AccelRange;
use crate::accelerometer_magnetometer_driver::driver::mag_config::{MagOperatingMode, MagRange};
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
    accel_range: AccelRange, // Store for scaling factor accelerometer range selection
    mag_range: MagRange,     // Store for scaling factor magnetometer range selection
}

impl<I2C> Lsm303agr<I2C> {
    /// Create a new instance of the LSM303AGR driver.
    ///
    /// # Arguments
    /// * `i2c` - The I2C peripheral implementing the required traits.
    /// * `accel_range` - The desired accelerometer range selection.
    /// * `mag_range` - The desired magnetometer range selection.
    ///
    /// # Returns
    /// A new instance of the LSM303AGR driver.
    pub fn new(
        i2c: I2C,
        // Accelerometer configuration
        accel_range: AccelRange,
        // Magnetometer configuration
        mag_range: MagRange,
    ) -> Self {
        Lsm303agr {
            i2c,
            accel_range,
            mag_range,
        }
    }

    /// Write to an accelerometer register.
    ///
    ///   Description: In I2C communication, writing to a register typically involves sending
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
        self.i2c
            .write(ACCEL_ADDRESS, &payload)?;
        Ok(())
    }

    /// Read from an accelerometer register.
    ///
    ///     Description: In I2C communication, reading from a register typically
    ///         involves first writing the register address to the device,
    ///         followed by reading the data from that register.
    ///         This method encapsulates that process for the accelerometer.
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
        self.i2c
            .write_read(ACCEL_ADDRESS, &[reg_address], buffer)?;
        Ok(buffer[0])
    }

    /// Write to a magnetometer register.
    ///
    ///  Description: Similar above, but for the magnetometer.
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
        self.i2c
            .write(MAG_ADDRESS, &payload)?;

        Ok(())
    }

    /// Read from a magnetometer register.
    ///
    /// Description: Similar above, but for the magnetometer.
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
        self.i2c
            .write_read(MAG_ADDRESS, &[reg_address], buffer)?;

        Ok(buffer[0])
    }
    /// Read the WHO_AM_I_A register of the accelerometer to verify its identity.
    ///
    /// Description: The WHO_AM_I register is a standard register in many sensors that contains
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
    /// Description: Similar to the accelerometer, this method reads the WHO_AM_I register
    ///  from the magnetometer and returns its value.
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
    /// Description: This method reads the WHO_AM_I registers of both the accelerometer and
    /// magnetometer, compares the returned values to the expected values, and returns a tuple
    /// indicating whether each device's identity is verified.
    ///
    /// Arguments:
    /// * `&mut self` - The mutable reference to the LSM303AGR driver
    /// Returns:
    /// * `Ok((bool, bool))` - A tuple where the first element indicates whether the accelerometer
    ///      identity is verified, and the second element indicates whether the magnetometer identity
    ///      is verified.
    /// * `Err(DriverError<I2C::Error>)` - If there is an I2C communication error while reading
    ///      either WHO_AM_I register.
    ///
    pub fn verify_device_identity(&mut self) -> Result<(bool, bool), DriverError<I2C::Error>>
    where
        I2C: WriteRead,
    {
        let accel_id = self.who_am_i_accel()?;
        let mag_id = self.who_am_i_mag()?;
        Ok((accel_id == 0x33, mag_id == 0x40)) // Return true if IDs match expected values
    }

    // todo implement the init and set range after here !!!
}

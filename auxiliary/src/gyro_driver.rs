//! Custom I3G4250D Driver
//!
//! This module provides direct SPI control over the I3G4250D gyroscope.
//! Use this for learning, debugging, or when you need precise control over registers.
//!
//! # Example
//!
//! ```ignore
//! let mut gyro = GyroDriver::new(spi, cs);
//! gyro.init()?;
//! gyro.set_data_rate(DataRate::Hz380)?;
//! gyro.set_range(Range::DPS500)?;
//!
//! loop {
//!     let (x, y, z) = gyro.read_angular_velocity()?;
//!     iprintln!(&mut itm.stim[0], "X: {:.2}°/s, Y: {:.2}°/s, Z: {:.2}°/s", x, y, z);
//! }
//! ```

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

// =====================================================================
// REGISTER ADDRESS MAP
// =====================================================================

/// WHO_AM_I register (0x0F) - Returns 0xD3 for I3G4250D
pub const WHO_AM_I: u8 = 0x0F;

/// CTRL_REG1 (0x20) - Power mode and data rate configuration
/// Bit 7-6: DR (Data Rate): 00=100Hz, 01=200Hz, 10=400Hz, 11=800Hz
/// Bit 5-4: BW (Bandwidth): Depends on DR
/// Bit 3:   PD (Power Down): 0=Powered down, 1=Normal/Sleep mode
/// Bit 2-0: Xen, Yen, Zen (Axis enable): 1=Enable, 0=Disable
pub const CTRL_REG1: u8 = 0x20;

/// CTRL_REG2 (0x21) - High-pass filter configuration
pub const CTRL_REG2: u8 = 0x21;

/// CTRL_REG3 (0x22) - Interrupt configuration
pub const CTRL_REG3: u8 = 0x22;

/// CTRL_REG4 (0x23) - Scale/sensitivity configuration
/// Bit 5-4: FS (Full Scale): 00=250°/s, 01=500°/s, 10=1000°/s, 11=2000°/s
/// Bit 6:   BLE (Big Endian): 0=Little Endian, 1=Big Endian
pub const CTRL_REG4: u8 = 0x23;

/// CTRL_REG5 (0x24) - FIFO and SPI mode
pub const CTRL_REG5: u8 = 0x24;

/// Temperature register (0x26) - Temperature data (optional)
pub const TEMP: u8 = 0x26;

/// Angular velocity output data registers
pub const OUT_X_L: u8 = 0x28; // X-axis low byte
pub const OUT_X_H: u8 = 0x29; // X-axis high byte
pub const OUT_Y_L: u8 = 0x2A; // Y-axis low byte
pub const OUT_Y_H: u8 = 0x2B; // Y-axis high byte
pub const OUT_Z_L: u8 = 0x2C; // Z-axis low byte
pub const OUT_Z_H: u8 = 0x2D; // Z-axis high byte

// =====================================================================
// DATA RATE OPTIONS (CTRL_REG1 bits 7-6)
// =====================================================================

pub const DR_100_HZ: u8 = 0b00 << 6;   // 100 Hz
pub const DR_200_HZ: u8 = 0b01 << 6;  // 200 Hz
pub const DR_400_HZ: u8 = 0b10 << 6;  // 400 Hz
pub const DR_800_HZ: u8 = 0b11 << 6;  // 800 Hz

// =====================================================================
// FULL SCALE RANGE OPTIONS (CTRL_REG4 bits 5-4)
// =====================================================================

pub const FS_245_DPS: u8    = 0b00 << 4;    // 245 °/s,  scale: 8.75 mDPS/LSB
pub const FS_500_DPS: u8    = 0b01 << 4;    // 500 °/s,  scale: 17.5 mDPS/LSB
pub const FS_2000_DPS_1: u8 = 0b10 << 4;    // 2000 °/s, scale: 70 mDPS/LSB
pub const FS_2000_DPS_2: u8 = 0b11 << 4;    // 2000 °/s, scale: 245 mDPS/LSB

// =====================================================================
// GYROSCOPE DRIVER STRUCT
// =====================================================================

/// I3G4250D Custom Driver
///
/// Generic over SPI peripheral and CS pin types to work with any HAL.
///
/// # Type Parameters
/// - `SPI`: SPI peripheral implementing `Transfer<u8>` trait
/// - `CS`: Chip Select pin implementing `OutputPin` trait
pub struct GyroDriver<SPI, CS> {
    spi: SPI,
    cs: CS,
    range: Range,  // Store for scaling factor
}

/// Full-scale range selection and scaling factor
#[derive(Clone, Copy)]
pub enum Range {
    /// 245 °/s, scale factor: 8.75 mDPS/LSB
    DPS245,
    /// 500 °/s, scale factor: 17.5 mDPS/LSB
    DPS500,
    /// 2000 °/s, scale factor: 70 mDPS/LSB
    DPS2000_1,
    /// 2000 °/s, scale factor: 70 mDPS/LSB
    DPS2000_2,

}

impl Range {
    /// Get the register value for CTRL_REG4 bits [5:4]
    pub fn ctrl_reg4_bits(&self) -> u8 {
        match self {
            Range::DPS245 => FS_245_DPS,
            Range::DPS500 => FS_500_DPS,
            Range::DPS2000_1 => FS_2000_DPS_1,
            Range::DPS2000_2 => FS_2000_DPS_2,
        }
    }

    /// Get the scale factor in millidegrees per second per LSB (mDPS/LSB)
    pub fn scale_factor_mdps(&self) -> f32 {
        match self {
            Range::DPS245 => 8.75,
            Range::DPS500 => 17.5,
            Range::DPS2000_1 => 70.0,
            Range::DPS2000_2 => 70.0,
        }
    }
}

/// Data rate selection
#[derive(Clone, Copy)]
pub enum DataRate {
    /// 100 Hz output rate
    Hz100,
    /// 200 Hz output rate
    Hz200,
    /// 400 Hz output rate (recommended for most applications)
    Hz400,
    /// 800 Hz output rate (high speed, high power)
    Hz800,
}

impl DataRate {
    /// Get the register value for CTRL_REG1 bits [7:6]
    pub fn ctrl_reg1_bits(&self) -> u8 {
        match self {
            DataRate::Hz100 => DR_100_HZ,
            DataRate::Hz200 => DR_200_HZ,
            DataRate::Hz400 => DR_400_HZ,
            DataRate::Hz800 => DR_800_HZ,
        }
    }
}

// =====================================================================
// DRIVER IMPLEMENTATION
// =====================================================================

impl<SPI, CS, E> GyroDriver<SPI, CS>
where
    SPI: Transfer<u8, Error = E>,       // SPI with unit error type
    CS: OutputPin,                      // CS pin
{
    /// Create a new gyroscope driver instance
    ///
    /// # Arguments
    /// * `spi` - SPI peripheral instance
    /// * `cs` - Chip Select output pin
    ///
    /// # Returns
    /// New GyroDriver with default range (DPS245)
    pub fn new(spi: SPI, cs: CS) -> Self {
        GyroDriver {
            spi,
            cs,
            range: Range::DPS245,  // Default range
        }
    }

    /// Read a single register via SPI
    ///
    /// Protocol: Send [address | 0x80 (read bit), dummy_byte], receive [address_echo, value]
    ///
    /// # Arguments
    /// * `reg_addr` - Register address to read
    ///
    /// # Returns
    /// Register value or error
    fn read_register(&mut self, reg_addr: u8) -> Result<u8, &'static str> {
        let mut buffer = [reg_addr | 0x80, 0x00];  // Set read bit (MSB = 1)

        self.cs.set_low().ok();
        self.spi.transfer(&mut buffer)
            .map_err(|_| "SPI transfer failed")?;
        self.cs.set_high().ok();

        Ok(buffer[1])  // Return received byte
    }

    /// Write a single register via SPI
    ///
    /// Protocol: Send [address (read bit = 0), value], receive [address_echo, status]
    ///
    /// # Arguments
    /// * `reg_addr` - Register address to write
    /// * `value` - Value to write
    ///
    /// # Returns
    /// Ok(()) or error
    fn write_register(&mut self, reg_addr: u8, value: u8) -> Result<(), &'static str> {
        let mut buffer = [reg_addr & 0x7F, value];  // Clear read bit (MSB = 0)

        self.cs.set_low().ok();
        self.spi.transfer(&mut buffer)
            .map_err(|_| "SPI transfer failed")?;
        self.cs.set_high().ok();

        Ok(())
    }

    /// Read WHO_AM_I register (should return 0xD3 for I3G4250D)
    ///
    /// # Returns
    /// WHO_AM_I value or error
    pub fn who_am_i(&mut self) -> Result<u8, &'static str> {
        self.read_register(WHO_AM_I)
    }

    /// Initialize gyroscope with power-on and axis enable
    ///
    /// Sets CTRL_REG1 to enable all three axes and normal mode.
    /// Default: 95 Hz, all axes enabled.
    ///
    /// # Returns
    /// Ok(()) or error
    pub fn init(&mut self) -> Result<(), &'static str> {
        // CTRL_REG1: Power on (PD=1) + all axes enabled (Xen=Yen=Zen=1)
        // DR=400Hz (bits 7-6 = 10), BW bits 5-4 = 11
        // 400 Hz(ODR), Cutoff 110, all axes, normal mode 0b10111111
        //let ctrl_reg1 = 0xBF;
        let ctrl_reg1 = 0x0F;  // 0b00001111
        // Bit 3 (PD) = 1   → Normal mode
        // Bits 2-0 = 111   → All axes enabled
        // Bits 7-6 = 00    → 100 Hz (placeholder, will be set by set_data_rate())
        // Bits 5-4 = 00    → Low BW (placeholder)
        self.write_register(CTRL_REG1, ctrl_reg1)?;

        // CTRL_REG2: Disable high-pass filter
        self.write_register(CTRL_REG2, 0x00)?;

        // CTRL_REG3: Disable interrupts
        self.write_register(CTRL_REG3, 0x00)?;

        // Also initialize CTRL_REG4 to little-endian, 245 DPS, 8.75 mDPS/LSB
        let ctrl4 = 0x00;      // 0b00000000
        // Bit 6 (BLE) = 0  → Little-endian
        // Bits 5-4 = 00    → 245 DPS (placeholder, will be set by set_range())
        self.write_register(CTRL_REG4, ctrl4)?;

        // CTRL_REG5: FIFO disabled, normal SPI
        self.write_register(CTRL_REG5, 0x00)?;


        Ok(())
    }

    /// Configure data rate
    /// CTRL_REG1 bits [7:6] control the output data rate (ODR) and bandwidth.
    ///
    /// # Arguments
    /// * `rate` - Desired data rate (Hz100, Hz250, Hz400, or Hz800)
    ///
    /// # Returns
    /// Ok(()) or error
    pub fn set_data_rate(&mut self, rate: DataRate) -> Result<(), &'static str> {
        let current = self.read_register(CTRL_REG1)?;
        let new_val = (current & 0x3F) | rate.ctrl_reg1_bits();  // Preserve bits [5:0]
        self.write_register(CTRL_REG1, new_val)
    }

    /// Configure full-scale range (and store for scaling)
    /// CTRL_REG4 bits [5:4] control the full-scale range (FS).
    ///
    /// # Arguments
    /// * `range` - Desired full-scale range (DPS250, DPS500, DPS1000, or DPS2000)
    ///
    /// # Returns
    /// Ok(()) or error
    pub fn set_range(&mut self, range: Range) -> Result<(), &'static str> {
        let current = self.read_register(CTRL_REG4)?;
        let new_val = (current & 0xCF) | range.ctrl_reg4_bits();  //0xCF 0b11001111 (keep bits 6,3-0)
        self.write_register(CTRL_REG4, new_val)?;
        self.range = range;
        Ok(())
    }

    /// Read 6 data bytes from output registers (X_L, X_H, Y_L, Y_H, Z_L, Z_H)
    ///
    /// Uses auto-increment feature to read all axes in one SPI transaction.
    /// Returns raw (x, y, z) values as signed 16-bit integers (little-endian)
    ///
    /// # Returns
    /// Tuple of (x_raw, y_raw, z_raw) or error
    pub fn read_raw_data(&mut self) -> Result<(i16, i16, i16), &'static str> {
        // Auto-increment read starting from OUT_X_L (0x28)
        // Buffer: [command_byte, x_l, x_h, y_l, y_h, z_l, z_h]
        let mut buffer = [
            OUT_X_L | 0xC0,  // buffer[0]: Command (0x28 | 0xC0 = 0xE8)
            0,               // buffer[1]: Will receive X_L
            0,               // buffer[2]: Will receive X_H
            0,               // buffer[3]: Will receive Y_L
            0,               // buffer[4]: Will receive Y_H
            0,               // buffer[5]: Will receive Z_L
            0,               // buffer[6]: Will receive Z_H
        ];

        self.cs.set_low().ok();
        self.spi.transfer(&mut buffer)
            .map_err(|_| "SPI transfer failed")?;
        self.cs.set_high().ok();

        // Extract and combine low/high bytes (little-endian)
        let x = i16::from_le_bytes([buffer[1], buffer[2]]);
        let y = i16::from_le_bytes([buffer[3], buffer[4]]);
        let z = i16::from_le_bytes([buffer[5], buffer[6]]);

        Ok((x, y, z))
    }

    /// Read angular velocity in degrees per second (°/s)
    ///
    /// Reads raw data and applies the configured scale factor.
    ///
    /// Formula: DPS = (raw_value * scale_factor) / 1000
    /// where scale_factor is in mDPS/LSB (see Range enum)
    ///
    /// # Returns
    /// Tuple of (x_dps, y_dps, z_dps) or error
    pub fn read_angular_velocity(&mut self) -> Result<(f32, f32, f32), &'static str> {
        let (x_raw, y_raw, z_raw) = self.read_raw_data()?;
        let scale = self.range.scale_factor_mdps();

        // Convert raw counts to degrees per second
        let x_dps = (x_raw as f32 * scale) / 1000.0;
        let y_dps = (y_raw as f32 * scale) / 1000.0;
        let z_dps = (z_raw as f32 * scale) / 1000.0;

        Ok((x_dps, y_dps, z_dps))
    }

    /// Read temperature (optional feature)
    ///
    /// Returns temperature in °C. Accuracy is limited (~10°C).
    ///
    /// # Returns
    /// Temperature value or error
    pub fn read_temperature(&mut self) -> Result<i8, &'static str> {
        let temp = self.read_register(TEMP)? as i8;
        Ok(temp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_scale_factors() {
        assert_eq!(Range::DPS245.scale_factor_mdps(), 8.75);
        assert_eq!(Range::DPS500.scale_factor_mdps(), 17.5);
        assert_eq!(Range::DPS2000_1.scale_factor_mdps(), 70.0);
        assert_eq!(Range::DPS2000_2.scale_factor_mdps(), 70.0);
    }

    #[test]
    fn test_register_bits() {
        // Test DR (data rate) bits
        assert_eq!(DataRate::Hz100.ctrl_reg1_bits(), 0b00 << 6);
        assert_eq!(DataRate::Hz200.ctrl_reg1_bits(), 0b01 << 6);
        assert_eq!(DataRate::Hz400.ctrl_reg1_bits(), 0b10 << 6);
        assert_eq!(DataRate::Hz800.ctrl_reg1_bits(), 0b11 << 6);

        // Test FS (full scale) bits
        assert_eq!(Range::DPS245.ctrl_reg4_bits(), 0b00 << 4);
        assert_eq!(Range::DPS500.ctrl_reg4_bits(), 0b01 << 4);
        assert_eq!(Range::DPS2000_1.ctrl_reg4_bits(), 0b10 << 4);
        assert_eq!(Range::DPS2000_2.ctrl_reg4_bits(), 0b11 << 4);
    }
}


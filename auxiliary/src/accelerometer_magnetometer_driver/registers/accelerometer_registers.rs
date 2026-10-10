//! =====================================================================
//! REGISTER ADDRESS MAP
//! This file contains the register address map for the LSM303AGR accelerometer.
//! =====================================================================

// =====================================================================
// I2C SLAVE ADDRESSES
// =====================================================================

/// Section 6.1.1 of LSM303AGR datasheet specifies I2C slave addresses for accelerometer and magnetometer.
/// SAD example: 7-bit address, left-shifted for R/W bit (slave address is 0x19 for accelerometer)
/// Accelerometer I2C slave address (depends on SA0 pin)
pub const ACCEL_ADDRESS: u8 = 0x19;  // SA0 = 0 (default)

// =====================================================================
// ACCELEROMETER REGISTERS (LSM303AGR)
// =====================================================================

/// WHO_AM_I register (0x0F) - Read-only, returns 0x33 for LSM303AGR
pub const ACCEL_WHO_AM_I: u8 = 0x0F;

/// Control Registers
pub const ACCEL_CTRL_REG1: u8 = 0x20;  // Power mode, data rate, axis enable
pub const ACCEL_CTRL_REG2: u8 = 0x21;  // High-pass filter
pub const ACCEL_CTRL_REG3: u8 = 0x22;  // Interrupt configuration
pub const ACCEL_CTRL_REG4: u8 = 0x23;  // Full-scale range, SPI mode
pub const ACCEL_CTRL_REG5: u8 = 0x24;  // FIFO, axis reboot
pub const ACCEL_CTRL_REG6: u8 = 0x25;  // Click interrupt

/// Status Register
pub const ACCEL_STATUS_REG: u8 = 0x27;  // Data ready flags


/// Temperature Config register
// Note: to enable the temperature sensor set TEMP_EN[1:0] = 11
pub const ACCEL_TEMP_CFG_REG: u8 = 0x1F; // Temperature sensor enable
pub const ACCEL_TEMP_L: u8 = 0x0C;     // Temperature low byte
pub const ACCEL_TEMP_H: u8 = 0x0D;     // Temperature high byte

/// Output Data Registers
pub const ACCEL_OUT_X_L: u8 = 0x28;    // X-axis low byte
pub const ACCEL_OUT_X_H: u8 = 0x29;    // X-axis high byte
pub const ACCEL_OUT_Y_L: u8 = 0x2A;    // Y-axis low byte
pub const ACCEL_OUT_Y_H: u8 = 0x2B;    // Y-axis high byte
pub const ACCEL_OUT_Z_L: u8 = 0x2C;    // Z-axis low byte
pub const ACCEL_OUT_Z_H: u8 = 0x2D;    // Z-axis high byte

// =====================================================================
// ACCEL_CTRL_REG1 BIT DEFINITIONS
// =====================================================================
/// Bit: 7        6      5        4     3       2      1       0
///     ODR3    ODR2    ODR1    ODR0   LPen    Zen    Yen     Xen

/// ACCEL_CTRL_REG1 Data Rate Selection (bits 7-4)
pub const ACCEL_ODR_POWER_DOWN: u8 = 0x00 << 4;                 // Power-down mode
pub const ACCEL_ODR_1_HZ: u8 = 0x01 << 4;                       // 1 Hz
pub const ACCEL_ODR_10_HZ: u8 = 0x02 << 4;                      // 10 Hz
pub const ACCEL_ODR_25_HZ: u8 = 0x03 << 4;                      // 25 Hz
pub const ACCEL_ODR_50_HZ: u8 = 0x04 << 4;                      // 50 Hz
pub const ACCEL_ODR_100_HZ: u8 = 0x05 << 4;                     // 100 Hz
pub const ACCEL_ODR_200_HZ: u8 = 0x06 << 4;                     // 200 Hz
pub const ACCEL_ODR_400_HZ: u8 = 0x07 << 4;                     // 400 Hz
pub const ACCEL_ODR_LOW_POWER_1_6_KHZ: u8 = 0x08 << 4;          // Low-power mode 1.620 kHz
pub const ACCEL_ODR_LOW_POWER_5_376_KHZ: u8 = 0x09 << 4;        // HR/Normal (1.344KHz) | Low-power mode 5.376 kHz

/// Low Power Mode Enable (bit 3)
pub const ACCEL_LP_ENABLE: u8 = 0x01 << 3;                      // Low-power mode enable (8 bit data output)

/// ACCEL_CTRL_REG1 Axis Enable (bits 2-0)
pub const ACCEL_X_ENABLE: u8 = 0x01 << 0;                        // X-axis enable
pub const ACCEL_Y_ENABLE: u8 = 0x01 << 1;                        // Y-axis enable
pub const ACCEL_Z_ENABLE: u8 = 0x01 << 2;                        // Z-axis enable
pub const ACCEL_ALL_AXES: u8 = 0x07;                             // All axes enable

// =====================================================================
// ACCEL_CTRL_REG2 BIT DEFINITIONS
// =====================================================================
/// Bit:    7        6       5        4       3       2       1       0
///        HPM1    HPM0    HPCF2    HPCF1   FDS    HPCLICK  HPIS2   HPIS1

// =====================================================================
// ACCEL_CTRL_REG3 BIT DEFINITIONS
// =====================================================================
/// Bit:    7            6       5        4          3       2          1         0
///        I1_CLICK  I1_AOI1  I1_AOI2  I1_DRDY1  I1_DRDY2  I1_WTM  I1_OVERRUN     _


// =====================================================================
// ACCEL_CTRL_REG4 BIT DEFINITIONS
// =====================================================================
/// Bit:   7       6       5        4       3       2       1       0
///       BDU     BLE     FS1      FS0     HR      ST1     ST0     SIM

/// Block Data Update (bit 7)
pub const ACCEL_BDU_ENABLE: u8 = 0x01 << 7;

/// Big/Little Endian Selection (bit 6)
pub const ACCEL_BLE_LITTLE_ENDIAN: u8 = 0x00 << 6;  // Data LSB @ lower address
pub const ACCEL_BLE_BIG_ENDIAN: u8 = 0x01 << 6;     // Data MSB @ lower address

/// Full-Scale Range (bits 5-4)
pub const ACCEL_FS_2G: u8 = 0x00 << 4;              // ±2g, ~1 mg/LSB
pub const ACCEL_FS_4G: u8 = 0x01 << 4;              // ±4g, ~2 mg/LSB
pub const ACCEL_FS_8G: u8 = 0x02 << 4;              // ±8g, ~4 mg/LSB
pub const ACCEL_FS_16G: u8 = 0x03 << 4;             // ±16g, ~12 mg/LSB

/// High Resolution Mode (bit 3)
pub const ACCEL_HR_ENABLE: u8 = 0x01 << 3;          // High-resolution mode enable (12-bit data output)

/// Enable Self-Test (bits 2-1)
pub const ACCEL_SELF_TEST_DISABLE: u8 = 0x00 << 1;  // normal mode (self-test disabled)
pub const ACCEL_SELF_TEST_0: u8 = 0x01 << 1;        // Self-test 0
pub const ACCEL_SELF_TEST_1: u8 = 0x02 << 1;        // Self-test 1

/// SPI Serial Interface Mode Selection (bit 0), Enable 3-wire or 4-wire interface
pub const ACCEL_SPI_4_WIRE: u8 = 0x00 << 0;          // 3-wire interface disabled (default)
pub const ACCEL_SPI_3_WIRE: u8 = 0x01 << 0;          // 3-wire interface enabled

// =====================================================================
// ACCEL_CTRL_REG5 BIT DEFINITIONS
// =====================================================================
/// Bit:   7       6         5        4       3          2          1        0
///       BOOT    FIFO_EN   ---      ---    LIR_INT1  D4D_INT1  LIR_INT2  D4D_INT2

// =====================================================================
// ACCEL_CTRL_REG6 BIT DEFINITIONS
// =====================================================================
/// Bit:   7           6         5        4       3          2          1        0
///     I2_CLICKen  I2_INT1   I2_INT2   BOOT_I2  P2_ACT     ---      H_Lactive  ---

// =====================================================================
// STATUS_REG_A BIT DEFINITIONS
// =====================================================================
/// Bit:   7       6        5       4       3      2        1        0
///     ZYXOR     ZOR      YOR     XOR    ZYXDA   ZDA      YDA      XDA

pub const ACCEL_STATUS_ZYXOR: u8 = 0x01 << 7;  // Z, Y, X axis data overrun
pub const ACCEL_STATUS_ZOR: u8 = 0x01 << 6;  // Y, X axis data overrun
pub const ACCEL_STATUS_YOR: u8 = 0x01 << 5;  // Y axis data overrun
pub const ACCEL_STATUS_XOR: u8 = 0x01 << 4;  // X axis data overrun
pub const ACCEL_STATUS_ZYXDA: u8 = 0x01 << 3;  // Z, Y, X axis new data available
pub const ACCEL_STATUS_ZDA: u8 = 0x01 << 2;  // Z axis new data available
pub const ACCEL_STATUS_YDA: u8 = 0x01 << 1;  // Y axis new data available
pub const ACCEL_STATUS_XDA: u8 = 0x01 << 0;  // X axis new data available
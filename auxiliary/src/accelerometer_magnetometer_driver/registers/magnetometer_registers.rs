//! =====================================================================
//! REGISTER ADDRESS MAP
//! This file contains the register address map for the LSM303AGR magnetometer.
//! =====================================================================

/// Section 6.1.1 of LSM303AGR datasheet specifies I2C slave addresses for accelerometer and magnetometer.
/// SAD example: 7-bit address, left-shifted for R/W bit (0x1E for magnetometer)
/// Magnetometer I2C slave address
pub const MAG_ADDRESS: u8 = 0x1E;    // LSM303AGR magnetometer 0011110b

// =====================================================================
// MAGNETOMETER REGISTERS (LSM303AGR)
// =====================================================================

/// WHO_AM_I register (0x4F) - Read-only, returns 0x40 for LSM303AGR
pub const MAG_WHO_AM_I: u8 = 0x4F;

/// Control Registers
pub const MAG_CFG_REG_A: u8 = 0x60;    // Temperature sensor, output data rate, self-test
pub const MAG_CFG_REG_B: u8 = 0x61;    // Full-scale selection, reboot
pub const MAG_CFG_REG_C: u8 = 0x62;    // Mode selection, SPI, low-power, I2C_DIS inhibit
pub const MAG_INT_CTRL_REG: u8 = 0x63;    //Interrupt Control register, Data output format, endianness
pub const MAG_INT_SOURCE_REG: u8 = 0x64;    //Interrupt Control register, Fast read, block data update

/// Status Register
pub const MAG_STATUS_REG: u8 = 0x67;   // Data ready, overrun flags

/// Output Data Registers
pub const MAG_OUT_X_L: u8 = 0x68;      // X-axis low byte
pub const MAG_OUT_X_H: u8 = 0x69;      // X-axis high byte
pub const MAG_OUT_Y_L: u8 = 0x6A;      // Y-axis low byte
pub const MAG_OUT_Y_H: u8 = 0x6B;      // Y-axis high byte
pub const MAG_OUT_Z_L: u8 = 0x6C;      // Z-axis low byte
pub const MAG_OUT_Z_H: u8 = 0x6D;      // Z-axis high byte
pub const MAG_OUT_T_L: u8 = 0x6E;      // Temperature low byte
pub const MAG_OUT_T_H: u8 = 0x6F;      // Temperature high byte


// =====================================================================
// MAG_CFG_REG_A BIT DEFINITIONS
// =====================================================================
/// Bit:       7          6         5       4     3      2      1     0
///      COMP_TEMP_EN   REBOOT   SOFT_RST   LP   ODR1   ODR0   MD1   MD0

/// Temperature Compensation Enable (bit 7)
pub const MAG_TEMP_ENABLE: u8 = 0x01 << 7;

/// Data Rate Selection (ODR bit [3:2])
pub const MAG_ODR_10_HZ: u8 = 0x00 << 2;                         // 10 Hz
pub const MAG_ODR_20_HZ: u8 = 0x01 << 2;                         // 20 Hz
pub const MAG_ODR_50_HZ: u8 = 0x02 << 2;                         // 50 Hz
pub const MAG_ODR_100_HZ: u8 = 0x03 << 2;                        // 100 Hz

/// Operating Mode (bits 1-0)
pub const MAG_MODE_CONTINUOUS: u8 = 0x00;                        // Continuous-conversion mode
pub const MAG_MODE_SINGLE: u8 = 0x01;                            // Single-conversion mode
pub const MAG_MODE_IDLE: u8 = 0x02;                              // Idle mode


// =====================================================================
// MAG_CFG_REG_B BIT DEFINITIONS
// =====================================================================
/// Bit:  7    6    5       4         3       2       1      0
///      [0]  [0]  [0]  OFF_CANC_  INT_on_   Set_    OFF_   LPF
///                     ONE_SHOT   DataOFF   FREQ    CANC

/// Offset Cancellation in Single Measurement Mode (bit 4)
pub const MAG_OFF_CANC_ONE_SHOT: u8 = 0x01 << 4;

/// Interrupt on Data Output (bit 3) (hard-iron correction)
pub const MAG_INT_ON_DATAOFF: u8 = 0x01 << 3;

/// Set Pulse Frequency (bit 2)
pub const MAG_SET_FREQ: u8 = 0x01 << 2;

/// Offset Cancellation Enable (bit 1)
pub const MAG_OFF_CANC: u8 = 0x01 << 1;

/// Low-Pass Filter Enable (bit 0)
pub const MAG_LPF_ENABLE: u8 = 0x01 << 0;

// =====================================================================
// CFG_REG_C_M BIT DEFINITIONS
// =====================================================================
/// Bit   7       6      5       4    3   2       1          0
///       0   INT_MAG  I2C_DIS  BDU  BLE  0   Self_test   INT_MAG
///            _PIN


// =====================================================================
// INT_CTRL_REG_M DEFINITIONS
// =====================================================================

/// Bit:     7       6      5       4      3       2       1         0
///         XIEN    YIEN   ZIEN    0(1)   0(1)    IEA     IEL       IEN

/// Enable Interrupt on X-axis (bit 7)
pub const MAG_XIEN: u8 = 0x01 << 7;


/// Enable Interrupt on Y-axis (bit 6)
pub const MAG_YIEN: u8 = 0x01 << 6;


/// Enable Interrupt on Z-axis (bit 5)
pub const MAG_ZIEN: u8 = 0x01 << 5;

/// Control the polarity of the interrupt signal (bit 2)
pub const MAG_IEA: u8 = 0x01 << 2;

/// Control the interrupt signal latching (bit 1)
pub const MAG_IEL: u8 = 0x01 << 1;

/// Enable Interrupt (bit 0)
pub const MAG_IEN: u8 = 0x01 << 0;



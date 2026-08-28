# I3G4250D Data Handling: Mathematics, Conversion, and Protocol

## Overview

This document explains:
1. **How data is stored** in I3G4250D output registers
2. **Mathematical conversion** from raw sensor values to degrees per second
3. **Register address mapping** for reading angular velocity
4. **SPI protocol details** for full-duplex transfers
5. **Worked examples** with actual calculations
6. **Endianness handling** and byte ordering

---

## 1. Register Map for Data Reading

### Output Data Registers

The I3G4250D outputs angular velocity as 16-bit signed integers in three pairs of registers:

| Axis | Low Byte Register | High Byte Register | Address | Purpose |
|------|-------------------|-------------------|---------|---------|
| **X** | OUT_X_L | OUT_X_H | 0x28, 0x29 | Roll (rotation around X-axis) |
| **Y** | OUT_Y_L | OUT_Y_H | 0x2A, 0x2B | Pitch (rotation around Y-axis) |
| **Z** | OUT_Z_L | OUT_Z_H | 0x2C, 0x2D | Yaw (rotation around Z-axis) |

### Reading Protocol

The registers are **read-only** and update automatically every sensor cycle:

```
┌─────────────────────────────────────────────────┐
│         I3G4250D Angular Velocity Data          │
│                                                 │
│  Cycle 1: OUT_X_L=0x45, OUT_X_H=0xAF (X-axis) │
│           OUT_Y_L=0x10, OUT_Y_H=0x02 (Y-axis) │
│           OUT_Z_L=0xF0, OUT_Z_H=0xFF (Z-axis) │
│                                                 │
│  Cycle 2: [New values updated by sensor]       │
│                                                 │
│  Cycle N: [Next values...]                     │
└─────────────────────────────────────────────────┘
```

---

## 2. Raw Data to DPS Conversion Mathematics

### Step-by-Step Conversion Process

#### Step 1: Read Raw 16-bit Signed Integer

The sensor stores angular velocity as a **16-bit signed integer** in two bytes:
- **Byte 0** (LSB - Least Significant Byte) — Lower 8 bits
- **Byte 1** (MSB - Most Significant Byte) — Upper 8 bits, includes sign bit

#### Step 2: Combine Bytes (Little-Endian)

In **little-endian format** (I3G4250D default):
```
Raw_Value = (MSB << 8) | LSB

Example:
  LSB = 0x45 (binary: 01000101)
  MSB = 0xAF (binary: 10101111)
  
  Raw_Value = (0xAF << 8) | 0x45
            = 0xAF45
            = 44869 (unsigned interpretation)
            = -20667 (signed interpretation, two's complement)
```

#### Step 3: Apply Scale Factor

Different full-scale ranges have different scale factors:

```
Scale_Factor (mDPS/LSB) based on FS configuration:
  FS=250 °/s:  8.75 mDPS/LSB
  FS=500 °/s:  17.5 mDPS/LSB
  FS=1000 °/s: 70 mDPS/LSB
  FS=2000 °/s: 245 mDPS/LSB
```

#### Step 4: Convert to Degrees Per Second

```rust
// Formula
Angular_Velocity_DPS = (Raw_Value × Scale_Factor) / 1000.0

// Explanation:
// - Raw_Value is a signed 16-bit integer (-32768 to +32767)
// - Scale_Factor is in mDPS (millidegrees per second)
// - Divide by 1000 to convert from mDPS to DPS
```

---

## 3. Complete Worked Examples

### Example 1: 250 °/s Range (Scale: 8.75 mDPS/LSB)

**Scenario:** Sensor configured for 250 °/s range, measuring slow rotation.

**Raw Data from Registers:**
```
OUT_X_L = 0x10  (LSB)
OUT_X_H = 0x00  (MSB)
```

**Conversion:**
```
Step 1: Combine bytes (little-endian)
  Raw_Value = (0x00 << 8) | 0x10
            = 0x0010
            = 16 (decimal)

Step 2: Apply scale factor
  Angular_Velocity = (16 × 8.75) / 1000
                   = 140 / 1000
                   = 0.14 °/s

Result: Sensor is rotating at 0.14°/s around X-axis
```

**Code:**
```rust
let lsb: u8 = 0x10;
let msb: u8 = 0x00;
let raw: i16 = i16::from_le_bytes([lsb, msb]);  // 16

const SCALE_250DPS: f32 = 8.75;  // mDPS/LSB
let dps = (raw as f32 * SCALE_250DPS) / 1000.0;  // 0.14
```

---

### Example 2: 500 °/s Range (Scale: 17.5 mDPS/LSB) — Faster Rotation

**Scenario:** Faster rotation measured in 500 °/s range.

**Raw Data from Registers:**
```
OUT_Y_L = 0x80  (LSB)
OUT_Y_H = 0x02  (MSB)
```

**Conversion:**
```
Step 1: Combine bytes (little-endian)
  Raw_Value = (0x02 << 8) | 0x80
            = 0x0280
            = 640 (decimal)

Step 2: Apply scale factor
  Angular_Velocity = (640 × 17.5) / 1000
                   = 11,200 / 1000
                   = 11.2 °/s

Result: Sensor is rotating at 11.2°/s around Y-axis
```

**Code:**
```rust
let lsb: u8 = 0x80;
let msb: u8 = 0x02;
let raw: i16 = i16::from_le_bytes([lsb, msb]);  // 640

const SCALE_500DPS: f32 = 17.5;  // mDPS/LSB
let dps = (raw as f32 * SCALE_500DPS) / 1000.0;  // 11.2
```

---

### Example 3: Negative Value (Reverse Rotation)

**Scenario:** Device rotating opposite direction (negative angular velocity).

**Raw Data from Registers:**
```
OUT_Z_L = 0xC0  (LSB)
OUT_Z_H = 0xFF  (MSB)
```

**Conversion:**
```
Step 1: Combine bytes (little-endian)
  Raw_Value = (0xFF << 8) | 0xC0
            = 0xFFC0
            = -64 (in signed 16-bit two's complement)

Step 2: Apply scale factor (500 °/s range)
  Angular_Velocity = (-64 × 17.5) / 1000
                   = -1,120 / 1000
                   = -11.2 °/s

Result: Sensor is rotating at -11.2°/s (reverse direction)
```

**Code:**
```rust
let lsb: u8 = 0xC0;
let msb: u8 = 0xFF;
let raw: i16 = i16::from_le_bytes([lsb, msb]);  // -64

const SCALE_500DPS: f32 = 17.5;
let dps = (raw as f32 * SCALE_500DPS) / 1000.0;  // -11.2
```

---

### Example 4: Maximum Positive Value

**Scenario:** Maximum measurable rotation in 500 °/s range.

**Raw Data:**
```
OUT_X_L = 0xFF
OUT_X_H = 0x7F  (0x7FFF = 32767, max positive signed 16-bit)
```

**Conversion:**
```
Raw_Value = 0x7FFF = 32767

Angular_Velocity = (32767 × 17.5) / 1000
                 = 573,422.5 / 1000
                 = 573.42 °/s

Note: Exceeds ±500 °/s range! But value is valid.
```

---

### Example 5: Maximum Negative Value

**Scenario:** Maximum reverse rotation in 500 °/s range.

**Raw Data:**
```
OUT_Y_L = 0x00
OUT_Y_H = 0x80  (0x8000 = -32768, min negative signed 16-bit)
```

**Conversion:**
```
Raw_Value = 0x8000 = -32768

Angular_Velocity = (-32768 × 17.5) / 1000
                 = -573,440 / 1000
                 = -573.44 °/s
```

---

## 4. SPI Protocol for Reading Data

### Multi-Byte Read (Auto-Increment)

The I3G4250D supports **auto-incrementing addresses** for fast multi-byte reads:

```
When you set bit 7 of the address to 1, subsequent bytes auto-increment the address.

Example: Reading all 6 data bytes (OUT_X_L through OUT_Z_H)
  Address = OUT_X_L | 0x80 = 0x28 | 0x80 = 0xA8  (read bit set)

  SPI Sequence:
    MOSI: [0xA8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]  (7 bytes sent)
    MISO: [0xA8, X_L,  X_H,  Y_L,  Y_H,  Z_L,  Z_H]   (7 bytes received)
    
    Data extracted:
      buffer[1:2] = [X_L, X_H]
      buffer[3:4] = [Y_L, Y_H]
      buffer[5:6] = [Z_L, Z_H]
```

### Single-Byte Read

For reading a single register:

```
Protocol:
  1. Set CS low
  2. Send [address | 0x80, dummy_byte]  (address with read bit set)
  3. Receive [address_echo, register_value]
  4. Set CS high
  
Example: Reading WHO_AM_I (0x0F)
  MOSI: [0x8F, 0x00]
  MISO: [0x8F, 0xD3]  ← 0xD3 is I3G4250D ID
```

---

## 5. Implementation in Custom Driver

### Raw Data Reading Function

```rust
fn read_raw_data(&mut self) -> Result<(i16, i16, i16), &'static str> {
    // Auto-increment read starting from OUT_X_L (0x28)
    let mut buffer = [0x28 | 0x80, 0, 0, 0, 0, 0, 0];  // 7 bytes total
    
    self.cs.set_low().ok();
    self.spi.transfer(&mut buffer)
        .map_err(|_| "SPI transfer failed")?;
    self.cs.set_high().ok();

    // Combine bytes using little-endian interpretation
    let x = i16::from_le_bytes([buffer[1], buffer[2]]);
    let y = i16::from_le_bytes([buffer[3], buffer[4]]);
    let z = i16::from_le_bytes([buffer[5], buffer[6]]);

    Ok((x, y, z))
}
```

### DPS Conversion Function

```rust
fn read_angular_velocity(&mut self) -> Result<(f32, f32, f32), &'static str> {
    let (x_raw, y_raw, z_raw) = self.read_raw_data()?;
    
    // Use previously configured scale factor (e.g., 17.5 for 500 DPS)
    let scale = self.range.scale_factor_mdps();  // 17.5 mDPS/LSB
    
    // Convert each axis
    let x_dps = (x_raw as f32 * scale) / 1000.0;
    let y_dps = (y_raw as f32 * scale) / 1000.0;
    let z_dps = (z_raw as f32 * scale) / 1000.0;

    Ok((x_dps, y_dps, z_dps))
}
```

---

## 6. Endianness Explained

### Little-Endian (I3G4250D Default)

```
Least Significant Byte comes FIRST in memory/SPI transfer.

Example: Value 0xABCD
  Little-Endian: [0xCD, 0xAB]  ← LSB first, MSB second
  
  When transferred via SPI:
    First byte received = 0xCD (LSB)
    Second byte received = 0xAB (MSB)
    
  Reconstruction in Rust:
    i16::from_le_bytes([0xCD, 0xAB]) = 0xABCD ✓
```

### Big-Endian (Alternative, Not Default)

```
Most Significant Byte comes FIRST.

Same value 0xABCD:
  Big-Endian: [0xAB, 0xCD]  ← MSB first, LSB second
  
  Reconstruction:
    i16::from_be_bytes([0xAB, 0xCD]) = 0xABCD ✓
```

### I3G4250D Configuration

```rust
// In CTRL_REG4, bit 6 (BLE flag) controls endianness:
// BLE = 0 (default): Little-Endian
// BLE = 1: Big-Endian

// Recommended: Keep BLE = 0 and use from_le_bytes()

const CTRL_REG4_LITTLE_ENDIAN: u8 = 0b00000000;  // Bit 6 = 0
```

---

## 7. Complete Example: Reading Loop

### Using Custom Driver

```rust
loop {
    // Read raw data
    let (x_raw, y_raw, z_raw) = gyro.read_raw_data()?;
    
    // Scale according to configured range (e.g., 500 °/s)
    const SCALE: f32 = 17.5;  // mDPS/LSB
    
    let x_dps = (x_raw as f32 * SCALE) / 1000.0;
    let y_dps = (y_raw as f32 * SCALE) / 1000.0;
    let z_dps = (z_raw as f32 * SCALE) / 1000.0;
    
    iprintln!(&mut itm.stim[0], 
        "X: {:.2}°/s, Y: {:.2}°/s, Z: {:.2}°/s", 
        x_dps, y_dps, z_dps);
    
    // Delay for next measurement
    for _ in 0..100_000 {
        cortex_m::asm::nop();
    }
}
```

### Using i3g4250d Crate (Automatic Conversion)

```rust
loop {
    // Crate handles all conversion internally
    let (x_dps, y_dps, z_dps) = gyro.angular_velocity()?;
    
    iprintln!(&mut itm.stim[0], 
        "X: {:.2}°/s, Y: {:.2}°/s, Z: {:.2}°/s", 
        x_dps, y_dps, z_dps);
    
    // Delay
    for _ in 0..100_000 {
        cortex_m::asm::nop();
    }
}
```

---

## 8. Practical Test: Verify Your Conversions

### Static Test (Sensor Not Moving)

When the sensor is **stationary**, all readings should be close to **0.0 °/s** (with ±0.5°/s noise):

```
Expected Output (stationary):
  X: 0.05°/s, Y: -0.12°/s, Z: 0.08°/s   ← Small noise, acceptable
  X: -0.03°/s, Y: 0.10°/s, Z: -0.07°/s  ← OK

Problem Indicators:
  X: 45.5°/s, Y: 90.2°/s, Z: 120.5°/s   ← Way too high! (scale factor error)
  X: 0.00°/s, Y: 0.00°/s, Z: 0.00°/s    ← All zeros (register read error)
  X: NaN, Y: NaN, Z: NaN                 ← Conversion error (check math)
```

### Dynamic Test (Manual Rotation)

Slowly rotate the board around each axis:

```
Around Z-axis (rotate on table):
  X: ~0°/s, Y: ~0°/s, Z: 10–50°/s  ← Only Z changes

Around X-axis (tilt up/down):
  X: 10–50°/s, Y: ~0°/s, Z: ~0°/s  ← Only X changes

Around Y-axis (tilt left/right):
  X: ~0°/s, Y: 10–50°/s, Z: ~0°/s  ← Only Y changes
```

---

## Scale Factor Reference Table

For quick lookup during conversions:

```
Full-Scale Range | Scale Factor | Raw Value | DPS Output
─────────────────┼──────────────┼───────────┼──────────
250 °/s          | 8.75 mDPS    | 1000      | 8.75 °/s
500 °/s          | 17.5 mDPS    | 1000      | 17.5 °/s
1000 °/s         | 70 mDPS      | 1000      | 70 °/s
2000 °/s         | 245 mDPS     | 1000      | 245 °/s
```

---

## References

- [I3G4250D Datasheet § 5.1](https://www.st.com/resource/en/datasheet/i3g4250d.pdf) — Register address map
- [I3G4250D Datasheet § 6.1](https://www.st.com/resource/en/datasheet/i3g4250d.pdf) — Register descriptions
- [Two's Complement (Wikipedia)](https://en.wikipedia.org/wiki/Two%27s_complement) — Signed integer representation
- [Endianness (Wikipedia)](https://en.wikipedia.org/wiki/Endianness) — Byte ordering


# I3G4250D Configuration Guide: Data Rates, Sensitivity, and Register Configuration

## Overview

This document explains how to configure the I3G4250D gyroscope's key parameters:
- **Data Rate (Output Data Rate / ODR)** — How often sensor readings update
- **Full-Scale Range (FS)** — Maximum measurable rotation speed
- **Bandwidth** — Filtering of high-frequency noise
- **Power Modes** — Trade-offs between power consumption and performance

---

## 1. Data Rate (Output Data Rate / ODR)

### What is Data Rate?

The **ODR** is how frequently the I3G4250D updates its output registers with new angular velocity measurements.

Higher ODR = More frequent updates = Higher power consumption  
Lower ODR = Fewer updates = Lower power consumption

### ODR Configuration (CTRL_REG1, bits 7-6)

The CTRL_REG1 register controls ODR via bits 7 and 6:

```
Bit 7-6: DR (Data Rate Select)
  00 = 100 Hz   (slowest, lowest power)
  01 = 200 Hz  (standard)
  10 = 400 Hz  (recommended for most applications)
  11 = 800 Hz  (fast, highest power)
```

### ODR Options Detailed

| ODR        | Frequency       | Period | Use Case | Power Draw | Latency | Noise Level |
|------------|-----------------|--------|----------|-----------|---------|-------------|
| **100 Hz** | 100 samples/sec | 10.5 ms | Battery-powered, slow motion | ⭐ (lowest) | High | Low |
| **200 Hz** | 200 samples/sec | 5.3 ms | Moderate speed, balanced | ⭐⭐ | Medium | Medium |
| **400 Hz** | 400 samples/sec | 2.6 ms | **Most common choice** | ⭐⭐⭐ | Low | Medium-High |
| **800 Hz** | 800 samples/sec | 1.3 ms | ⭐⭐⭐⭐ (highest) | Fast real-time | Lowest | Highest |

### Trade-Off Analysis

#### Power Consumption Trade-Off

```
100 Hz:   15 mW  (battery-friendly)
200 Hz:  25 mW  (balanced)
400 Hz:  40 mW  (standard for robotics)
800 Hz:  60 mW  (high-performance)
```

**Example:** A 100 mAh battery with 800 Hz mode would last ~100 hours. With 100 Hz, ~400 hours.

#### Update Latency Trade-Off

```
Latency = Period between samples
100 Hz:   10.5 ms (delay before new data available)
200 Hz:   5.3 ms
400 Hz:   2.6 ms  ← Good for real-time control
800 Hz:   1.3 ms  (minimal delay, best for drone stabilization)
```

**Example:** A quadrotor needs < 5 ms latency for stable control. Choose **400 Hz or higher**.

#### Noise Trade-Off

```
Lower ODR + internal filtering = lower noise
Higher ODR = sees more high-frequency noise

100 Hz:   -40 dB noise (very clean)
400 Hz:  -30 dB noise (moderate noise)
800 Hz:  -20 dB noise (noisier, needs software filtering)
```

### Recommended ODR by Application

| Application | Recommended ODR | Reason |
|---|-----------------|---|
| **Fitness Tracker** | 100 Hz          | Low power, coarse motion detection |
| **Smartphone IMU** | 200 Hz          | Balanced power and responsiveness |
| **Robotics/RC** | 400 Hz          | Good latency for control loops |
| **Quadrotor Stabilization** | 800 Hz          | Ultra-low latency for fast control |
| **Statistical Analysis** | 100 Hz           | Fewer samples = faster processing |
| **Motion Capture** | 800 Hz          | High fidelity motion recording |

---

## 2. Full-Scale Range (FS)

### What is Full-Scale Range?

The **FS** defines the maximum angular velocity the sensor can measure without saturation.

Higher FS = Can measure faster rotations, BUT less sensitive to slow motion  
Lower FS = More sensitive to small rotations, BUT saturates on fast motion

### FS Configuration (CTRL_REG4, bits 5-4)

```
Bit 5-4: FS (Full Scale Selection)
  00 = 250 °/s  (sensitive, smallest range)
  01 = 500 °/s  (balanced)
  10 = 1000 °/s (wide range, less sensitive)
  11 = 2000 °/s (widest range, least sensitive)
```

### FS Options with Scale Factors

| Range | Max Speed | Sensitivity (mDPS/LSB) | Resolution | Use Case |
|-------|-----------|------------------------|------------|----------|
| **250 °/s** | ±250 °/s | 8.75 mDPS/LSB | ⭐⭐⭐⭐ (highest) | Precise slow rotation |
| **500 °/s** | ±500 °/s | 17.5 mDPS/LSB | ⭐⭐⭐ | Balanced (recommended) |
| **1000 °/s** | ±1000 °/s | 70 mDPS/LSB | ⭐⭐ | Fast motion, robotics |
| **2000 °/s** | ±2000 °/s | 245 mDPS/LSB | ⭐ (lowest) | Extreme motion only |

### Understanding "mDPS/LSB"

**mDPS/LSB** = millidegrees per second per Least Significant Bit

The raw sensor output is a 16-bit signed integer. To convert to actual °/s:

```
Angular_Velocity (°/s) = Raw_Value × Scale_Factor / 1000
```

**Example: 250 °/s range**
- Scale factor: 8.75 mDPS/LSB
- Raw value from sensor: 1000
- Actual speed: 1000 × 8.75 / 1000 = 8.75 °/s

**Example: 2000 °/s range**
- Scale factor: 245 mDPS/LSB
- Raw value from sensor: 1000
- Actual speed: 1000 × 245 / 1000 = 245 °/s

### Recommended FS by Application

| Application | Recommended FS | Typical Speed Range |
|---|---|---|
| **Smartphone rotation** | 250 °/s | 0–100 °/s |
| **Fitness tracker** | 500 °/s | 0–200 °/s |
| **Drone stabilization** | 500–1000 °/s | 0–500 °/s |
| **Industrial robot arm** | 1000 °/s | 0–800 °/s |
| **High-speed spinning** | 2000 °/s | 1000+ °/s |

---

## 3. Bandwidth Configuration (CTRL_REG1, bits 5-4)

### Overview

After selecting ODR, bits 5-4 of CTRL_REG1 select **bandwidth**, which controls the internal low-pass filter.

**Why filter?** — Removes high-frequency noise from vibration and electromagnetic interference.

### Bandwidth Options by ODR

Bandwidth is **dependent on ODR selection**. Each ODR has multiple bandwidth options:

```
For ODR = 100 Hz (bits 7-6 = 00):
  BW (bits 5-4):
    00 = 12.5 Hz cutoff (aggressive filtering)
    01 = 25 Hz cutoff
    10 = 25 Hz cutoff (default)
    11 = 25 Hz cutoff

For ODR = 200 Hz (bits 7-6 = 01):
  BW (bits 5-4):
    00 = 12.5 Hz cutoff
    01 = 25 Hz cutoff
    10 = 50 Hz cutoff
    11 = 70 Hz cutoff

For ODR = 400 Hz (bits 7-6 = 10):
  BW (bits 5-4):
    00 = 20 Hz cutoff
    01 = 25 Hz cutoff
    10 = 50 Hz cutoff (default, recommended)
    11 = 110 Hz cutoff

For ODR = 800 Hz (bits 7-6 = 11):
  BW (bits 5-4):
    00 = 30 Hz cutoff
    01 = 35 Hz cutoff
    10 = 50 Hz cutoff (default)
    11 = 110 Hz cutoff
```

**See I3G4250D Datasheet Table 21 (page 24) for complete bandwidth table.**

### Filtering Strategy

- **Narrow bandwidth** = More filtering, slower response, but cleaner data
- **Wide bandwidth** = Less filtering, faster response, but more noise

**Recommendation:** Use default bandwidth settings (usually bits 5-4 = 10 binary).

### Automatic Cutoff Frequency Scaling

**Key Point:** Cutoff frequency automatically scales with data rate to maintain ~50% of data rate:

```
100 Hz DR + BW=11 → 50 Hz cutoff    (50% of 100 Hz)
200 Hz DR + BW=11 → 100 Hz cutoff   (50% of 200 Hz)
400 Hz DR + BW=11 → 200 Hz cutoff   (50% of 400 Hz)
800 Hz DR + BW=11 → 400 Hz cutoff   (50% of 800 Hz)
```

This ensures good Nyquist coverage: your cutoff is always high enough to preserve real motion signals.

---

## 3.5. How ODR, Cutoff, and Range Work Together

### The Three-Parameter Relationship

These three parameters are **independent but work together** to define sensor behavior:

```
                    ┌─────────────────────┐
                    │   DATA RATE (ODR)   │
                    │  100/200/400/800 Hz │
                    └──────────┬──────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
   │ CUTOFF FREQ │      │ POWER USAGE │      │ TIME PER    │
   │ (Auto-set)  │      │ (Auto-set)  │      │ SAMPLE      │
   │ Higher DR   │      │ Higher DR   │      │ Higher DR   │
   │ → Higher    │      │ → More      │      │ → Shorter   │
   │   Cutoff    │      │   Power     │      │   Interval  │
   └─────────────┘      └─────────────┘      └─────────────┘
        │
        └──────────────────────┬──────────────────────┐
                               │
                    ┌──────────▼──────────┐
                    │  FULL-SCALE RANGE   │
                    │  245/500/2000 °/s   │
                    │  (Independent!)     │
                    └─────────┬───────────┘
                              │
        ┌─────────────────────┼────────────────────┐
        │                     │                    │
        ▼                     ▼                    ▼
   ┌─────────────┐    ┌─────────────┐    ┌──────────────┐
   │ RESOLUTION  │    │ MAX MOTION  │    │ SCALE FACTOR │
   │ (Precision) │    │ MEASURABLE  │    │ SENSITIVITY  │
   │ 245°/s:     │    │ 245°/s:     │    │ 245°/s:      │
   │ 0.0087°/s   │    │ ±245°/s     │    │ 8.75 mDPS/LSB│
   │             │    │             │    │              │
   │ 2000°/s:    │    │ 2000°/s:    │    │ 2000°/s:     │
   │ 0.061°/s    │    │ ±2000°/s    │    │ 245 mDPS/LSB │
   └─────────────┘    └─────────────┘    └──────────────┘
```

### Nyquist Theorem and Data Rate

To accurately capture motion, your data rate must be **at least 2× the highest frequency of motion**:

```
Example 1: Slow head movement
  Maximum motion frequency: ~2 Hz
  Required data rate: 2 × 2 = 4 Hz minimum
  Safe margin: Use 100 Hz (25× higher)

Example 2: Fast drone rotation
  Maximum motion frequency: ~80 Hz
  Required data rate: 2 × 80 = 160 Hz minimum
  Safe margin: Use 400-800 Hz (2.5-5× higher)
```

---

## 4. Use Case Selection Guide

### 🎮 Use Case 1: VR Head Tracking (High Precision)

```rust
gyro.set_data_rate(DataRate::Hz100)?;   // Slow motion only
gyro.set_range(Range::DPS245)?;         // Maximum precision
```

**Why this combination:**
- Head motion is slow (< 5 Hz), so 100 Hz is 20× higher (safe margin)
- 245°/s is never exceeded in normal head motion
- 8.75 mDPS/LSB scale gives excellent precision (0.0087°/s)
- Cutoff: 50 Hz (removes drift, keeps motion)
- Power: 3.2 mA (very low)

**Typical output (stationary):**
```
X:  0.02°/s | Y: -0.05°/s | Z:  0.01°/s
```

**Typical output (slow head tilt):**
```
X:  5.2°/s  | Y: -3.1°/s  | Z:  0.3°/s
```

---

### 📱 Use Case 2: Smartphone (BALANCED - RECOMMENDED) ⭐

```rust
gyro.set_data_rate(DataRate::Hz400)?;   // ← YOUR CURRENT CONFIG
gyro.set_range(Range::DPS500)?;         // ← YOUR CURRENT CONFIG
```

**Why this combination:**
- Smartphone motion is ~15 Hz, so 400 Hz is 25× higher (good margin)
- 500°/s covers all normal phone rotations
- 17.5 mDPS/LSB scale is good precision (0.0175°/s)
- Cutoff: 200 Hz (removes drift, keeps motion)
- Power: 4.5 mA (acceptable for most devices)

**Typical output (stationary):**
```
X:  0.05°/s  | Y:  0.08°/s  | Z: -0.02°/s
```

**Typical output (landscape rotation):**
```
X:  0.1°/s   | Y:  0.2°/s   | Z: 45.3°/s
```

**Typical output (quick shake):**
```
X:  89.5°/s  | Y:  76.2°/s  | Z: 42.1°/s
```

---

### 🚁 Use Case 3: Fast Drone (Speed-Focused)

```rust
gyro.set_data_rate(DataRate::Hz800)?;   // Maximum speed
gyro.set_range(Range::DPS2000)?;        // Can handle fast spins
```

**Why this combination:**
- Drone motion is ~80 Hz, so 800 Hz is 10× higher (minimum safe)
- 2000°/s covers all drone maneuvers (rapid rolls, loops)
- 70 mDPS/LSB is acceptable for control (0.07°/s)
- Cutoff: 400 Hz (captures high-frequency dynamics)
- Power: 6.5 mA (acceptable for drones)

**Typical output (hovering):**
```
X:  0.05°/s  | Y:  0.08°/s  | Z: -0.02°/s
```

**Typical output (rapid roll):**
```
X: 450°/s   | Y: -280°/s   | Z: 120°/s
```

---

### 🏥 Use Case 4: Medical Motion Capture (Precision)

```rust
gyro.set_data_rate(DataRate::Hz200)?;   // Sufficient for humans
gyro.set_range(Range::DPS245)?;         // Maximum precision
```

**Why this combination:**
- Human motion is ~5 Hz, so 200 Hz is 40× higher (excellent margin)
- 245°/s never exceeded in medical applications
- 8.75 mDPS/LSB scale allows detecting tiny movements
- Can detect tremors (0.01°/s resolution)
- Power: 3.5 mA (low)

---

## 5. What Happens if You Choose Wrong?

### ❌ Problem 1: Data Rate Too Low

```
Configuration: 100 Hz data rate
Actual motion: 60 Hz rotation (rapid movement)
Nyquist requirement: 2 × 60 Hz = 120 Hz minimum

Result: ALIASING
  Real smooth 60 Hz motion appears as random jitter
  Data becomes unreliable
  
Fix: Use 200 Hz or 400 Hz data rate
```

### ❌ Problem 2: Full-Scale Range Too Small

```
Configuration: 245°/s range
Actual motion: 400°/s spin

Result: CLIPPING (data loss)
  Real motion: 400°/s
  Sensor reads: 245°/s (stuck at maximum!)
  
Loss: 155°/s of motion data is lost

Fix: Use 500°/s or 2000°/s range
```

### ❌ Problem 3: Full-Scale Range Too Large

```
Configuration: 2000°/s range
Actual motion: 10°/s slow tilt
Scale factor: 70 mDPS/LSB = 0.07°/s precision

Result: COARSE MEASUREMENT
  Resolution loss: ~7× worse than optimal 245°/s range
  Cannot detect small tremors or precise motion
  
Fix: Use 245°/s or 500°/s range for better precision
```

### ❌ Problem 4: Cutoff Frequency Too Low

```
Configuration: 100 Hz data rate with BW=00 (12.5 Hz cutoff)
Actual motion: 2 Hz slow head tilt

Result: SIGNAL LOSS
  Your 2 Hz motion is being attenuated (weakened)
  Measured: 0.5°/s instead of 2°/s (75% signal loss!)
  
Fix: Always use BW=11 (maximum bandwidth)
     Current default in init() is correct: 0xBF for 400 Hz
```

---

## 6. Decision Flow Chart

```
START
  ↓
What's your maximum expected motion frequency?
  ↓
  ├─ < 5 Hz   → Use 100 Hz (save power)
  ├─ < 20 Hz  → Use 200 Hz (balanced)
  ├─ < 50 Hz  → Use 400 Hz ⭐ BEST FOR MOST
  └─ > 50 Hz  → Use 800 Hz (high speed)
  ↓
What's your maximum expected rotation rate?
  ↓
  ├─ < 250°/s → Use 245°/s (maximum precision: 8.75 mDPS/LSB)
  ├─ < 500°/s → Use 500°/s ⭐ RECOMMENDED
  └─ > 500°/s → Use 2000°/s (can measure fast spin)
  ↓
DONE! 
  ODR determines: Cutoff frequency, Power, Latency
  Range determines: Precision, Maximum measurable motion
  BW: Always use 11 (maximum bandwidth)
```

---

## 4. Axis Enable/Disable (CTRL_REG1, bits 2-0)

### Configuration

```
Bit 2: Zen (Z-axis enable)   1 = enabled, 0 = disabled
Bit 1: Yen (Y-axis enable)   1 = enabled, 0 = disabled
Bit 0: Xen (X-axis enable)   1 = enabled, 0 = disabled
```

### Use Cases

| Scenario | Configuration | Reason |
|----------|---|---|
| All axes enabled (default) | Xen=1, Yen=1, Zen=1 | Normal 3-axis gyroscope |
| Only Z-rotation | Xen=0, Yen=0, Zen=1 | Yaw-only tracking (compass) |
| X-Y motion only | Xen=1, Yen=1, Zen=0 | Tilt/pitch detection |
| Single axis debug | Enable one axis | Test one axis at a time |

---

## 5. Power-Down Mode (CTRL_REG1, bit 3)

### Configuration

```
Bit 3: PD (Power Down)
  0 = Power-down mode (device OFF, minimal power draw)
  1 = Normal/Sleep mode (device ON, active measurements)
```

### Power Modes

| Mode | Power Draw | Operation | Wake-Up Time |
|------|-----------|-----------|--------------|
| **Power-Down** (PD=0) | <1 mW | No measurements | 10-20 ms |
| **Normal** (PD=1) | 15-60 mW | Active measurements | Immediate |

---

## CTRL_REG1 Configuration Examples

### Example 1: Default (Recommended for Most Applications)

```
Configuration: 380 Hz, All axes enabled, Normal mode, Default bandwidth
Binary: 0b00111111
Hex:    0x3F

Breakdown:
  Bits 7-6 (DR):  10 = 380 Hz
  Bits 5-4 (BW):  11 = default bandwidth
  Bit  3   (PD):  1  = Power-down disabled (normal mode)
  Bits 2-0 (XYZ): 111 = All axes enabled
```

Code:
```rust
const CTRL_REG1_DEFAULT: u8 = 0x3F;
```

### Example 2: Low-Power Battery Mode

```
Configuration: 95 Hz, All axes, Normal mode
Binary: 0b00001111
Hex:    0x0F

Breakdown:
  Bits 7-6: 00 = 95 Hz
  Bits 5-4: 00 = low BW
  Bit 3:    1  = Normal mode
  Bits 2-0: 111 = All axes
```

Code:
```rust
const CTRL_REG1_LOWPOWER: u8 = 0x0F;
```

### Example 3: High-Speed Robotics Mode

```
Configuration: 760 Hz, All axes, Normal mode
Binary: 0b11111111
Hex:    0xFF

Breakdown:
  Bits 7-6: 11 = 760 Hz
  Bits 5-4: 11 = high BW
  Bit 3:    1  = Normal mode
  Bits 2-0: 111 = All axes
```

Code:
```rust
const CTRL_REG1_HIGHSPEED: u8 = 0xFF;
```

---

## 6. CTRL_REG4 Configuration: Range and Endianness

### Bits 5-4: Full-Scale Range (FS)

```
Bits 5-4:
  00 = 250 °/s
  01 = 500 °/s
  10 = 1000 °/s
  11 = 2000 °/s
```

### Bit 6: Endianness (BLE)

```
Bit 6: BLE (Big Endian / Little Endian)
  0 = Little Endian (default, recommended)
  1 = Big Endian
```

**Stay with Little Endian (0)** unless you have a specific reason.

### Example CTRL_REG4 Configurations

```rust
// 250 °/s, little-endian
const CTRL_REG4_250DPS: u8 = 0b00000000;  // 0x00

// 500 °/s, little-endian (recommended)
const CTRL_REG4_500DPS: u8 = 0b00010000;  // 0x10

// 1000 °/s, little-endian
const CTRL_REG4_1000DPS: u8 = 0b00100000; // 0x20

// 2000 °/s, little-endian
const CTRL_REG4_2000DPS: u8 = 0b00110000; // 0x30
```

---

## Complete Initialization Example

### Using Custom Driver

```rust
// Configure: 380 Hz, 500 °/s range
let mut gyro = GyroDriver::new(spi, cs);

// Write CTRL_REG1: 380 Hz, all axes enabled
gyro.write_register(0x20, 0x3F)?;

// Write CTRL_REG4: 500 °/s range, little-endian
gyro.write_register(0x23, 0x10)?;

// Start reading
loop {
    let (x, y, z) = gyro.read_raw_data()?;
    
    // Convert using scale factor 17.5 mDPS/LSB
    let x_dps = (x as f32 * 17.5) / 1000.0;
    let y_dps = (y as f32 * 17.5) / 1000.0;
    let z_dps = (z as f32 * 17.5) / 1000.0;
    
    iprintln!(&mut itm.stim[0], "X: {:.2}°/s, Y: {:.2}°/s, Z: {:.2}°/s", x_dps, y_dps, z_dps);
}
```

### Using i3g4250d Crate

```rust
use i3g4250d::{I3g4250d, DataRate, Range};

let mut gyro = I3g4250d::new(spi, cs)?;
gyro.set_data_rate(DataRate::DPS380)?;
gyro.set_range(Range::DPS500)?;

loop {
    let (x, y, z) = gyro.angular_velocity()?;  // Already in °/s
    iprintln!(&mut itm.stim[0], "X: {:.2}°/s, Y: {:.2}°/s, Z: {:.2}°/s", x, y, z);
}
```

---

## Summary Table: Complete Configuration Reference

### All 12 Possible Combinations

```
┌──────────┬──────────┬──────────────┬───────────┬──────────┬──────────────┐
│ DATA     │ RANGE    │ CUTOFF FREQ  │ POWER     │ MAX      │ SCALE FACTOR │
│ RATE     │          │ (BW=11)      │ DRAW      │ MEASURE  │ (mDPS/LSB)   │
├──────────┼──────────┼──────────────┼───────────┼──────────┼──────────────┤
│ 100 Hz   │ 245°/s   │ 50 Hz        │ 3.2 mA    │ ±245°/s  │ 8.75         │
│ 100 Hz   │ 500°/s   │ 50 Hz        │ 3.2 mA    │ ±500°/s  │ 17.5         │
│ 100 Hz   │ 2000°/s  │ 50 Hz        │ 3.2 mA    │ ±2000°/s │ 70.0         │
├──────────┼──────────┼──────────────┼───────────┼──────────┼──────────────┤
│ 200 Hz   │ 245°/s   │ 100 Hz       │ 3.5 mA    │ ±245°/s  │ 8.75         │
│ 200 Hz   │ 500°/s   │ 100 Hz       │ 3.5 mA    │ ±500°/s  │ 17.5         │
│ 200 Hz   │ 2000°/s  │ 100 Hz       │ 3.5 mA    │ ±2000°/s │ 70.0         │
├──────────┼──────────┼──────────────┼───────────┼──────────┼──────────────┤
│ 400 Hz   │ 245°/s   │ 200 Hz       │ 4.5 mA    │ ±245°/s  │ 8.75         │
│ 400 Hz   │ 500°/s   │ 200 Hz       │ 4.5 mA    │ ±500°/s  │ 17.5         │ ⭐ BEST
│ 400 Hz   │ 2000°/s  │ 200 Hz       │ 4.5 mA    │ ±2000°/s │ 70.0         │
├──────────┼──────────┼──────────────┼───────────┼──────────┼──────────────┤
│ 800 Hz   │ 245°/s   │ 400 Hz       │ 6.5 mA    │ ±245°/s  │ 8.75         │
│ 800 Hz   │ 500°/s   │ 400 Hz       │ 6.5 mA    │ ±500°/s  │ 17.5         │
│ 800 Hz   │ 2000°/s  │ 400 Hz       │ 6.5 mA    │ ±2000°/s │ 245.0        │ ⭐ FASTEST
└──────────┴──────────┴──────────────┴───────────┴──────────┴──────────────┘
```

### Recommended Configurations by Use Case

| Use Case | ODR | Range | Cutoff | Power | Precision | Rating |
|----------|-----|-------|--------|-------|-----------|--------|
| **VR Head Tracking** | 100 Hz | 245°/s | 50 Hz | 3.2mA | 8.75 | ⭐⭐⭐⭐⭐ |
| **Smartphone** ⭐ | 400 Hz | 500°/s | 200 Hz | 4.5mA | 17.5 | ⭐⭐⭐⭐⭐ |
| **Drone Stabilization** | 800 Hz | 2000°/s | 400 Hz | 6.5mA | 245 | ⭐⭐⭐⭐ |
| **Medical Motion** | 200 Hz | 245°/s | 100 Hz | 3.5mA | 8.75 | ⭐⭐⭐⭐⭐ |
| **Fitness Tracker** | 100 Hz | 500°/s | 50 Hz | 3.2mA | 17.5 | ⭐⭐⭐⭐ |
| **Industrial Robot** | 400 Hz | 1000°/s | 200 Hz | 4.5mA | 70 | ⭐⭐⭐⭐ |

---

## Key Takeaways

### ✅ Best Practices

1. **Default Best Choice:** `400 Hz + 500°/s` (covers 95% of use cases)
   
2. **Always Use:** `BW=11` (maximum bandwidth) in init
   - Prevents filtering out real motion
   - Balances noise removal with signal preservation

3. **Nyquist Safety Margin:** Use data rate 10-20× your motion frequency
   - Minimum: 2× (Nyquist theorem)
   - Recommended: 10-20× (practical safety margin)

4. **Range Selection:** Choose range slightly higher than your max expected motion
   - Not too high (precision loss)
   - Not too low (clipping/saturation)

### ❌ Avoid

1. **Never use BW=00** (narrow bandwidth filters out slow genuine motion)

2. **Never use data rate < 10× motion frequency** (aliasing causes unreliable data)

3. **Never use range < 2× expected max motion** (clipping causes data loss)

4. **Never choose range > 5× expected max motion** (excessive precision loss)

### 🎯 Quick Selection Process

1. Estimate your maximum motion frequency
2. Choose data rate = 10-20× that frequency
3. Estimate your maximum expected rotation rate
4. Choose range = 1.5-2× that rate
5. Done! Cutoff automatically scales with data rate

### Current Your Configuration

```rust
// Your current setup in main.rs:
gyro.set_data_rate(DataRate::Hz800)?;   // 800 Hz
gyro.set_range(Range::DPS245)?;         // 245°/s
```

**Analysis:**
- ODR (800 Hz): Higher than needed for smartphones, but fine for testing
- Range (245°/s): Good for slow motion, but limited for fast rotations
- Cutoff (400 Hz): Wide, good for capturing fast dynamics
- **Better alternative:** `400 Hz + 500°/s` for more balanced performance

---

## References

- [I3G4250D Datasheet § 6.1–6.2](https://www.st.com/resource/en/datasheet/i3g4250d.pdf) — Register descriptions
- [I3G4250D Datasheet Table 28](https://www.st.com/resource/en/datasheet/i3g4250d.pdf) — Bandwidth table by ODR


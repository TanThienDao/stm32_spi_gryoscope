# I3G4250D Gyroscope Implementation Guide

## Overview

This guide explains **two approaches** to reading gyroscope data from the I3G4250D sensor connected via SPI:

1. **Approach 1: Using the `i3g4250d` Crate** (Recommended for rapid prototyping)
2. **Approach 2: Custom SPI Driver** (Recommended for learning and full control)

Both approaches use the same underlying SPI protocol but differ in abstraction level and API.

---

## Hardware Quick Reference

| Property | Value |
|----------|-------|
| **Communication** | SPI Mode 3 (CPOL=1, CPHA=1) |
| **Clock Speed** | 1 MHz |
| **Chip Select** | PE3 (Active Low) |
| **Data Lines** | PA5 (SCK), PA6 (MISO), PA7 (MOSI) |
| **WHO_AM_I Address** | 0x0F |
| **WHO_AM_I Value (I3G4250D)** | 0xD3 |
| **Angular Data Registers** | 0x28–0x2D (6 bytes: X_L, X_H, Y_L, Y_H, Z_L, Z_H) |

---

## Approach 1: Using the `i3g4250d` Crate

### What is the `i3g4250d` Crate?

The `i3g4250d` crate (v0.3.0, already in your `Cargo.toml`) is a high-level Rust driver that:
- Abstracts away raw SPI register reads/writes
- Provides typed configuration options (e.g., `DataRate::DPS400`, `Range::DPS500`)
- Returns data in human-friendly units (angular velocity in degrees per second)
- Handles endianness and scaling automatically
- Includes error handling

### Advantages of Crate Approach

✅ **Rapid Development** — Get reading data in 10–15 lines of code  
✅ **Type-Safe APIs** — Compiler catches invalid configurations  
✅ **Abstraction** — No need to know register addresses or byte-order details  
✅ **Maintenance** — Bugs fixed in crate, not your code  
✅ **Documentation** — Crate has its own [docs.rs](https://docs.rs/i3g4250d/latest/i3g4250d/) page  

### Disadvantages of Crate Approach

❌ **Black Box** — Limited control over exact register settings  
❌ **Code Size** — Adds ~500 bytes of binary overhead  
❌ **Learning Curve** — Hides implementation details (less educational)  
❌ **Dependency** — Adds external dependency to project  

### Crate-Based Implementation

#### Step 1: Update `auxiliary/src/lib.rs` — Add Gyroscope Struct

Add a function to initialize the I3G4250D using the crate:

```rust
use i3g4250d::{I3g4250d, DataRate, Range};

/// Initialize I3G4250D gyroscope using the i3g4250d crate
pub fn init_gyroscope<SPI, CS>(
    spi: &mut SPI,
    cs: &mut CS,
) -> Result<I3g4250d<SPI, CS>, &'static str>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
{
    // Create gyroscope driver instance
    let mut gyro = I3g4250d::new(spi, cs).map_err(|_| "Failed to create gyroscope")?;
    
    // Configure: 400 Hz output rate, 500 °/s range
    gyro.set_data_rate(DataRate::DPS400)
        .map_err(|_| "Failed to set data rate")?;
    gyro.set_range(Range::DPS500)
        .map_err(|_| "Failed to set range")?;
    
    Ok(gyro)
}

/// Read single angular velocity measurement from I3G4250D (crate-based)
pub fn read_gyro_data<SPI, CS>(
    gyro: &mut I3g4250d<SPI, CS>,
) -> Result<(f32, f32, f32), &'static str>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
{
    // Read returns (x, y, z) in degrees per second as f32
    gyro.angular_velocity().map_err(|_| "Failed to read gyroscope")
}
```

#### Expected Output (Crate Approach)
```
Gyroscope initialization starting...
Found: I3g4250d
Gyroscope configured!
X: 0.05°/s, Y: -0.10°/s, Z: 0.15°/s
X: 0.02°/s, Y: 0.12°/s, Z: 0.08°/s
...
```

---

## Approach 2: Custom SPI Driver

### What is a Custom Driver?

A custom driver is Rust code **you write** that:
- Directly reads/writes gyroscope registers via raw SPI transfers
- Manually handles byte conversion and scaling
- Gives you complete control over configuration
- Requires understanding the I3G4250D register map

### Advantages of Custom Driver Approach

✅ **Educational** — Learn exactly how SPI sensors work  
✅ **Control** — Configure any register combination you want  
✅ **Minimal Overhead** — No external dependencies  
✅ **Debugging** — Easy to add custom logging/timing  
✅ **Embedded-Friendly** — Works in pure `#![no_std]` projects  

### Disadvantages of Custom Driver Approach

❌ **Time-Consuming** — 50–100 lines of code to write  
❌ **Error-Prone** — Easy to make byte-order or scaling mistakes  
❌ **Maintenance** — You're responsible for bug fixes  
❌ **Complexity** — Need to understand register maps and SPI protocol  

### Key Register Definitions

| Register | Address | Purpose |
|----------|---------|---------|
| WHO_AM_I | 0x0F | Device ID (0xD3 = I3G4250D) |
| CTRL_REG1 | 0x20 | Power mode, data rate, axis enable |
| CTRL_REG2 | 0x21 | High-pass filter config |
| CTRL_REG3 | 0x22 | Interrupt config |
| CTRL_REG4 | 0x23 | Scale range, endianness |
| CTRL_REG5 | 0x24 | FIFO, SPI mode |
| TEMP | 0x26 | Temperature (optional) |
| OUT_X_L | 0x28 | X-axis low byte |
| OUT_X_H | 0x29 | X-axis high byte |
| OUT_Y_L | 0x2A | Y-axis low byte |
| OUT_Y_H | 0x2B | Y-axis high byte |
| OUT_Z_L | 0x2C | Z-axis low byte |
| OUT_Z_H | 0x2D | Z-axis high byte |

### SPI Read/Write Protocol

**Reading a Register:**
```
1. Set CS low
2. Send: [register_address | 0x80, dummy_byte]  (0x80 = read bit)
3. Receive: [address_echo, register_value]
4. Set CS high
```

**Writing a Register:**
```
1. Set CS low
2. Send: [register_address & 0x7F, new_value]  (MSB = 0 for write)
3. Receive: [address_echo, status]
4. Set CS high
```

### Custom Driver Implementation

See implementation in **Step 1** below (gyro_driver.rs file contains all register definitions and driver logic).

---

## Comparison Table

| Feature | i3g4250d Crate | Custom Driver |
|---------|---|---|
| **Lines of Code** | ~10 | ~200+ |
| **Setup Time** | < 5 minutes | 30–60 minutes |
| **Learning Curve** | Low | Medium–High |
| **Debugging** | Harder (black box) | Easier (you control it) |
| **Binary Size** | +500 bytes | +200 bytes |
| **Customization** | Limited | Unlimited |
| **Error Handling** | Built-in | Your responsibility |
| **Dependencies** | 1 external | 0 external |
| **Recommended For** | Production, quick prototyping | Learning, research, debugging |

---

## Which Approach to Choose?

- **Use the Crate if:** You want working code quickly, don't need to understand internals, or are building a product
- **Use Custom Driver if:** You're learning embedded systems, need tight control, or want to optimize for space/speed

---

## Next Steps

1. Choose one approach (start with the crate for speed, switch to custom driver later for learning)
2. Implement the code in your project
3. Read the [I3G4250D_Configuration.md](I3G4250D_Configuration.md) for advanced data rate and register options
4. Refer to [I3G4250D_Data_Handling.md](I3G4250D_Data_Handling.md) for mathematical details

---

## References

- [i3g4250d Crate Docs](https://docs.rs/i3g4250d/latest/i3g4250d/)
- [I3G4250D Datasheet](https://www.st.com/resource/en/datasheet/i3g4250d.pdf)
- [STM32F3 Discovery Board Manual](https://www.st.com/resource/en/datasheet/stm32f303vc.pdf)


# Read and Write Register Operations

This document explains how the `read_register()` and `write_register()` functions work in the I3G4250D gyroscope driver.

## Overview

The I3G4250D communicates via SPI using a specific protocol where the **MSB (Most Significant Bit) of the address byte** acts as a control flag:
- **MSB = 1** → Read operation
- **MSB = 0** → Write operation

## Read Register Function

### Code
```rust
fn read_register(&mut self, reg_addr: u8) -> Result<u8, &'static str> {
    let mut buffer = [reg_addr | 0x80, 0x00];  // Set read bit (MSB = 1)

    self.cs.set_low().ok();
    self.spi.transfer(&mut buffer)
        .map_err(|_| "SPI transfer failed")?;
    self.cs.set_high().ok();

    Ok(buffer[1])  // Return received byte
}
```

### How It Works

1. **Set Read Bit**: `reg_addr | 0x80`
   - Uses bitwise OR with `0x80` (binary: `10000000`)
   - Forces the MSB to 1, indicating a read command
   - Preserves all other bits (bits 6-0) of the register address
   
   **Example:**
   ```
   If reg_addr = 0x0F (WHO_AM_I register)
   0x0F = 00001111
   0x80 = 10000000
   OR   = 10001111 (0x8F)  ← MSB is now 1 (read mode)
   ```

2. **SPI Transaction**:
   - Lower CS (Chip Select) to start communication
   - Send the command byte with read bit set, followed by dummy byte
   - Receive echo of command and the register value
   - Raise CS to end communication

3. **Extract Data**: Return `buffer[1]` which contains the register value

### SPI Protocol
```
Transmission: [0x8F (read addr), 0x00 (dummy)]
Reception:    [0x8F (echo),     register_value]
              └─── ignored ────┘ └─ this is returned ─┘
```

---

## Write Register Function

### Code
```rust
fn write_register(&mut self, reg_addr: u8, value: u8) -> Result<(), &'static str> {
    let mut buffer = [reg_addr & 0x7F, value];  // Clear read bit (MSB = 0)

    self.cs.set_low().ok();
    self.spi.transfer(&mut buffer)
        .map_err(|_| "SPI transfer failed")?;
    self.cs.set_high().ok();

    Ok(())
}
```

### How It Works

1. **Clear Read Bit**: `reg_addr & 0x7F`
   - Uses bitwise AND with `0x7F` (binary: `01111111`)
   - Forces the MSB to 0, indicating a write command
   - Preserves all other bits (bits 6-0) of the register address
   
   **Example:**
   ```
   If reg_addr = 0x20 (CTRL_REG1)
   0x20 = 00100000
   0x7F = 01111111
   AND  = 00100000 (0x20)  ← MSB remains 0 (write mode)
   ```

2. **SPI Transaction**:
   - Lower CS to start communication
   - Send the command byte with write bit clear (MSB=0), followed by the value to write
   - Receive echo of command and status byte (not used)
   - Raise CS to end communication

3. **Confirm Success**: Return `Ok(())`

### SPI Protocol
```
Transmission: [0x20 (write addr), value_to_write]
Reception:    [0x20 (echo),       status_byte]
              └─── ignored ────┘ └─ ignored ─┘
```

---

## Bitwise Operation Comparison

### OR Operation (`|`) - Setting Bits

```
reg_addr | 0x80
```

| Scenario | Input MSB | 0x80 MSB | Result MSB | Purpose |
|----------|-----------|----------|-----------|---------|
| New address | 0 | 1 | **1** ✓ | Switches MSB to 1 |
| Address already has MSB | 1 | 1 | **1** ✓ | Keeps MSB as 1 |

**Result**: MSB is guaranteed to be 1, other bits unchanged.

### AND Operation (`&`) - Clearing Bits

```
reg_addr & 0x7F
```

| Scenario | Input MSB | 0x7F MSB | Result MSB | Purpose |
|----------|-----------|----------|-----------|---------|
| New address | 0 | 1 | **0** ✓ | Keeps MSB as 0 |
| Address has MSB | 1 | 1 | **0** ✓ | Switches MSB to 0 |

**Result**: MSB is guaranteed to be 0, other bits unchanged.

---

## Example Usage

```rust
// Read WHO_AM_I register (should return 0xD3 for I3G4250D)
let device_id = gyro.read_register(0x0F)?;  // Internally: 0x0F | 0x80 = 0x8F

// Write to CTRL_REG1 (power on and enable axes)
gyro.write_register(0x20, 0xBF)?;  // Internally: 0x20 & 0x7F = 0x20
```

---

## Why This Design?

The I3G4250D datasheet specifies this protocol to allow a single address byte to encode both:
1. **Which register** to access (bits 6-0)
2. **What type of operation** (bit 7: read/write)

This efficient design reduces the complexity of the SPI communication while maintaining full control over the device registers.


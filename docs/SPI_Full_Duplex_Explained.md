# SPI Full-Duplex Communication Explained

## Table of Contents
1. [SPI Basics](#spi-basics)
2. [Full-Duplex Communication](#full-duplex-communication)
3. [Vector Explanation](#vector-explanation)
4. [SPI Command Protocol (0x0F | 0x80)](#spi-command-protocol)
5. [Practical Example with I3G4250D](#practical-example-with-i3g4250d)
6. [Testing with Mock Objects](#testing-with-mock-objects)

---

## SPI Basics

### What is SPI?

**Serial Peripheral Interface (SPI)** is a synchronous serial communication protocol commonly used for:
- Reading sensor data (gyroscope, etc.)
- Flash memory access
- Display controllers
- DACs/ADCs

### Key Components

```
Master (STM32F3)              Slave (I3G4250D Gyroscope)
    │                              │
    ├─ MOSI (Master Out) ────────→ MOSI (Master Out, Slave In)
    ├─ MISO (Master In) ←──────── MISO (Master In, Slave Out)
    ├─ SCK (Clock) ─────────────→ SCK (Shift Clock)
    └─ CS (Chip Select) ────────→ CS (Chip Select, Active Low)
```

### Signal Lines

| Signal | Direction | Purpose |
|--------|-----------|---------|
| **MOSI** | Master → Slave | Master sends commands and data |
| **MISO** | Slave → Master | Slave sends responses and data |
| **SCK** | Master → Slave | Clock signal (synchronizes communication) |
| **CS** | Master → Slave | Active-low chip select (selects the slave) |

---

## Full-Duplex Communication

### What is Full-Duplex?

**Full-duplex** means data flows in **BOTH directions SIMULTANEOUSLY**:
- Master sends data on MOSI while receiving data on MISO
- Slave sends data on MISO while receiving data on MOSI
- Both happen at the same clock cycle

### vs. Half-Duplex

```
Half-Duplex (Sequential):
┌─────────────────────────────────┐
│ Master sends → then ← Slave responds │
│ (Takes 2x time)                 │
└─────────────────────────────────┘

Full-Duplex (Simultaneous):
┌─────────────────────────────────┐
│ Master sends → ← Slave responds │
│ AT THE SAME TIME               │
│ (Faster!)                       │
└─────────────────────────────────┘
```

### Timing Diagram

```text
Clock (SCK):  ──┐  ┌──┐  ┬──┐  ┬──┐  ┬──┐  ┬──┐  ┬──┐  ┬──┐  ┬──
              ──┘──┘  └──┘  └──┘  └──┘  └──┘  └──┘  └──┘  └──┘  └──

Byte 0 Transmission:
Master (MOSI): ─┐ 1 0 0 0 1 1 1 1 ┐─  (0x8F: Read WHO_AM_I register `0x0F | 0x80`)
               └─────────────────┘

Slave (MISO):  ─┐ 0 0 0 0 0 0 0 0 ┐─  (0x00: Dummy/Busy response)
               └─────────────────┘

Byte 1 Transmission:
Master (MOSI): ─┐ 0 0 0 0 0 0 0 0 ┐─  (0x00: Dummy byte to clock in data)
               └─────────────────┘

Slave (MISO):  ─┐ 1 1 0 1 0 0 1 1 ┐─  (0xD3: WHO_AM_I response for I3G4250D)
               └─────────────────┘
```

---

## Vector Explanation

### The Buffer Concept

In SPI full-duplex communication, we use a **single buffer** that gets overwritten during transmission:

```rust
let mut buffer = [0x00, 0xD4];
//               └────┘ └────┘
//                 │      │
//         Byte 0  │      │  Byte 1
```

### Before Transfer

```rust
let mut buffer = [0x8F, 0x00];
                 └─┬──┘ └─┬──┘
                   │      │
            Command │      └─ Dummy byte for clocking in response
            to send └─ Address (0x0F) + Read bit (0x80)
```

**What Master Will Send:**
- `buffer[0] = 0x8F` → "Read from WHO_AM_I register (0x0F)"
- `buffer[1] = 0x00` → Dummy byte to keep clock running

### During Transfer (Full-Duplex)

```
Clock Cycle 1:
  ┌────────────────────────────────┐
  │ Master sends:    0x8F          │
  │ Slave responds:  0x00 (dummy)  │
  │ buffer[0] gets overwritten     │
  └────────────────────────────────┘

Clock Cycle 2:
  ┌────────────────────────────────┐
  │ Master sends:    0x00 (dummy)  │
  │ Slave responds:  0xD3 (data!)  │
  │ buffer[1] gets overwritten     │
  └────────────────────────────────┘
```

### After Transfer

```rust
buffer = [0x00, 0xD3];
         └─┬──┘ └─┬──┘
           │      │
    Slave's │      └─ Slave's actual response (WHO_AM_I value)
    dummy   │
    response
    
let who_am_i = buffer[1];  // Extract: 0xD3 = I3G4250D
```

### Why Two Bytes?

| Byte | Purpose | Why |
|------|---------|-----|
| **Byte 0** | Command byte | Tell slave what to do (read/write, register address) |
| **Byte 1** | Response byte | Slave needs time to respond, so we clock in a second byte |

The gyroscope needs at least one clock cycle to prepare the WHO_AM_I value, so we send a dummy byte to keep the clock running.

### Complete Timeline

```
Timeline:

1. Master prepares: buffer = [0x8F, 0x00]
                              ↓
2. CS goes LOW (select slave)
                              ↓
3. SPI transfer starts:
   - Clock cycle 1: Master sends 0x8F, Slave responds 0x00
   - Clock cycle 2: Master sends 0x00, Slave responds 0xD3
                              ↓
4. Buffer now contains: [0x00, 0xD3]
                              ↓
5. CS goes HIGH (deselect slave)
                              ↓
6. Extract response: who_am_i = buffer[1] = 0xD3
                              ↓
7. Identify: 0xD3 = I3G4250D ✓
```

---

## Common Confusion: Master Data vs Slave Responses

### Understanding the Vector in Tests

When you see this in test code:
```rust
let mut spi = MockSpi::new(vec![0x00, 0xD4]);
```

**This `[0x00, 0xD4]` is what the SLAVE responds with, NOT what the master sends!**

### The Complete Picture

```
┌────────────────────────────────────────────────────────────┐
│ What Master Initially Prepares                             │
└────────────────────────────────────────────────────────────┘
let mut buffer = [0x8F, 0x00];
                 └───┘ └───┘
              Command  Dummy (master's outgoing data)

┌────────────────────────────────────────────────────────────┐
│ What Slave Will Respond With                               │
└────────────────────────────────────────────────────────────┘
MockSpi::new(vec![0x00, 0xD4])
                 └───┘ └───┘
          Slave's Slave's response
          Byte 0   Byte 1
```

### Step-by-Step Communication Flow

**Step 1: Master Prepares Buffer**
```rust
let mut buffer = [0x8F, 0x00];  // Master's data to send
```

**Step 2: SPI Transfer Happens (Full-Duplex)**

```
Clock Cycle 1 (Byte 0):
  MOSI (Master Out):   0x8F ← Master sends command
  MISO (Slave Out):    0x00 ← Slave responds (dummy)
  buffer[0] ← 0x00    (buffer gets overwritten with slave's response)

Clock Cycle 2 (Byte 1):
  MOSI (Master Out):   0x00 ← Master sends dummy to keep clock
  MISO (Slave Out):    0xD4 ← Slave responds (actual data)
  buffer[1] ← 0xD4    (buffer gets overwritten with slave's response)
```

**Step 3: After Transfer**
```rust
buffer = [0x00, 0xD4]  // Now contains SLAVE's responses
who_am_i = buffer[1]   // Extract: 0xD4 = I3G4250D
```

### Key Distinction Table

| Aspect | Master | Slave |
|--------|--------|-------|
| **Initial buffer** | `[0x8F, 0x00]` | - |
| **Clock Cycle 1 sends** | 0x8F (command) | 0x00 (responds) |
| **Clock Cycle 2 sends** | 0x00 (dummy) | 0xD4 (responds) |
| **Final buffer** | `[0x00, 0xD4]` | - |
| **What we extract** | `buffer[1]` = 0xD4 | This value |

### In Test Context

```rust
#[test]
fn test_detect_l3gd20() {
    // Step 1: Prepare mock with SLAVE's responses
    let mut spi = MockSpi::new(vec![0x00, 0xD4]);
    //                               │      │
    //                   Slave responds with these bytes
    //                   (NOT what master sends!)
    
    let mut cs = MockCs::new();
    
    // Step 2: Inside detect_gyroscope(), master prepares buffer
    // (NOT the test's responsibility - happens inside the function)
    // let mut buffer = [0x8F, 0x00];  // Master sends this
    
    // Step 3: Call function (transfer happens internally)
    let result = detect_gyroscope(&mut spi, &mut cs);
    
    // Step 4: MockSpi simulates transfer by overwriting buffer
    // Before: buffer = [0x8F, 0x00]
    // After:  buffer = [0x00, 0xD4]  ← MockSpi did this
    
    // Step 5: Function extracts buffer[1] = 0xD4
    assert_eq!(result.unwrap(), GyroVariant::L3gd20);
}
```

### Remember

✅ **`MockSpi::new(vec![...])` = Slave's responses**  
✅ **`buffer = [...]` (in function) = Master's outgoing data**  
✅ **Buffer gets overwritten during transfer**  
✅ **After transfer, buffer contains slave's responses**

---

## SPI Command Protocol

### I3G4250D Register Access Format

The first byte sent to the I3G4250D follows this bit structure: ==***Section 5.2 (SPI Bus Interface)***==

```
Bit Position:  7     6     5 4 3 2 1 0
               ┌─────┬─────┬───────────┐
               │ R/W │  M  │ Address   │
               └─────┴─────┴───────────┘
                 │     │      └─ Register address (0x0F for WHO_AM_I)
                 │     └─ Auto-increment mode (1=enabled, 0=disabled)
                 └─ Read/Write (1=Read, 0=Write)
```

### Why `0x0F | 0x80`?

#### Breaking It Down:

```
0x0F = 0000 1111  (Register address: WHO_AM_I)
0x80 = 1000 0000  (Read bit at position 7)
       ─────────  (Bitwise OR)
0x8F = 1000 1111  (Complete command: Read WHO_AM_I)
       │    └────── Register 0x0F
       └────────── Read mode (1)
```

#### Step-by-Step:

```rust
const WHO_AM_I: u8 = 0x0F;              // Register address
let command = WHO_AM_I | 0x80;          // Set bit 7 to 1
//            0000 1111 | 1000 0000 = 1000 1111 = 0x8F
```

### Other Command Examples

| Operation | Register | Calculation | Result | Binary |
|-----------|----------|-------------|--------|--------|
| **Read** WHO_AM_I | 0x0F | `0x0F \| 0x80` | **0x8F** | 10001111 |
| **Read** CTRL_REG1 | 0x20 | `0x20 \| 0x80` | **0xA0** | 10100000 |
| **Write** CTRL_REG1 | 0x20 | `0x20 \| 0x00` | **0x20** | 00100000 |
| **Read** with Auto-Inc | 0x28 | `0x28 \| 0xC0` | **0xE8** | 11101000 |

### Bitwise OR Operator `|`

The `|` operator sets bits to 1:

```
  0000 1111  (0x0F)
| 1000 0000  (0x80)
= 1000 1111  (0x8F)
  ↑
  This bit became 1
```

---

## Practical Example with I3G4250D

### Reading WHO_AM_I Register

```rust
pub fn detect_gyroscope<SPI, CS, E>(spi: &mut SPI, cs: &mut CS) -> Result<GyroVariant, E>
where
    SPI: Transfer<u8, Error = E>,
    CS: OutputPin,
{
    // Step 1: Prepare command buffer
    let mut buffer = [0x0F | 0x80, 0x00];  // [0x8F, 0x00]
    
    // Step 2: Select slave (CS low)
    cs.set_low().ok();
    
    // Step 3: Full-duplex transfer
    // Master sends:  [0x8F,        0x00]
    // Slave sends:   [0x00,        0xD3]
    // Buffer becomes:[0x00,        0xD3]
    let result = spi.transfer(&mut buffer);
    
    // Step 4: Deselect slave (CS high)
    cs.set_high().ok();
    
    // Step 5: Handle errors
    result?;
    
    // Step 6: Extract response
    let who_am_i = buffer[1];  // 0xD3
    
    // Step 7: Identify variant
    match who_am_i {
        0xD3 => Ok(GyroVariant::I3g4250d),  // I3G4250D
        0xD4 => Ok(GyroVariant::L3gd20),    // L3GD20
        0xD7 => Ok(GyroVariant::L3gd20h),   // L3GD20H
        other => Ok(GyroVariant::Unknown(other)),
    }
}
```

### Known Device Responses

| Device | WHO_AM_I Value | Hex | Binary |
|--------|----------------|-----|--------|
| **I3G4250D** | 211 | 0xD3 | 11010011 |
| **L3GD20** | 212 | 0xD4 | 11010100 |
| **L3GD20H** | 215 | 0xD7 | 11010111 |

---

## Testing with Mock Objects

### Why We Mock

In tests, we can't use real hardware, so we simulate SPI responses:

```rust
struct MockSpi {
    response: Vec<u8>,  // What the slave will respond with
}

impl Transfer<u8> for MockSpi {
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        // Overwrite buffer with slave's responses (simulating full-duplex)
        for (i, byte) in self.response.iter().enumerate() {
            words[i] = *byte;  // ← Simulates slave sending data
        }
        Ok(words)
    }
}
```

### Test Example

```rust
#[test]
fn test_detect_i3g4250d() {
    // Step 1: Create mock that simulates I3G4250D responses
    let mut spi = MockSpi::new(vec![0x00, 0xD3]);
    //                            └───┘  └───┘
    //                         Slave's responses
    //                         Byte0   Byte1 (WHO_AM_I)
    
    let mut cs = MockCs::new();
    
    // Step 2: Call the function
    let result = detect_gyroscope(&mut spi, &mut cs);
    
    // Step 3: Verify the buffer gets overwritten correctly
    // Before:  buffer = [0x8F, 0x00]
    // After:   buffer = [0x00, 0xD3]  ← MockSpi overwrites it
    // Extract: buffer[1] = 0xD3 = I3G4250D ✓
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), GyroVariant::I3g4250d);
}
```

### How the Mock Simulates Full-Duplex

```rust
// During spi.transfer(&mut buffer):
let mut buffer = [0x8F, 0x00];

// MockSpi does this:
for (i, byte) in response.iter().enumerate() {  // response = [0x00, 0xD3]
    buffer[i] = *byte;
}
// buffer[0] = 0x00  (slave's response to master's 0x8F)
// buffer[1] = 0xD3  (slave's response to master's 0x00)

// Result:
buffer = [0x00, 0xD3]  ✓
```

---

## Summary

### Key Concepts

1. **SPI is Full-Duplex**: Master and Slave communicate simultaneously
2. **Buffer Gets Overwritten**: After transfer, the buffer contains slave responses
3. **Command Byte Format**: `register_address | 0x80` for read operations
4. **Two Bytes Needed**: First byte for command, second byte for response
5. **CS Protocol**: LOW before transfer, HIGH after transfer

### Memory Aid

```
Before Transfer:    buffer = [command,  dummy]
                              ↓         ↓
Master Sends:                [0x8F,   0x00]
Slave Responds:              [0x00,   0xD3]
                              ↓         ↓
After Transfer:    buffer = [0x00,   0xD3]
                             (ignore) (extract this!)
```

### Testing Approach

- **Mock the slave responses** in tests
- **Verify buffer gets overwritten** correctly
- **Extract the response byte** (usually buffer[1])
- **Match against known values** to identify the device

---

## References

- I3G4250D Datasheet: Register 0x0F (WHO_AM_I)
- SPI Specification: Full-duplex communication mode
- STM32F3 HAL Documentation: `embedded_hal::blocking::spi::Transfer`

---
Author: Tan Dao



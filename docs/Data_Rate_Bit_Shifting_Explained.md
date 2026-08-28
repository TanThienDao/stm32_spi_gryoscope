# Data Rate Bit-Shifting Explained (`<< 6`)

## Overview

The I3G4250D gyroscope uses **CTRL_REG1** to configure various settings, including the data rate. The data rate is controlled by bits 7-6 of this register, requiring us to position a 2-bit value at the correct bit locations using bit-shifting.

![ctrl_reg1_i3g4250d_1.png](/docs/Screenshots/ctrl_reg1_i3g4250d_1.png)

![ctrl_reg1_i3g4250d_2.png](/docs/Screenshots/ctrl_reg1_i3g4250d_2.png)
---

## What is the `<<` Operator?

The `<<` is a **left bit-shift operator** in Rust. It shifts all bits to the left by a specified number of positions, filling in zeros on the right.

### Basic Example:
```rust
0b0011 << 2  // Result: 0b1100
```

**Step-by-step:**
- Original: `0011` (binary) = 3 (decimal)
- Shift left 2 positions: `1100` (binary) = 12 (decimal)
- Each bit moves 2 positions to the left
- New positions on the right are filled with 0s

---

## Why `<< 6` for Data Rate?

The I3G4250D's **CTRL_REG1** register has this bit layout:

```
Bit Position:  7  6  5  4  3  2   1   0
Register:     [DR DR BW BW PD Xen Yen Zen]
              └────┘  └──┘  └──────────┘
              Data   Band- Power &
              Rate   width Axes
```

**Data Rate (DR) is located at bits [7:6]** — we need the 2-bit value positioned there.

Since we're defining just the 2-bit value (00, 01, 10, or 11), we must shift it left by 6 positions to place it at bits [7:6].

---

## Data Rate Constants Breakdown

### Constant Definition:
```rust
pub const DR_100_HZ: u8 = 0b00 << 6;   // 100 Hz
pub const DR_200_HZ: u8 = 0b01 << 6;  // 200 Hz
pub const DR_400_HZ: u8 = 0b10 << 6;  // 400 Hz
pub const DR_800_HZ: u8 = 0b11 << 6;  // 800 Hz
```

### Detailed Breakdown for Each Rate:

#### DR_100_HZ
```
Value:        0b00          (the 2-bit data rate code for 100 Hz)
Shift:        << 6          (move 6 positions left)
Binary:       0b00000000
Hexadecimal:  0x00
Decimal:      0

Register Placement:
Bit Position:  7  6  5  4  3  2  1  0
CTRL_REG1:    [0  0  ?  ?  ?  ?  ?  ?]
              └────┘
              DR = 00 (100 Hz)
```

#### DR_200_HZ
```
Value:        0b01          (the 2-bit data rate code for 200 Hz)
Shift:        << 6          (move 6 positions left)

Before shift:
Binary:       0b00000001
              └──┘
              The value we want to position

After shift:
Binary:       0b01000000    (moved 6 positions left)
Hexadecimal:  0x40
Decimal:      64

Register Placement:
Bit Position:  7  6  5  4  3  2  1  0
CTRL_REG1:    [0  1  ?  ?  ?  ?  ?  ?]
              └────┘
              DR = 01 (200 Hz)
```

#### DR_400_HZ
```
Value:        0b10          (the 2-bit data rate code for 400 Hz)
Shift:        << 6

Before shift:
Binary:       0b00000010
              
After shift:
Binary:       0b10000000    (moved 6 positions left)
Hexadecimal:  0x80
Decimal:      128

Register Placement:
Bit Position:  7  6  5  4  3  2  1  0
CTRL_REG1:    [1  0  ?  ?  ?  ?  ?  ?]
              └────┘
              DR = 10 (400 Hz)
```

#### DR_800_HZ
```
Value:        0b11          (the 2-bit data rate code for 800 Hz)
Shift:        << 6

Before shift:
Binary:       0b00000011

After shift:
Binary:       0b11000000    (moved 6 positions left)
Hexadecimal:  0xC0
Decimal:      192

Register Placement:
Bit Position:  7  6  5  4  3  2  1  0
CTRL_REG1:    [1  1  ?  ?  ?  ?  ?  ?]
              └────┘
              DR = 11 (800 Hz)
```

---

## Using Data Rate in `set_data_rate()`

The `set_data_rate()` function shows how these shifted values are used:

```rust
pub fn set_data_rate(&mut self, rate: DataRate) -> Result<(), &'static str> {
    let current = self.read_register(CTRL_REG1)?;
    let new_val = (current & 0x3F) | rate.ctrl_reg1_bits();  // Preserve bits [5:0]
    self.write_register(CTRL_REG1, new_val)
}
```

### Step-by-Step Explanation:

#### 1. Read Current Register
```rust
let current = self.read_register(CTRL_REG1)?;
```
- Reads the existing CTRL_REG1 value (may have other settings like BW, PD, Xen, Yen, Zen)

#### 2. Mask to Preserve Other Bits
```rust
(current & 0x3F)
```

**What is 0x3F?**

0x3F is a **mask** used to selectively clear bits:
```
0x3F in binary: 0b00111111
                   └───────┘
                   1s here = preserve bits
                   0s here = clear bits
```

**How the AND operation works:**

When you AND a value with this mask, it clears bits 7-6 and preserves bits 5-0:
```
Example with actual values:
Current register: 0b11110101
Mask (0x3F):      0b00111111
                  ──────────
Result:           0b00110101
                  └────┘
                  Bits 7-6 cleared to 0
                  Bits 5-0 preserved
```

**Bit-by-bit breakdown:**
```
Bit Position:  7  6  5  4  3  2  1  0
Current:       1  1  1  1  0  1  0  1
Mask:          0  0  1  1  1  1  1  1
Result:        0  0  1  1  0  1  0  1
               └────┘
               Cleared   Preserved
```

This operation clears bits 7-6 (the data rate bits) while preserving bits 5-0 (BW, PD, Xen, Yen, Zen) which contain other important settings.

#### 3. Get Data Rate Bits
```rust
rate.ctrl_reg1_bits()
```
- Returns the shifted data rate value (already positioned at bits 7-6)

#### 4. Combine with Bitwise OR
```rust
(current & 0x3F) | rate.ctrl_reg1_bits()
```

**Example:**
```
Current CTRL_REG1:   0b??110101   (? = data rate bits we want to clear)
After mask (0x3F):   0b00110101   (bits 7-6 cleared)
DR_400_HZ (0b10<<6): 0b10000000   (data rate at bits 7-6)
After OR:            0b10110101   (new data rate, other bits preserved)
                     └────┘
                     New data rate (400 Hz)
```

---

## Complete Example

### Setting 400 Hz Data Rate:

```rust
let mut gyro = GyroDriver::new(spi, cs);
gyro.set_data_rate(DataRate::Hz400)?;
```

**What happens internally:**

1. `DataRate::Hz400.ctrl_reg1_bits()` returns:
   ```
   DR_400_HZ = 0b10 << 6 = 0b10000000 = 0x80
   ```

2. Read current CTRL_REG1 (assume: `0b00110101`)

3. Clear data rate bits:
   ```
   0b00110101 & 0b00111111 = 0b00110101
   ```

4. Insert new data rate:
   ```
   0b00110101 | 0b10000000 = 0b10110101
   ```

5. Write back to register: `0b10110101`
   ```
   Bit 7-6: 10 → 400 Hz data rate
   Bits 5-0: 110101 → Original values preserved
   ```

---

## Why This Design?

1. **Modularity**: Data rate can be changed independently without affecting other register settings
2. **Hardware Requirement**: The I3G4250D datasheet specifies bits 7-6 for data rate
3. **Bit Efficiency**: Only 2 bits needed for 4 options (00, 01, 10, 11)
4. **Protection**: Masking prevents accidentally overwriting other register bits

---

## Key Takeaways

- `<< 6` shifts the 2-bit value 6 positions left
- This positions the data rate code at bits [7:6] where the hardware expects it
- The four data rates (100, 200, 400, 800 Hz) are encoded as 00, 01, 10, 11
- `0x3F` mask (0b00111111) clears only bits 7-6, preserving other register settings
- The `|` operator combines the masked register with the shifted data rate value


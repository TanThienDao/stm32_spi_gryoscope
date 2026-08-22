# STM32F3 Discovery SPI Gyroscope Project

A Rust embedded systems project for identifying and communicating with 3-axis gyroscope sensors (L3GD20, I3G4250D, L3GD20H) connected via SPI to an STM32F3 Discovery board.

## 📋 Project Overview

This project demonstrates:
- **SPI Communication** - Full-duplex SPI protocol implementation (Mode 3)
- **Device Identification** - WHO_AM_I register reading to detect gyroscope models
- **Embedded Rust** - Using `cortex-m`, `embedded-hal`, and `stm32f3xx-hal` crates
- **Error Handling** - Proper error management in embedded contexts
- **ITM Debugging** - Real-time debug output via ARM Instrumentation Trace Macrocell

## 🎯 Supported Devices

| Device | WHO_AM_I | Variant |
|--------|----------|---------|
| **L3GD20** | `0xD4` | Original 3-axis gyroscope |
| **L3GD20H** | `0xD7` | High-performance variant |
| **I3G4250D** | `0xD3` | Alternative 3-axis gyroscope |

## 🛠️ Hardware Setup

### Required Components
- STM32F3-Discovery board
- L3GD20 / I3G4250D / L3GD20H gyroscope module
- ST-Link V2 debugger (for flashing and debugging)
- USB cable for power and debugging

### Wiring Configuration

#### SPI Pins (SPI1)
| Signal | STM32F3 Pin | Function |
|--------|------------|----------|
| **SCK** | PA5 | SPI Clock |
| **MISO** | PA6 | Master In, Slave Out |
| **MOSI** | PA7 | Master Out, Slave In |
| **CS** | PE3 | Chip Select (Active Low) |

#### Power & Ground
- **VCC** → 3.3V
- **GND** → Ground

### SPI Mode Configuration
- **Polarity (CPOL)**: Idle High
- **Phase (CPHA)**: Capture on Second Transition
- **Clock Speed**: 1 MHz
- **Mode**: SPI Mode 3

## 🚀 Getting Started

### Prerequisites
```bash
# Install Rust and embedded tools
rustup target add thumbv7em-none-eabihf
cargo install cargo-embed
```

### Building the Project
```bash
# Build for release (optimized)
cargo build --release

# Build for debug
cargo build
```

### Flashing to Hardware
```bash
# Using cargo-embed
cargo embed --release

# Or with OpenOCD manually
openocd -f openocd.gdb
```

## 💡 Code Usage

### Main Function (`detect_gyroscope`) - Recommended

The `detect_gyroscope` function is the **recommended** approach. It:
- Takes ownership of SPI and CS pin references
- Returns a `GyroVariant` enum
- Handles errors gracefully
- Doesn't call `Peripherals::take()` inside the function

```rust
use auxiliary::*;

#[entry]
fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs) = init();
    
    iprintln!(&mut itm.stim[0], "Gyroscope initialization starting...");
    
    match detect_gyroscope(&mut spi, &mut cs) {
        Ok(variant) => {
            iprintln!(&mut itm.stim[0], "Found Gyroscope Variant: {:?}", variant);
        }
        Err(_) => {
            iprintln!(&mut itm.stim[0], "Error detecting gyroscope variant");
        }
    }
    
    loop {}
}
```

### Available Variants
```rust
pub enum GyroVariant {
    I3g4250d,
    L3gd20,
    L3gd20h,
    Unknown(u8),
}
```

## 📡 SPI Protocol

### WHO_AM_I Register Read Sequence

1. **Pull CS Low** - Select the device
2. **Send Address Byte** - `0x0F | 0x80` (Register address with read bit)
3. **Receive Data** - Device responds with ID byte
4. **Pull CS High** - Deselect the device
5. **Match Result** - Compare ID against known values

### Example SPI Transaction
```
Buffer Before:  [0x8F, 0x00]  (address with read bit, dummy)
Buffer After:   [0x8F, 0xD4]  (echoed address, device ID)
```

## ⚠️ Troubleshooting

### Panic: "called `Option::unwrap()` on a `None` value"

**Cause:** The older `identify_gryoscope()` function calls `Peripherals::take().unwrap()` which fails on subsequent calls because `Peripherals` is a singleton.

**Solution:** Use `detect_gyroscope()` instead, which doesn't have this issue.

**Why it happens:**
- `Peripherals::take()` returns `Option<Peripherals>` 
- First call: Returns `Some(Peripherals)` ✓
- Second call: Returns `None` ✗ (already taken)
- `.unwrap()` on `None` causes panic

### No Device Response

**Possible causes:**
1. Wiring connections are incorrect
2. CS pin not properly toggled
3. SPI mode mismatch (should be Mode 3)
4. Device not powered or defective

**Debugging steps:**
1. Verify all pin connections match the wiring table
2. Use a logic analyzer to check SPI signals
3. Confirm device power supply (3.3V)
4. Test with a known-good gyroscope device

### ITM Output Not Appearing

1. Ensure ST-Link debugger is properly connected
2. Check ITM configuration in `.gdb` script
3. Verify `openocd.gdb` has correct device settings
4. Try restarting the debug session

## 📚 Project Structure

```
stm32_spi_gryoscope/
├── .cargo/
│   └── config.toml       # Cargo configuration for embedded targets
├── .github/
│   └── workflows/
│       └── build.yml     # GitHub Actions build workflow
├── src/
│   └── main.rs           # Entry point, device identification logic
├── auxiliary/
│   ├── Cargo.toml        # Auxiliary crate dependencies
│   └── src/
│       └── lib.rs        # SPI initialization, gyroscope detection
├── docs/                 # Documentation and technical notes (My Notes)
│   └── FIXES.md          # Detailed technical fixes applied
├── Cargo.toml            # Main project manifest
├── Cargo.lock            # Dependency lock file
├── openocd.gdb           # OpenOCD debug configuration
└── README.md             # This file
```

## 🔧 Key Functions

### `init() -> (ITM, Delay, Spi, OutputPin)`
Initializes the STM32F3 hardware:
- Configures RCC (clock system)
- Sets up GPIO ports (PA5-PA7 for SPI, PE3 for CS)
- Configures SPI1 peripheral
- Returns ITM (for debugging), Delay timer, SPI, and CS pin

### `detect_gyroscope<SPI, CS>(spi: &mut SPI, cs: &mut CS) -> Result<GyroVariant, E>`
Identifies the connected gyroscope:
- Pulls CS low
- Sends WHO_AM_I read command
- Pulls CS high
- Compares response against known IDs
- Returns `GyroVariant` enum

**Generic Parameters:**
- `SPI`: Must implement `Transfer<u8>` trait
- `CS`: Must implement `OutputPin` trait

## 📖 Technical Details

For detailed information about fixes applied, compilation issues resolved, and design decisions, see [FIXES.md](docs/FIXES.md).

### Key Technical Points:
- **Trait Versions**: embedded_hal 0.2.x vs 1.0.x compatibility
- **HAL Methods**: Using `transfer()` instead of missing `write_read()`
- **Static Lifetimes**: Return type requires `&'static str` for error messages
- **SPI Mode 3**: Specific polarity/phase configuration for L3GD20

## 🔄 Testing Workflow

```bash
# 1. Build the project
cargo build --release

# 2. Flash to device
cargo embed --release

# 3. Monitor ITM output (in terminal 1)
# 4. Expected output:
#    "Gyroscope initialization starting..."
#    "Found Gyroscope Variant: L3gd20"  (or other variant)
```

## 📝 Expected Output

### Successful Detection
```
Gyroscope initialization starting...
Found Gyroscope Variant: I3g4250d
```

### Error Cases
```
Gyroscope initialization starting...
Error detecting gyroscope variant
```

## 🚦 Status

✅ **Working Features:**
- SPI communication with gyroscope
- Device identification via WHO_AM_I register
- ITM debug output
- Error handling

🎯 **Future Improvements:**
- Read gyroscope sensor data (X, Y, Z axes)
- Implement sensor calibration
- Add temperature monitoring
- Sensor fusion algorithms
- Create higher-level driver library

## 🔗 References

- [ARM Cortex-M Documentation](https://developer.arm.com/documentation/dui0553/a/)
- [RM0316 Reference Manual](https://www.st.com/resource/en/reference_manual/dm00043574.pdf#page=244&zoom=100,89,482)
- [STM32F3 Discovery Datasheet](https://www.st.com/resource/en/datasheet/stm32f303vc.pdf)
- Stupid 3 Gyroscopes: (Who Stupid enough create 3 separate datasheets for the same device?!) Pain in the ass to figure out which one is correct.
  - [L3GD20 Datasheet](https://www.st.com/resource/en/datasheet/l3gd20.pdf)
  - [L3GD20H Datasheet](https://www.mouser.com/datasheet/2/389/l3gd20h-954865.pdf?srsltid=AfmBOooLrBLER3RO5216hv_0QlzspRzKh5eitGTZq52QD1IQ3ziU3iln)
  - [I3G4250D Datasheet](https://www.st.com/resource/en/datasheet/i3g4250d.pdf)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [embedded-hal Documentation](https://docs.rs/embedded-hal/latest/embedded_hal/)

## 📄 License

This project is part of an embedded Rust learning Journey.

## 👤 Author

Tan Dao

---

**Last Updated:** August 2026  
**Status:** ✅ Production Ready


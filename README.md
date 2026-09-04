# STM32F3 Discovery SPI Gyroscope Project

A Rust embedded systems project for interrupt-driven 3-axis gyroscope sensor (I3G4250D) communication via SPI on an STM32F3 Discovery board with real-time performance monitoring.
***
## 📋 Project Overview

This project demonstrates:
- **Interrupt-Driven Design** - TIM2 timer generates 400 Hz interrupts for periodic sensor sampling
- **SPI Communication** - Full-duplex SPI protocol implementation (Mode 3) with DMA-free transfers
- **Device Identification** - WHO_AM_I register reading to detect gyroscope models (L3GD20, I3G4250D, L3GD20H)
- **Low-Power Operation** - CPU sleeps during intervals, reducing power consumption to ~5% idle time
- **Real-Time Monitoring** - CPU usage tracking via ARM Cortex-M4 DWT cycle counter
- **Error Handling** - Robust error management and anomaly detection
- **ITM Debugging** - Real-time debug output via ARM Instrumentation Trace Macrocell
- **Embedded Rust** - Using `cortex-m`, `embedded-hal`, `cortex-m-rt`, and `stm32f3xx-hal` crates
***
## 🎯 Supported Devices

| Device | WHO_AM_I | Status | Features |
|--------|----------|--------|----------|
| **I3G4250D** | `0xD3` | ✅ Fully Implemented | Primary device, configurable range & data rate |
| **L3GD20** | `0xD4` | ✅ Detected | Identification only (requires driver implementation) |
| **L3GD20H** | `0xD7` | ✅ Detected | Identification only (requires driver implementation) |

**Note:** The project is currently optimized for **I3G4250D**. Support for L3GD20/L3GD20H requires driver enhancements.
***
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


### SPI Mode Configuration
- **Polarity (CPOL)**: Idle High
- **Phase (CPHA)**: Capture on Second Transition
- **Clock Speed**: 1 MHz
- **Mode**: SPI Mode 3

### TIM2 Timer Configuration (Interrupt-Driven Sampling)
| Parameter | Value | Purpose |
|-----------|-------|---------|
| **CPU Clock** | 72 MHz | STM32F303VC system clock |
| **Prescaler (PSC)** | 719 | Divides 72 MHz to 100 kHz timer frequency |
| **Auto-Reload (ARR)** | 249 | Defines interrupt period (2.5 ms) |
| **Interrupt Frequency** | 400 Hz | Gyroscope sensor sampling rate |
| **IRQ Number** | 35 | NVIC interrupt request line for TIM2 |
| **Timer Tick** | 10 µs | Resolution of timer counter |

**Calculation:**
```
Timer Frequency = CPU Clock / (PSC + 1) = 72 MHz / 720 = 100 kHz
Interrupt Period = (ARR + 1) / Timer Frequency = 250 / 100 kHz = 2.5 ms
Interrupt Frequency = 1 / 2.5 ms = 400 Hz
```
***
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
***
## 📡 System Architecture

### Interrupt-Driven Data Flow

```
┌─────────────────────────────────────────────────────────┐
│ TIM2 Timer (72 MHz CPU clock)                           │
│  ├─ Prescaler: Divides 72 MHz → 100 kHz                 │
│  ├─ Auto-Reload: Counts 0-249 (250 ticks)               │
│  └─ Period: 2.5 ms (every 250 × 10 µs)                  │
└─────────────────────┬───────────────────────────────────┘
                      │ Interrupt fires every 2.5 ms
                      ▼
┌─────────────────────────────────────────────────────────┐
│ TIM2 ISR Handler                                        │
│  ├─ Sets NEW_DATA_READY flag                            │
│  ├─ Clears Update Interrupt Flag (UIF)                  │
│  └─ Total execution time: ~1 µs                         │
└─────────────────────┬───────────────────────────────────┘
                      │ Signal sent
                      ▼
┌─────────────────────────────────────────────────────────┐
│ Main Loop (Low Power)                                   │
│  ├─ Sleeps 95% of the time with wfe()                   │
│  ├─ Wakes on interrupt                                  │
│  ├─ Reads gyroscope data via SPI                        │
│  ├─ Monitors CPU usage (DWT cycle counter)              │
│  └─ Prints statistics every ~2.5 seconds                │
└─────────────────────────────────────────────────────────┘
```

### SPI Protocol (WHO_AM_I Register Read)

**Sequence:**
1. Pull CS Low - Select the device
2. Send Address Byte - `0x0F | 0x80` (Register address with read bit)
3. Receive Data - Device responds with ID byte
4. Pull CS High - Deselect the device
5. Match Result - Compare ID against known values

**Full-Duplex Transfer Example:**
```
Buffer Before:  [0x8F, 0x00]  (address with read bit, dummy)
Buffer After:   [0x8F, 0xD3]  (echoed address, device ID for I3G4250D)
```
***
## 📚 Project Structure

```
stm32_spi_gryoscope/
├── .cargo/
│   └── config.toml                 # Cargo configuration for embedded targets
├── .github/
│   └── workflows/
│       └── build.yml               # GitHub Actions build workflow
├── src/
│   └── main.rs                     # Main entry point with interrupt-driven loop
├── auxiliary/
│   ├── Cargo.toml                  # Auxiliary crate dependencies
│   └── src/
│       ├── lib.rs                  # Hardware initialization (RCC, GPIO, SPI, TIM2, NVIC)
│       ├── gyro_driver.rs          # I3G4250D gyroscope driver with SPI communication
│       └── interrupt_handler.rs    # TIM2 ISR and NEW_DATA_READY synchronization
├── docs/
│   ├── ...           
│   └── RTOS/
│       └── TIMx/
│           ├── TIM2_Interrupt_Guide.md     # TIM2 configuration guide
│           ├── ...
│           └── ...
├── Cargo.toml                      # Main project manifest
├── Cargo.lock                      # Dependency lock file
├── openocd.gdb                     # OpenOCD debug configuration
└── README.md                       # This file
```
***
## 🔧 Key Functions and Structs

### Timer Configuration (`Tim2Guard`)
```rust
pub struct Tim2Guard {
    initialized: bool,
}

impl Tim2Guard {
    pub fn new() -> Self                              // Create new guard
    pub fn init(&mut self, tim2: &mut TIM2)           // Initialize TIM2 once
    pub fn config_tim2(&mut self, psc: u16, arr: u32) // Configure timer registers
    pub fn calculate_timer_values(...) -> (u16, u32)  // Calculate PSC and ARR
    pub fn check_and_clear_uif() -> bool              // Clear interrupt flag safely
}
```

**Configuration Formula:**
```
PSC = (CPU_Clock / Timer_Frequency) - 1
ARR = (Timer_Frequency / Interrupt_Frequency) - 1
```

### NVIC Management (`NvicGuard`)
```rust
pub struct NvicGuard {
    initialized: bool,
}

impl NvicGuard {
    pub fn new() -> Self                           // Create new guard
    pub fn unmask_tim2_safe(&mut self) -> Result   // Unmask TIM2 interrupt (IRQ 35)
}
```

### Gyroscope Driver (`GyroDriver`)
```rust
pub struct GyroDriver<SPI, CS> {
    spi: SPI,
    cs: CS,
    // ... internal state ...
}

impl<SPI, CS> GyroDriver<SPI, CS> {
    pub fn new(spi: SPI, cs: CS) -> Self                              // Create driver
    pub fn init(&mut self) -> Result<(), &'static str>                // Initialize sensor
    pub fn who_am_i(&mut self) -> Result<u8, Error>                  // Read ID register
    pub fn set_data_rate(&mut self, rate: DataRate) -> Result        // Set sampling rate
    pub fn set_range(&mut self, range: Range) -> Result              // Set sensitivity
    pub fn read_angular_velocity(&mut self) -> Result<(f32, f32, f32)>  // Read X,Y,Z values
}
```

### Interrupt Handler
```rust
// Global flag synchronized between ISR and main loop
pub static NEW_DATA_READY: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

// TIM2 Interrupt Service Routine (fires every 2.5 ms)
#[no_mangle]
pub extern "C" fn TIM2() {
    // Clear interrupt flag
    auxiliary::Tim2Guard::check_and_clear_uif();
    
    // Signal main loop that data is ready
    cortex_m::interrupt::free(|cs| {
        *NEW_DATA_READY.borrow(cs).borrow_mut() = true;
    });
}
```

### Hardware Initialization
```rust
pub fn init() -> (ITM, Delay, Spi, OutputPin, DWT) {
    // Initializes:
    // - RCC (clock system)
    // - GPIO (SPI pins and CS)
    // - SPI1 (1 MHz, Mode 3)
    // - TIM2 (72 MHz → 100 kHz → 400 Hz interrupts)
    // - Returns: ITM (debug), Delay, SPI, CS pin, DWT (cycle counter)
}
```

### Device Detection
```rust
pub fn detect_gyroscope<SPI, CS, E>(spi: &mut SPI, cs: &mut CS) -> Result<GyroVariant, E>
where
    SPI: Transfer<u8, Error = E>,
    CS: OutputPin,
{
    // Reads WHO_AM_I register (0x0F)
    // Returns: I3g4250d, L3gd20, L3gd20h, or Unknown(u8)
}
```
***
## 📖 Technical Details

For detailed information about fixes applied, compilation issues resolved, and design decisions, see [FIXES.md](docs/FIXES.md).

### Key Technical Points:
- **Trait Versions**: embedded_hal 0.2.x vs 1.0.x compatibility
- **HAL Methods**: Using `transfer()` instead of missing `write_read()`
- **Static Lifetimes**: Return type requires `&'static str` for error messages
- **SPI Mode 3**: Specific polarity/phase configuration for L3GD20

### Measurement Points

**DWT Cycle Counter Metrics:**
- Avg Cycles/Loop: Clock cycles per main loop iteration
- Loop Time (μs): Time spent in active processing
- CPU Usage (%): Percentage of time CPU is actively running

**Formula:**
```
CPU Usage = (Average Loop Time / Interrupt Period) × 100%
          = (Average Loop Time / 2.5 ms) × 100%

At 400 Hz:
- Expected: ~5-10% when reading and processing data
- Higher: May indicate SPI timing issues or excessive processing
```
***
## 📝 Expected Output

### Successful Initialization and Data Collection
```
===============================
I3G4250D Gyroscope
===============================

Step 1: Detecting gyroscope...
✓ Found: I3g4250d

Step 2: Initializing driver...
✓ WHO_AM_I: 0xD3

Step 3: Configuring sensor...
✓ Configuration complete:
  - Data Rate: 400 Hz (timer interrupt)
  - Range: 500 °/s

Step 4: Starting interrupt-driven mode...
──────────────────────────────────────

X:   12.34°/s | Y:   -5.67°/s | Z:    0.89°/s
X:   12.45°/s | Y:   -5.72°/s | Z:    0.91°/s
X:   12.38°/s | Y:   -5.69°/s | Z:    0.88°/s

📊 Interrupt Stats (every 1000 loops / ~2.5s):
  Avg Cycles/Loop: 15840
  Loop Time: 220.000μs
  Anomalies: 0
  CPU Usage: 8.80%
──────────────────────────────────────

X:   12.52°/s | Y:   -5.75°/s | Z:    0.92°/s
...
```

### Expected Behavior
- **Data Rate:** New readings every 2.5 ms (400 Hz)
- **CPU Usage:** ~5-10% during normal operation
- **Anomaly Detection:** Flags if sensor value changes exceed threshold
- **Statistics:** Printed every ~2.5 seconds (1000 interrupt cycles)
***
## 🔗 References

### Official Documentation
- **RM0316 STM32F303 Reference Manual** ⭐ REQUIRED
- **STM32F303VC Datasheet** (DS10190)
- **ARM Cortex-M4 Devices Generic User Guide** (Optional)

### Rust & Embedded Resources
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [cortex-m Crate Documentation](https://docs.rs/cortex-m/)
- [embedded-hal Traits](https://docs.rs/embedded-hal/latest/embedded_hal/)
- [stm32f3-discovery HAL](https://docs.rs/stm32f3-discovery/)

### Gyroscope Datasheets
- [I3G4250D Datasheet](https://www.st.com/resource/en/datasheet/i3g4250d.pdf)
- [L3GD20 Datasheet](https://www.st.com/resource/en/datasheet/l3gd20.pdf)
- [L3GD20H Datasheet](https://www.mouser.com/datasheet/2/389/l3gd20h-954865.pdf)

## 📄 License
MIT License - See [LICENSE](LICENSE) for details.

## 👤 Author
Tan Dao
---
**Last Updated:** September 2026


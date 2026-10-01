# STM32F3 Discovery SPI Gyroscope Project - (RTIC)

A Rust embedded systems project for RTIC-based multi-task 3-axis gyroscope sensor (I3G4250D) communication via SPI on an STM32F3 Discovery board with real-time performance monitoring and per-task metrics.
***
## 📋 Project Overview

This project demonstrates:
- **RTIC Framework** - Real-Time Interrupt-driven Concurrency for deterministic multi-task execution
- **Interrupt-Driven Design** - TIM2 timer generates 400 Hz interrupts for periodic sensor sampling
- **SPI Communication** - Full-duplex SPI protocol implementation (Mode 3) with DMA-free transfers
- **Device Identification** - WHO_AM_I register reading to detect gyroscope models (L3GD20, I3G4250D, L3GD20H)
- **Low-Power Operation** - CPU sleeps during intervals, reducing power consumption to ~0.26% active time
- **Real-Time Monitoring** - Per-task CPU usage tracking via ARM Cortex-M4 DWT cycle counter
- **Accurate Measurements** - Separate tracking of `read_sensor` (high-priority ISR) and `process_data` (low-priority task)
- **Error Handling** - Robust error management and anomaly detection
- **ITM Debugging** - Real-time debug output via ARM Instrumentation Trace Macrocell
- **Embedded Rust** - Using `cortex-m`, `embedded-hal`, `cortex-m-rt`, `rtic`, and `stm32f3xx-hal` crates
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
## 📡 System Architecture (RTIC-Based)

### Multi-Task Interrupt-Driven Data Flow

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
│ HIGH PRIORITY Task: read_sensor (binds to TIM2)         │
│  ├─ Runs immediately on interrupt                       │
│  ├─ Reads gyroscope via SPI (~16.2 μs)                  │
│  ├─ Updates shared sensor_data                          │
│  ├─ Spawns process_data task                            │
│  └─ Execution: 400 times per 2.5 seconds               │
└─────────────────────┬───────────────────────────────────┘
                      │ Spawns async task
                      ▼
┌─────────────────────────────────────────────────────────┐
│ LOW PRIORITY Task: process_data (async, priority=1)     │
│  ├─ Processes gyroscope data                            │
│  ├─ Checks for anomalies                                │
│  ├─ Monitors consistency (~26.6 μs)                     │
│  ├─ Prints statistics every 1000 calls                  │
│  └─ Execution: 400 times per 2.5 seconds               │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ IDLE Task (Lowest Priority)                             │
│  ├─ Runs when no tasks pending                          │
│  ├─ Executes wfi() (Wait For Interrupt)                 │
│  └─ CPU sleeps 99.74% of the time                       │
└─────────────────────────────────────────────────────────┘

RTIC Scheduler: Ensures high-priority tasks preempt lower-priority ones
Result: Deterministic, real-time execution with minimal latency!
```

### RTIC Task Priority Hierarchy

| Task | Priority | Trigger | Execution Time |
|------|----------|---------|---|
| **read_sensor** | Hardware (highest) | TIM2 interrupt | ~16.2 μs |
| **process_data** | 1 | Spawned by read_sensor | ~26.6 μs |
| **idle** | 0 (lowest) | No pending tasks | CPU sleep |

### Shared Resources & Synchronization

```rust
#[shared]
struct Shared {
    sensor_data: (f32, f32, f32),           // X, Y, Z angular velocities
    new_data_ready: bool,                   // Data availability flag
    read_sensor_total_cycles: u32,          // Cumulative execution cycles
    read_sensor_count: u32,                 // Number of executions
    process_data_total_cycles: u32,         // Cumulative execution cycles
    process_data_count: u32,                // Number of executions
}
```

All shared resources are automatically protected by RTIC's lock mechanism (critical sections).


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
├── .gitignore                      # Git ignore rules
├── src/
│   └── main.rs                     # RTIC app with read_sensor & process_data tasks
├── auxiliary/                      # Support library crate for hardware abstraction
│   ├── Cargo.toml                  # Auxiliary crate manifest and dependencies
│   ├── src/
│   │   ├── lib.rs                  # Library root with public module exports
│   │   ├── gyro_driver.rs          # I3G4250D gyroscope driver with SPI communication
│   │   ├── interrupt_handler.rs    # RTIC interrupt coordination
│   │   ├── nvic.rs                 # NVIC interrupt controller management
│   │   └── timer/
│   │       ├── mod.rs              # Timer module exports
│   │       └── tim2.rs             # TIM2 timer configuration (prescaler, ARR, ISR handling)
│   └── tests/
│       └── integration_tests.rs    # Integration tests for hardware initialization
├── docs/                           # Comprehensive technical documentation
│   ├── ...
│   ├── SPI_Full_Duplex_Explained.md # Full-duplex SPI communication protocol
│   └── Screenshots/                # Hardware configuration and datasheet references
├── Cargo.toml                      # Main project manifest with dependencies
├── Cargo.lock                      # Locked dependency versions
├── LICENSE                         # MIT License
├── openocd.gdb                     # OpenOCD debugger configuration
├── openocd.cfg                     # OpenOCD hardware configuration
├── README.md                       # This file
├── *.log                           # Debug and instrumentation trace logs
└── .git/                           # Git repository
```
***
## 🔧 Key Functions and Structs (RTIC-Based)

### RTIC App Structure
```rust
#[app(device = stm32f3_discovery::stm32f3xx_hal::pac, dispatchers = [EXTI0])]
mod app {
    #[shared]
    struct Shared { /* Shared resources */ }
    
    #[local]
    struct Local { /* Per-task local resources */ }
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) { /* Initialize hardware */ }
    
    #[idle]
    fn idle(cx: idle::Context) -> ! { /* Low-priority idle task */ }
    
    #[task(binds = TIM2, shared = [...], local = [...])]
    fn read_sensor(cx: read_sensor::Context) { /* High-priority interrupt task */ }
    
    #[task(priority = 1, shared = [...], local = [...])]
    async fn process_data(cx: process_data::Context) { /* Low-priority async task */ }
}
```

### `#[init]` - Hardware Initialization Task
```rust
#[init]
fn init(mut ctx: init::Context) -> (Shared, Local) {
    // Configures:
    // - RCC (clock system)
    // - GPIO (SPI pins: PA5/PA6/PA7, CS: PE3)
    // - SPI1 (1 MHz, Mode 3 - IdleHigh, CaptureOnSecondTransition)
    // - TIM2 (400 Hz periodic interrupts)
    // - DWT (cycle counter for performance measurement)
    // Returns: Shared and Local resource structures
}
```

### `#[task(binds = TIM2)]` - High-Priority Interrupt Task
```rust
#[task(binds = TIM2, 
       shared = [sensor_data, new_data_ready, read_sensor_total_cycles, read_sensor_count],
       local = [gyro])]
fn read_sensor(mut cx: read_sensor::Context) {
    // Executes every 2.5 ms when TIM2 interrupt fires
    // ├─ Measures execution time with DWT cycle counter
    // ├─ Reads gyroscope via SPI (~16.2 μs average)
    // ├─ Updates shared sensor_data with (x, y, z)
    // ├─ Sets new_data_ready flag
    // ├─ Spawns process_data task
    // └─ Accumulates execution statistics
}
```

### `#[task(priority = 1)]` - Low-Priority Async Task
```rust
#[task(priority = 1,
       shared = [sensor_data, new_data_ready, ...metrics...],
       local = [itm, prev_data, counter, anomaly_counter])]
async fn process_data(mut cx: process_data::Context) {
    // Spawned by read_sensor after each interrupt
    // ├─ Measures execution time with DWT cycle counter
    // ├─ Waits for new_data_ready flag
    // ├─ Reads sensor_data from shared buffer
    // ├─ Checks for anomalies (delta > 100.0 °/s threshold)
    // ├─ Prints readings every 4th iteration (~10 ms)
    // ├─ Accumulates execution statistics
    // └─ Prints statistics every 1000 calls (~2.5 seconds)
}
```

### `#[idle]` - Lowest Priority Idle Task
```rust
#[idle(shared = [sensor_data, new_data_ready])]
fn idle(cx: idle::Context) -> ! {
    // Runs when no other tasks are pending
    // └─ Executes asm::wfi() to put CPU into sleep mode
    // Result: CPU sleeps 99.74% of the time!
}
```

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

### Device Detection
```rust
pub fn detect_gyroscope<SPI, CS, E>(spi: &mut SPI, cs: &mut CS) -> Result<GyroVariant, E>
where
    SPI: Transfer<u8, Error = E>,
    CS: OutputPin,
{
    // Reads WHO_AM_I register (0x0F)
    // Returns: I3g4250d (0xD3), L3gd20 (0xD4), L3gd20h (0xD7), or Unknown(u8)
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

## 📊 Measurement & Performance Metrics (Phase RTIC)

### Per-Task Execution Tracking

The system independently measures both task executions using the DWT cycle counter:

**read_sensor (High-Priority):**
```
- Called: 400 times per 2.5 seconds (every 2.5 ms)
- Avg Execution: ~16.2 μs per call
- Total Time: 16.2 μs × 400 = 6,480 μs per window
```

**process_data (Low-Priority):**
```
- Called: 400 times per 2.5 seconds (spawned by each read_sensor)
- Avg Execution: ~26.6 μs per call
- Total Time: 26.6 μs × 1000 calls = 26,600 μs per window (reported every 1000 calls)
```

### Total CPU Usage Calculation

```
Total Active Time = (Process Data Time) + (Read Sensor Time)
                  = 26.6 μs + (16.2 μs × 400)
                  = 26.6 μs + 6,480 μs
                  = 6,506.6 μs per 2.5s window

Total Measurement Period = 2.5 seconds = 2,500,000 μs

CPU Usage = (Total Active Time / Total Period) × 100%
          = (6,506.6 / 2,500,000) × 100%
          = 0.26% ✓

CPU Idle = 100% - 0.26% = 99.74% (sleeping with wfi())
```

***
## 📝 Expected Output

### Successful Phase 3 RTIC Initialization
```
===============================
Phase 3: RTIC-based I3G4250D Gyroscope
===============================

Step 1: Detect Gyro driver...
✓ Gyro driver initialized.
✓ Gyro ID: 0xD3 (I3G4250D)

Step 2: Init Gyro driver...
✓ Gyro driver initialized.

Step 3: Configuring Gyro...
✓ Gyro configured).
✓ Configuration complete:
  - Data Rate: 400 Hz (timer interrupt)
  - Range: 500 °/s

Enabling DWT cycle counter for performance measurement...
✓ RTIC system ready. Starting main loop...
===============================

X:   12.34°/s | Y:   -5.67°/s | Z:    0.89°/s
X:   12.45°/s | Y:   -5.72°/s | Z:    0.91°/s
X:   12.38°/s | Y:   -5.69°/s | Z:    0.88°/s

📊 System Stats (every 1000 process_data calls / ~2.5s):
  Process Data Avg: 26.569μs (1000 calls)
  Read Sensor Avg: 16.208μs (400 calls)
  Total Time: 6509.903μs
  Anomalies: 0
  CPU Usage: 0.26%
──────────────────────────────────────

X:   12.52°/s | Y:   -5.75°/s | Z:    0.92°/s
...
```

### Expected Behavior
- **Data Rate:** New readings every 2.5 ms (400 Hz from TIM2 interrupt)
- **Process Rate:** 400 readings processed per 2.5 seconds (one per interrupt)
- **CPU Usage:** 0.24-0.28% during normal operation ✅
- **Anomaly Detection:** Flags if sensor value changes exceed 100.0 °/s threshold
- **Statistics Window:** Printed every 1000 process_data calls (~2.5 seconds)
- **Data Quality:** 0 anomalies under normal, stationary conditions
- **Task Separation:**
  - **read_sensor:** Runs 400 times per window, avg ~16.2 μs
  - **process_data:** Runs 400 times per window, avg ~26.6 μs

***

## ✅ Validation Checklist (Phase 3 RTIC)

- [x] Code compiles without errors
- [x] Flashes to board successfully
- [x] ITM output appears
- [x] Sensor data printed continuously every ~10 ms (every 4th iteration)
- [x] **Process Data execution: 26.5-27 μs**
- [x] **Read Sensor execution: 16-17 μs**
- [x] **CPU Usage: 0.24-0.28%** (accurate measurement of both tasks)
- [x] Anomalies detected = 0 (data consistency verified)
- [x] Data quality maintained from Phase 2
- [x] Statistics printed every 1000 process_data calls (~2.5s)
- [x] **Per-task breakdown visible in output**
- [x] **RTIC task priority isolation working**

---

## 🎯 Phase 3 (RTIC) Advantages Over Phase 2

| Aspect | Phase 2 (Interrupt-based) | Phase 3 (RTIC) |
|--------|---|---|
| **Framework** | Manual interrupt handlers | RTIC framework with compile-time checks |
| **Task Scheduling** | Single loop, cooperative | Preemptive multi-task with priorities |
| **Code Safety** | Manual Mutex/critical sections | Automatic lock generation by RTIC |
| **Measurement Accuracy** | Incomplete (main loop only) | **Complete (both tasks tracked)** |
| **Priority Support** | ❌ None | ✅ Hardware-based preemption |
| **Resource Sharing** | Manual locking required | Automatic RTIC-based protection |
| **Determinism** | Fair | **Guaranteed by RTIC priority** |
| **Development Speed** | Verbose, error-prone | Cleaner, less boilerplate |
| **Real-Time Guarantees** | Loose | **Strict, verified at compile-time** |
| **Debugging Support** | Basic | **Enhanced with RTIC introspection** |

**CPU Usage Comparison:**
- **Phase 2:** 0.10% (incomplete - only main loop measured)
- **Phase 3:** 0.26% (accurate - both tasks included) ✅ **2.6x increase but correct!**

---

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

## 📈 Project Phases

1. **Phase 1: Busy-Wait Polling** - Initial implementation with continuous sensor polling (100% CPU)
2. **Phase 2: Interrupt-Driven** - Timer interrupt with manual ISR handling (0.10% CPU, but incomplete measurement)
3. **Phase 3: RTIC Framework** (Current) - Multi-task real-time system with accurate per-task metrics (0.26% CPU, complete measurement)

**Last Updated:** September 2026  
**Current Phase:** 3 (RTIC-based multi-task system) ✅


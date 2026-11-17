# STM32F3 Discovery Multi-Sensor RTIC Project

A high-performance embedded Rust system demonstrating real-time concurrent sensor acquisition on the STM32F3 Discovery board. The system orchestrates an **I3G4250D 3-axis gyroscope** via SPI and an **LSM303AGR 6-axis accelerometer & magnetometer** via I2C using the **RTIC (Real-Time Interrupt-driven Concurrency)** framework, bare-metal hardware timers, custom register-level drivers, and DWT cycle-accurate profiling.

***

## 📋 Project Overview

This project showcases professional embedded Rust design patterns and real-time concurrency:

- **RTIC Framework (v2)**: Deterministic, zero-cost task scheduling and hardware interrupt binding with compile-time race-free resource sharing.
- **Dual Hardware Interrupt Architecture**:
  - **TIM2 Hardware Task (100 Hz)**: Periodically samples the I3G4250D Gyroscope over SPI1.
  - **TIM3 Hardware Task (100 Hz)**: Periodically samples the LSM303AGR Accelerometer & Magnetometer over I2C1.
- **Hybrid Bare-Metal Design**:
  - **Bare-Metal Register Level**: Custom timer drivers (`TIM2`, `TIM3`), direct RCC peripheral clock gating, custom ARM Cortex-M `PRIMASK` critical section, and datasheet-level sensor register drivers.
  - **HAL Abstraction Level**: Type-safe GPIO pin configuration (`into_af5_push_pull`, `into_af4_open_drain`), SPI1/I2C1 bus peripherals, and RCC clock tree freezing.
- **Custom Peripheral Drivers**:
  - In-house driver for **I3G4250D** (SPI Mode 3, 1 MHz, configurable data rate and full-scale range).
  - In-house driver for **LSM303AGR** (I2C, 100 kHz, multi-register burst reads, two's-complement unit conversions to $g$ and Gauss).
- **Compile-Time Feature Configuration**: Structured compile-time configuration (`AppConfig`) with Cargo feature flags (`production`, `development`, `debug-logs`, `enable-stats`).
- **Cycle-Accurate Real-Time Profiling**: Sub-microsecond execution tracking using the ARM Cortex-M4 Data Watchpoint and Trace (DWT) cycle counter.
- **Low-Power Idle Loop**: CPU enters low-power sleep via `wfi()` (Wait For Interrupt) during idle periods, keeping active duty cycle extremely low.
- **Multi-Channel ITM Instrumentation**: Real-time non-intrusive logging through the ARM Instrumentation Trace Macrocell (ITM) on the SWO pin with task-aware ANSI colors and microsecond timestamps.

***

## 🎯 Supported Sensors & Identification

The firmware identifies sensors at startup via their respective `WHO_AM_I` registers:

| Sensor | Subsystem | Bus | Bus Address / CS | WHO_AM_I Value | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **I3G4250D** | 3-Axis Gyroscope | SPI1 | `PE3` (Active Low) | `0xD3` | ✅ Fully Implemented (Custom Driver) |
| **LSM303AGR (Accel)** | 3-Axis Accelerometer | I2C1 | `0x19` | `0x33` | ✅ Fully Implemented (Custom Driver) |
| **LSM303AGR (Mag)** | 3-Axis Magnetometer | I2C1 | `0x1E` | `0x40` | ✅ Fully Implemented (Custom Driver) |
| **L3GD20** | Gyroscope | SPI1 | `PE3` | `0xD4` | 🔍 Auto-Detected Variant |
| **L3GD20H** | Gyroscope | SPI1 | `PE3` | `0xD7` | 🔍 Auto-Detected Variant |

***

## 🛠️ Hardware Setup & Pin Mapping

### 1. SPI1 Interface (I3G4250D Gyroscope)
| Signal | STM32F3 Pin | Mode / Alternate Function | Description |
| :--- | :--- | :--- | :--- |
| **SCK** | `PA5` | AF5 (Push-Pull, High Speed) | SPI1 Serial Clock |
| **MISO** | `PA6` | AF5 (Push-Pull, High Speed) | Master In, Slave Out |
| **MOSI** | `PA7` | AF5 (Push-Pull, High Speed) | Master Out, Slave In |
| **CS** | `PE3` | GPIO Output (Push-Pull) | Chip Select (Active Low) |

- **SPI Mode**: Mode 3 (Clock Polarity: `IdleHigh`, Clock Phase: `CaptureOnSecondTransition`)
- **Baud Rate**: 1.0 MHz

### 2. I2C1 Interface (LSM303AGR Accelerometer & Magnetometer)
| Signal | STM32F3 Pin | Mode / Alternate Function | Description |
| :--- | :--- | :--- | :--- |
| **SCL** | `PB6` | AF4 (Open-Drain, Pull-Up) | I2C1 Serial Clock |
| **SDA** | `PB7` | AF4 (Open-Drain, Pull-Up) | I2C1 Serial Data |

- **I2C Speed**: 100 kHz Standard Mode

### 3. Hardware Timers Configuration (Register-Level)

Both timers are clocked from the APB1 timer clock domain ($72\text{ MHz}$):

| Parameter | TIM2 (Gyroscope Task) | TIM3 (Accel/Mag Task) | Purpose |
| :--- | :--- | :--- | :--- |
| **Input Clock** | 72 MHz | 72 MHz | System Clock |
| **Prescaler (PSC)** | 719 | 719 | Divides clock to 100 kHz ($10\ \mu\text{s}$ tick) |
| **Auto-Reload (ARR)** | 999 | 999 | 100 Hz interrupt period ($10\text{ ms}$) |
| **IRQ Binding** | `TIM2` (IRQ 28) | `TIM3` (IRQ 29) | Bound directly to RTIC hardware tasks |

$$\text{Timer Frequency} = \frac{72\text{ MHz}}{719 + 1} = 100\text{ kHz}$$
$$\text{Interrupt Period} = \frac{999 + 1}{100\text{ kHz}} = 10\text{ ms} \implies 100\text{ Hz}$$

***

## 📡 RTIC System Architecture

```
                               ┌────────────────────────────────┐
                               │   72 MHz Cortex-M4 Core Clock   │
                               └───────────────┬────────────────┘
                                               │
                       ┌───────────────────────┴───────────────────────┐
                       ▼                                               ▼
         ┌───────────────────────────┐                   ┌───────────────────────────┐
         │     TIM2 Hardware Timer   │                   │     TIM3 Hardware Timer   │
         │   (100 Hz / Every 10 ms)  │                   │   (100 Hz / Every 10 ms)  │
         └─────────────┬─────────────┘                   └─────────────┬─────────────┘
                       │ IRQ TIM2                                      │ IRQ TIM3
                       ▼                                               ▼
         ┌───────────────────────────┐                   ┌───────────────────────────┐
         │ #[task(binds = TIM2)]     │                   │ #[task(binds = TIM3)]     │
         │ fn gyroscope_task(...)    │                   │ fn accel_mag_task(...)    │
         ├───────────────────────────┤                   ├───────────────────────────┤
         │ • Clear TIM2 UIF flag     │                   │ • Clear TIM3 UIF flag     │
         │ • SPI Read: I3G4250D      │                   │ • I2C Read: LSM303AGR     │
         │ • Check Anomaly (> 100°/s)│                   │ • Convert to g and Gauss  │
         │ • Update sensor_data      │                   │ • Update sensor_data_accel│
         │ • Cycle count & statistics│                   │   & sensor_data_mag       │
         └─────────────┬─────────────┘                   └─────────────┬─────────────┘
                       │                                               │
                       └───────────────────────┬───────────────────────┘
                                               │ Updates Protected Resources
                                               ▼
                                 ┌───────────────────────────┐
                                 │      RTIC Mutex Shared    │
                                 │      • sensor_data (Gyro) │
                                 │      • sensor_data_accel  │
                                 │      • sensor_data_mag    │
                                 │      • itm (Trace Macro)  │
                                 └─────────────┬─────────────┘
                                               │
                                               ▼
                                 ┌───────────────────────────┐
                                 │       #[idle] Task        │
                                 │   • Lowest Priority (0)   │
                                 │   • asm::wfi() (Sleep)    │
                                 │   • CPU sleeps ~99% time  │
                                 └───────────────────────────┘
```

***

## 📚 Project Structure

```
stm32_spi_gryoscope/
├── .cargo/
│   └── config.toml                         # Target definition (thumbv7em-none-eabihf) & runner
├── .github/
│   └── workflows/
│       └── build.yml                       # CI pipeline for embedded target builds
├── Cargo.toml                              # Workspace & package manifest, features & profiles
├── Cargo.lock                              # Dependency lockfile
├── openocd.cfg                             # OpenOCD board & interface configuration
├── openocd.gdb                             # GDB script configuring ITM / SWO logging
│
├── src/                                    # Application Crate (RTIC Application)
│   ├── main.rs                             # RTIC app: init, idle, gyroscope_task, accel_mag_task
│   ├── config.rs                           # AppConfig compile-time settings & feature assertions
│   ├── critical_section_impl.rs            # Custom Cortex-M PRIMASK critical section implementation
│   └── logging.rs                          # Task-aware, multi-port ANSI color ITM logging
│
├── auxiliary/                              # Hardware Abstraction & Driver Crate
│   ├── Cargo.toml                          # Auxiliary crate manifest
│   ├── src/
│   │   ├── lib.rs                          # Peripheral initializers, variant detection
│   │   ├── gyroscope_driver.rs             # Custom I3G4250D SPI register driver
│   │   ├── timer/
│   │   │   ├── mod.rs                      # Timer module exports & TimerConfig
│   │   │   ├── tim2.rs                     # High-level Tim2 helpers
│   │   │   └── general_purpose_timers/     # Bare-metal PAC timer implementations
│   │   │       ├── mod.rs
│   │   │       ├── timer_trait.rs          # TimerGuard trait definition
│   │   │       ├── tim2.rs                 # Tim2Guard (PAC register writes: PSC, ARR, CR1, DIER, SR)
│   │   │       └── tim3.rs                 # Tim3Guard (PAC register writes: PSC, ARR, CR1, DIER, SR)
│   │   └── accelerometer_magnetometer_driver/
│   │       ├── mod.rs
│   │       ├── registers/                  # LSM303AGR Register maps
│   │       │   ├── mod.rs
│   │       │   ├── accelerometer_registers.rs
│   │       │   └── magnetometer_registers.rs
│   │       └── driver/                     # Custom LSM303AGR I2C driver
│   │           ├── mod.rs
│   │           ├── accl_mag_driver.rs      # High-level sensor coordination & burst reads
│   │           ├── accel_config.rs         # Accel data rates, ranges, operating modes
│   │           ├── mag_config.rs           # Mag data rates, power modes, continuous mode
│   │           └── accel_data.rs           # Accel raw and scaled measurement containers
│   └── tests/
│       └── integration_tests.rs            # Hardware initialization integration tests
│
└── docs/                                   # Architecture & Technical Documentation
    ├── Rust_Embedded_Crate_Hierarchy.md    # Guide to PAC, HAL, BSP, and Core crates
    ├── SPI_Full_Duplex_Explained.md        # Full-duplex SPI protocol explanation
    ├── Data_Rate_Bit_Shifting_Explained.md # Bit shifting and register manipulation guide
    ├── datasheet/                          # Sensor datasheets for I3G4250D and LSM303AGR and more
    └── Screenshots/                        # Logic analyzer captures and terminal logs
    ***
```

***

## ⚙️ Compile-Time Configuration (`AppConfig`)

The project uses Cargo features to configure functionality at compile time without runtime penalties. Controlled in [src/config.rs](src/config.rs):

### Available Features
- `production` (default): Optimizes binary, enables statistics tracking, disables verbose debug logging.
- `development`: Enables both verbose `debug-logs` and `enable-stats`.
- `debug-logs`: Compiles verbose debug trace calls into the firmware.
- `enable-stats`: Compiles DWT performance calculations and periodic statistics reporting.

### Safety Assertion
A compile-time assertion prevents accidental debug overhead in production:
```rust
#[cfg(all(feature = "production", feature = "debug-logs"))]
compile_error!("Cannot enable debug logs in production build. Use --feature development instead.");
```

### Configuration Constants
| Constant | Default Value | Description |
| :--- | :--- | :--- |
| `MAX_ANGULAR_VELOCITY_CHANGE` | `100.0 °/s` | Threshold for detecting abrupt gyro anomalies |
| `LOG_FREQUENCY` | `4` iterations | Frequency of sensor readouts |
| `STATS_INTERVAL` | `1000` iterations | Interval for reporting performance summaries (~10 s) |

***

## 🚀 Building & Flashing

### Prerequisites
```bash
# 1. Install Rust toolchain and Cortex-M4 target
rustup target add thumbv7em-none-eabihf

# 2. Install cargo-embed or OpenOCD + GDB
cargo install cargo-embed
```

### Build Commands
```bash
# Build for release (production mode)
cargo build --release

# Build for development (with debug logs enabled)
cargo build --features development

# Verify without building binary
cargo check
```

### Flashing to Hardware
```bash
# Option A: Using OpenOCD and GDB with ITM trace logging
openocd -f openocd.cfg

# In a separate terminal:
arm-none-eabi-gdb -x openocd.gdb target/thumbv7em-none-eabihf/release/stm32_spi_gryoscope

# Option B: Using cargo-embed
cargo embed --release
```

***

## 📝 Multi-Channel ITM Logging

Logging is performed over the ARM Cortex-M **Instrumentation Trace Macrocell (ITM)**, outputting to stimulus ports over the SWO pin without halting CPU execution.

### Task & Sensor Color Scheme
| Task / Subsystem | Color | ANSI Code | Purpose |
| :--- | :--- | :--- | :--- |
| **Init** | Cyan | `\x1b[36m` | System setup and sensor verification |
| **Gyroscope** | Bright Green | `\x1b[32;1m` | Gyroscope data acquisition & anomalies |
| **Accelerometer** | Bright Purple | `\x1b[35;1m` | Accelerometer g-force measurements |
| **Magnetometer** | Bright Teal | `\x1b[36;1m` | Magnetometer Gauss field readings |
| **Idle** | Honey Yellow | `\x1b[33;1m` | Low-power idle state notifications |
| **Warning** | Yellow | `\x1b[33m` | Sensor anomalies or communication retries |
| **Stats** | Magenta | `\x1b[35m` | Cycle times, average latencies, CPU usage |

### Logging Macros
```rust
log_init!(itm, "System initialized");
log_info!(cx, "Gyroscope", "✓ Sensor data: X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s", x, y, z);
log_info!(cx, "Accelerometer", "✓ Read accelerometer data: X: {:7.2}g | Y: {:7.2}g | Z: {:7.2}g", ax, ay, az);
log_info!(cx, "Magnetometer", "✓ Read magnetometer data: X: {:7.2}G | Y: {:7.2}G | Z: {:7.2}G", mx, my, mz);
log_warn!(cx, "Gyroscope", "⚠️ ANOMALY #1: Δx=120.45, Δy=2.10, Δz=0.50");
log_stats!(cx, "Gyroscope", "CPU Usage: {:.2}%", cpu_usage);
```

***

## 📊 Expected Output

### Startup & Hardware Detection
```text
==============================================================
Phase 3: RTIC-based I3G4250D Gyroscope & LSM303AGR IMU
==============================================================
--------------------------------------------------------------
📋 Configuration Loaded:
 Debug Logs: false
 Stats Enabled: true
 Max Angular Velocity Change: 100°/s
 Log Frequency: Every 4 iterations
 Stats Interval: Every 1000 iterations
--------------------------------------------------------------
Step 1.1: Detect Gyro driver...
✓ Gyro driver initialized.
✓ Gyro ID: 0xD3 (I3G4250D)

Step 1.2: Detect Magnetometer driver...
✓ Accelerometer ID: 0x33 (LSM303AGR)
✓ Magnetometer ID: 0x40 (LSM303AGR)

Step 2.1: Init Gyro driver...
✓ Gyro driver initialized.
 Configuring Gyro...
✓ Gyro configured.
✓ Configuration Gyroscope complete:
  - Data Rate: 100 Hz (timer interrupt)
  - Range: 500 °/s

Step 2.2: Init Accelerometer and Magnetometer driver...
✓ Accelerometer and Magnetometer driver initialized.
✓ Configuration Accelerometer and Magnetometer complete:
  - Accelerometer: ±2g, High-Resolution, 100 Hz
  - Magnetometer: Continuous, 100 Hz, High-Resolution

Enabling DWT cycle counter for performance measurement...

✓ RTIC system ready. Starting main loop...
==============================================================
```

### Periodic Real-Time Acquisition & Statistics
```text
✓ Read sensor data: X:    0.15°/s | Y:   -0.22°/s | Z:    0.05°/s
✓ Read accelerometer data: X:   -0.02g | Y:    0.01g | Z:    0.99g
✓ Read magnetometer data:  X:    0.18G | Y:   -0.34G | Z:    0.42G

────────────────────────────────────────────────────────
📊 Read_sensor Gyroscope Stats (every 1000 calls / ~10.0s):
  Total Cycles: 1167400
  Abnormal Count: 0
  Read Sensor Avg: 16.214μs (1000 calls)
  CPU Usage: 0.16%
────────────────────────────────────────────────────────
```
***

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 👤 Author

Tan Dao

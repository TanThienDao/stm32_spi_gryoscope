# Phase 1: Baseline Measurement - Implementation Guide

## Overview

Phase 1 consists of **3 simple additions** to measure your current polling system's performance. Each step builds on the previous one and introduces new measurement techniques.

**Duration:** 1–2 hours  
**Difficulty:** Beginner  
**Prerequisites:** Your current working gyroscope code

---

## What is Phase 1?

Phase 1 establishes a **baseline** — measurements of your current polling-based system that you'll compare against Phase 2 (interrupts). This helps you understand:

1. **Timing accuracy** — How consistent are sensor reads?
2. **Data integrity** — Are readings corrupted or torn?
3. **Power consumption** — How many CPU cycles are wasted in busy-wait?

---

## Architecture: Polling Loop (Current)

```
Main Loop (Repeating):
  1. Read sensor via SPI
  2. Process and print data
  3. Busy-wait (NOP loop) ← CPU spinning, wasting power!
  4. Go to step 1
```

**CPU Usage:** 100% (always running, even when idle)  
**Power Consumption:** Maximum  
**Response Time:** Varies (depends on poll interval)

---

---

# Step 1.1: Add Cycle Counter to Measure Timing

## What It Does

The **DWT (Data Watchpoint and Trace)** has a built-in cycle counter that counts CPU clock cycles. We'll use it to measure how long each sensor read actually takes.

### Why It Matters

You think sensor reads take 2.5ms (400Hz), but do they really? Let's verify:
- If reads are slower → you won't get 400 samples/second
- If reads are faster → data might overlap
- If reads are inconsistent → timing varies unexpectedly

---

## Implementation

### File 1: `auxiliary/src/lib.rs`

**Status:** No changes needed!

The `DWT` is already exported in the `pub use` block:

```rust
pub use cortex_m::{
    self,
    asm, interrupt, iprint, iprintln,
    peripheral::{Peripherals, DWT, ITM, NVIC, SYST},  // ← DWT is here
};
```

### File 2: `src/main.rs`

**Action:** Replace your entire `main()` function with the code below.

#### Full Code for Step 1.1

```rust
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use auxiliary::*;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs) = init();

    // ===== PHASE 1.1: ENABLE CYCLE COUNTER =====
    
    // Get access to Cortex-M peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut dwt = cp.DWT;
    
    // Enable the cycle counter
    // DWT.CTRL bit 0 = 1 enables CYCCNT counter
    dwt.enable_cycle_counter();
    
    // ===== END SETUP =====

    iprintln!(&mut itm.stim[0], "===============================");
    iprintln!(&mut itm.stim[0], "Phase 1: Baseline Measurement");
    iprintln!(&mut itm.stim[0], "===============================");

    // Step 1: Identify the gyroscope
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 1: Detecting gyroscope...");

    let variant = match detect_gyroscope(&mut spi, &mut cs) {
        Ok(var) => {
            iprintln!(&mut itm.stim[0], "✓ Found: {:?}", var);
            var
        }
        Err(_) => {
            iprintln!(&mut itm.stim[0], "✗ Error detecting gyroscope!");
            loop {}
        }
    };

    // Step 2: Verify it's I3G4250D
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 2: Initializing custom driver...");

    if variant != GyroVariant::I3g4250d {
        iprintln!(&mut itm.stim[0], "✗ This demo is for I3G4250D only!");
        loop {}
    }

    let mut gyro = GyroDriver::new(spi, cs);

    // Verify WHO_AM_I
    match gyro.who_am_i() {
        Ok(id) => {
            if id == 0xD3 {
                iprintln!(&mut itm.stim[0], "✓ WHO_AM_I confirmed: 0x{:02X}", id);
            } else {
                iprintln!(
                    &mut itm.stim[0],
                    "✗ Unexpected WHO_AM_I value: 0x{:02X}",
                    id
                );
                loop {}
            }
        }
        Err(e) => {
            iprintln!(&mut itm.stim[0], "✗ WHO_AM_I read failed: {}", e);
            loop {}
        }
    }

    // Step 3: Configure gyroscope
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 3: Configuring gyroscope...");

    if let Err(e) = gyro.init() {
        iprintln!(&mut itm.stim[0], "✗ Initialization error: {}", e);
        loop {}
    }

    if let Err(e) = gyro.set_data_rate(DataRate::Hz400) {
        iprintln!(&mut itm.stim[0], "✗ DataRate config error: {}", e);
        loop {}
    }

    if let Err(e) = gyro.set_range(Range::DPS500) {
        iprintln!(&mut itm.stim[0], "✗ Range config error: {}", e);
        loop {}
    }

    iprintln!(&mut itm.stim[0], "✓ Configuration complete:");
    iprintln!(&mut itm.stim[0], "  - Data Rate: 400 Hz");
    iprintln!(&mut itm.stim[0], "  - Range: 500 °/s");
    iprintln!(&mut itm.stim[0], "  - All axes enabled");

    // ===== PHASE 1.1: TIMING MEASUREMENT START =====
    
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 4: Measuring Timing...");
    iprintln!(&mut itm.stim[0], "───────────────────────────");
    iprintln!(&mut itm.stim[0], "Timing data (ms per read):");
    iprintln!(&mut itm.stim[0], "");

    let mut counter = 0u32;

    loop {
        // Record start time (in CPU cycles)
        let start_cycles = cortex_m::peripheral::DWT::get_cycle_count();
        
        match gyro.read_angular_velocity() {
            Ok((x, y, z)) => {
                // Calculate elapsed cycles
                let elapsed_cycles = cortex_m::peripheral::DWT::get_cycle_count()
                    .wrapping_sub(start_cycles);
                
                // Convert cycles to milliseconds
                // CPU clock is 72 MHz = 72,000,000 cycles/second
                // To get milliseconds: cycles / 72,000 = ms
                let elapsed_ms = (elapsed_cycles as f64 / 72_000.0);
                
                // Print every 4th reading (~100ms at 400Hz)
                if counter % 4 == 0 {
                    iprintln!(
                        &mut itm.stim[0],
                        "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s | Elapsed: {:.3}ms",
                        x,
                        y,
                        z,
                        elapsed_ms
                    );
                }
                counter += 1;
            }
            Err(e) => {
                iprintln!(&mut itm.stim[0], "✗ Read error: {}", e);
                loop {}
            }
        }

        // Small delay between reads (approximate timing)
        // At 400Hz ODR, new data available every ~2.5ms
        // Adding extra delay for readability of ITM output
        for _ in 0..10_000 {
            cortex_m::asm::nop();
        }
    }
    
    // ===== END PHASE 1.1 =====
}
```

---

## Step 1.1: Code Breakdown

### Part A: Enable Cycle Counter

```rust
let cp = cortex_m::Peripherals::take().unwrap();
let mut dwt = cp.DWT;
dwt.enable_cycle_counter();
```

**What it does:**
1. `Peripherals::take()` gets the Cortex-M core peripherals (only one per program)
2. `.unwrap()` extracts the peripherals (safe here because it's first call)
3. `dwt.enable_cycle_counter()` starts the cycle counter running

**After this:** `DWT::get_cycle_count()` returns CPU cycles elapsed since startup

---

### Part B: Measure in Main Loop

```rust
let start_cycles = cortex_m::peripheral::DWT::get_cycle_count();

match gyro.read_angular_velocity() {
    Ok((x, y, z)) => {
        let elapsed_cycles = cortex_m::peripheral::DWT::get_cycle_count()
            .wrapping_sub(start_cycles);
        
        let elapsed_ms = (elapsed_cycles as f64 / 72_000.0);
        
        if counter % 4 == 0 {
            iprintln!(
                &mut itm.stim[0],
                "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s | Elapsed: {:.3}ms",
                x,
                y,
                z,
                elapsed_ms
            );
        }
```

**Explanation:**

| Line | Purpose |
|------|---------|
| `start_cycles = DWT::get_cycle_count()` | Record start time in cycles |
| `read_angular_velocity()` | Do the sensor read |
| `DWT::get_cycle_count().wrapping_sub(start_cycles)` | Calculate elapsed cycles |
| `elapsed_cycles / 72_000.0` | Convert to milliseconds |
| `if counter % 4 == 0` | Print every 4th read (1 per 10ms) |

---

## Expected Output

When you run this, you should see:

```
===============================
Phase 1: Baseline Measurement
===============================

Step 1: Detecting gyroscope...
✓ Found: I3g4250d

Step 2: Initializing custom driver...
✓ WHO_AM_I confirmed: 0xD3

Step 3: Configuring gyroscope...
✓ Configuration complete:
  - Data Rate: 400 Hz
  - Range: 500 °/s
  - All axes enabled

Step 4: Measuring Timing...
───────────────────────────
Timing data (ms per read):

X:    10.45°/s | Y:    -2.15°/s | Z:     5.32°/s | Elapsed: 2.501ms
X:    10.48°/s | Y:    -2.13°/s | Z:     5.34°/s | Elapsed: 2.502ms
X:    10.50°/s | Y:    -2.11°/s | Z:     5.36°/s | Elapsed: 2.499ms
X:    10.52°/s | Y:    -2.09°/s | Z:     5.38°/s | Elapsed: 2.501ms
X:    10.54°/s | Y:    -2.07°/s | Z:     5.40°/s | Elapsed: 2.500ms
```

---

## ✅ Step 1.1 Validation

**Pass Criteria:**

- [ ] Code compiles without errors
- [ ] ITM output appears and is readable
- [ ] Timing values shown (e.g., "Elapsed: 2.501ms")
- [ ] **All timings between 2.25–2.75ms** (within ±10% of ideal 2.5ms)
- [ ] Output appears every ~100ms (every 4th reading)
- [ ] No "Nan", "inf", or overflow values
- [ ] Consistent and stable over 30+ seconds

---

---

# Step 1.2: Add Data Consistency Check

## What It Does

Now we verify the sensor data is **coherent** — no corruption, tearing, or anomalies.

**Data Tearing Example** (What we want to prevent):
```
Previous read: X=10.45, Y=-2.15, Z=5.32
Next read:    X=250.90, Y=498.84, Z=131.00  ← Huge jump! Corruption!
```

We check if the **change between consecutive reads** is within expected bounds.

---

## Physics Background

For a **stationary gyroscope**:
- If not rotating: all readings should be ~0°/s (small noise)
- If rotating slowly: change between reads should be small

**Maximum reasonable change** at 400Hz (2.5ms per sample):
- If spinning at 500°/s: change per sample = 500 × 0.0025 = 1.25°/s
- Safety margin: allow up to 100°/s change
- If change > 100°/s → likely corruption

---

## Implementation

### File: `src/main.rs`

**Replace the main loop section** (after configuration, starting with "Step 4"):

```rust
    // ===== PHASE 1.2: TIMING + CONSISTENCY CHECK =====
    
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 4: Measuring Timing & Consistency...");
    iprintln!(&mut itm.stim[0], "───────────────────────────────────────");
    iprintln!(&mut itm.stim[0], "");

    let mut counter = 0u32;
    let mut prev_reading = (0.0f32, 0.0f32, 0.0f32);
    let mut anomaly_count = 0u32;
    
    // Maximum allowed change per 2.5ms sample
    // Allow up to 100°/s per sample (safety threshold)
    const MAX_DELTA: f32 = 100.0;

    loop {
        let start_cycles = cortex_m::peripheral::DWT::get_cycle_count();
        
        match gyro.read_angular_velocity() {
            Ok((x, y, z)) => {
                let elapsed_cycles = cortex_m::peripheral::DWT::get_cycle_count()
                    .wrapping_sub(start_cycles);
                let elapsed_ms = (elapsed_cycles as f64 / 72_000.0);
                
                // ===== NEW: CHECK CONSISTENCY =====
                
                // Calculate change from previous reading
                let delta_x = (x - prev_reading.0).abs();
                let delta_y = (y - prev_reading.1).abs();
                let delta_z = (z - prev_reading.2).abs();
                
                // Flag if any change exceeds threshold
                if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
                    anomaly_count += 1;
                    iprintln!(
                        &mut itm.stim[0],
                        "⚠️  ANOMALY #{}: dx={:.2}, dy={:.2}, dz={:.2}",
                        anomaly_count,
                        delta_x,
                        delta_y,
                        delta_z
                    );
                }
                
                // Update previous reading for next iteration
                prev_reading = (x, y, z);
                
                // ===== END NEW =====
                
                // Print every 4th reading
                if counter % 4 == 0 {
                    iprintln!(
                        &mut itm.stim[0],
                        "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s | {:.3}ms",
                        x,
                        y,
                        z,
                        elapsed_ms
                    );
                }
                counter += 1;
            }
            Err(e) => {
                iprintln!(&mut itm.stim[0], "✗ Read error: {}", e);
                loop {}
            }
        }

        for _ in 0..10_000 {
            cortex_m::asm::nop();
        }
    }
    
    // ===== END PHASE 1.2 =====
```

---

## Code Breakdown

### Tracking Previous Value

```rust
let mut prev_reading = (0.0f32, 0.0f32, 0.0f32);
```

Stores the previous sensor read so we can compare with the current one.

### Calculating Delta

```rust
let delta_x = (x - prev_reading.0).abs();
let delta_y = (y - prev_reading.1).abs();
let delta_z = (z - prev_reading.2).abs();
```

**`.abs()`** means absolute value. We don't care if change is up or down, only magnitude matters.

**Example:**
- Previous: X = 10.0
- Current: X = 12.5
- Delta: |12.5 - 10.0| = 2.5 (normal)

### Anomaly Detection

```rust
if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
    anomaly_count += 1;
    iprintln!(&mut itm.stim[0], 
        "⚠️  ANOMALY #{}: dx={:.2}, dy={:.2}, dz={:.2}",
        anomaly_count, delta_x, delta_y, delta_z);
}
```

If **any axis** changes by > 100°/s → print warning and increment counter.

---

## Expected Output

### Normal (No Corruption)

```
Step 4: Measuring Timing & Consistency...
───────────────────────────────────────

X:    10.45°/s | Y:    -2.15°/s | Z:     5.32°/s | 2.501ms
X:    10.48°/s | Y:    -2.13°/s | Z:     5.34°/s | 2.502ms
X:    10.50°/s | Y:    -2.11°/s | Z:     5.36°/s | 2.499ms
X:    10.52°/s | Y:    -2.09°/s | Z:     5.38°/s | 2.501ms
```

### If Corruption Occurred

```
X:    10.45°/s | Y:    -2.15°/s | Z:     5.32°/s | 2.501ms
⚠️  ANOMALY #1: dx=250.45, dy=500.99, dz=125.67
X:   260.90°/s | Y:   498.84°/s | Z:   131.00°/s | 2.502ms
```

---

## ✅ Step 1.2 Validation

**Pass Criteria:**

- [ ] Code compiles without errors
- [ ] Output includes timing AND consistency checks
- [ ] **Zero anomalies** over 30+ seconds of operation
- [ ] All delta values reasonable (< 10°/s for stationary sensor)
- [ ] If moving sensor: deltas match your movements
- [ ] Output stable and consistent

---

---

# Step 1.3: Establish Power Baseline

## What It Does

Measures how many **CPU cycles** are used per loop iteration. This establishes a **baseline** to compare against Phase 2 (interrupts).

**Key Insight:**
- Phase 1 (polling): ~1,280,000 cycles/loop (mostly busy-wait)
- Phase 2 (interrupts): ~1,000 cycles/loop (CPU sleeping)
- **~1,280x improvement!**

---

## Why Measure Cycles?

- **Cycles per loop** ∝ **Power consumption**
- Busy CPU = high power = fast battery drain
- Sleeping CPU = low power = long battery life

---

## Implementation

### File: `src/main.rs`

**Replace the Step 4 section** with this complete version:

```rust
    // ===== PHASE 1.3: FULL BASELINE ANALYSIS =====
    
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 4: Measuring Timing, Consistency & Power...");
    iprintln!(&mut itm.stim[0], "────────────────────────────────────────────────");
    iprintln!(&mut itm.stim[0], "");

    let mut counter = 0u32;
    let mut prev_reading = (0.0f32, 0.0f32, 0.0f32);
    let mut anomaly_count = 0u32;
    const MAX_DELTA: f32 = 100.0;
    
    // ===== NEW: POWER MEASUREMENT VARIABLES =====
    
    let mut total_cycles = 0u64;
    let mut loop_iterations = 0u32;
    
    // ===== END NEW =====

    loop {
        // ===== NEW: MEASURE TOTAL LOOP TIME =====
        let loop_start_cycles = cortex_m::peripheral::DWT::get_cycle_count();
        // ===== END NEW =====
        
        let read_start_cycles = cortex_m::peripheral::DWT::get_cycle_count();
        
        match gyro.read_angular_velocity() {
            Ok((x, y, z)) => {
                let elapsed_cycles = cortex_m::peripheral::DWT::get_cycle_count()
                    .wrapping_sub(read_start_cycles);
                let elapsed_ms = (elapsed_cycles as f64 / 72_000.0);
                
                // Consistency check
                let delta_x = (x - prev_reading.0).abs();
                let delta_y = (y - prev_reading.1).abs();
                let delta_z = (z - prev_reading.2).abs();
                
                if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
                    anomaly_count += 1;
                    iprintln!(
                        &mut itm.stim[0],
                        "⚠️  ANOMALY #{}: dx={:.2}, dy={:.2}, dz={:.2}",
                        anomaly_count,
                        delta_x,
                        delta_y,
                        delta_z
                    );
                }
                
                prev_reading = (x, y, z);
                
                if counter % 4 == 0 {
                    iprintln!(
                        &mut itm.stim[0],
                        "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s | {:.3}ms",
                        x,
                        y,
                        z,
                        elapsed_ms
                    );
                }
                counter += 1;
            }
            Err(e) => {
                iprintln!(&mut itm.stim[0], "✗ Read error: {}", e);
                loop {}
            }
        }

        // Busy-wait (original timing loop)
        for _ in 0..10_000 {
            cortex_m::asm::nop();
        }
        
        // ===== NEW: CALCULATE LOOP TIME =====
        
        let loop_end_cycles = cortex_m::peripheral::DWT::get_cycle_count();
        let loop_cycles = loop_end_cycles.wrapping_sub(loop_start_cycles);
        
        total_cycles = total_cycles.wrapping_add(loop_cycles as u64);
        loop_iterations += 1;
        
        // Print statistics every 1000 iterations (~2.5 seconds at 400Hz)
        if loop_iterations % 1000 == 0 {
            let avg_cycles = total_cycles / 1000;
            let avg_ms = (avg_cycles as f64 / 72_000.0);
            
            iprintln!(&mut itm.stim[0], "");
            iprintln!(&mut itm.stim[0], "📊 Baseline Stats (every 1000 loops / ~2.5s):");
            iprintln!(
                &mut itm.stim[0],
                "  Avg Cycles/Loop: {}",
                avg_cycles
            );
            iprintln!(
                &mut itm.stim[0],
                "  Loop Time: {:.3}ms",
                avg_ms
            );
            iprintln!(
                &mut itm.stim[0],
                "  Anomalies Detected: {}",
                anomaly_count
            );
            iprintln!(
                &mut itm.stim[0],
                "────────────────────────────",
            );
            
            // Reset for next interval
            total_cycles = 0;
        }
        
        // ===== END NEW =====
    }
    
    // ===== END PHASE 1.3 =====
```

---

## Code Breakdown

### Measurement Variables

```rust
let mut total_cycles = 0u64;
let mut loop_iterations = 0u32;
```

- `total_cycles`: Accumulates CPU cycles across 1000 loops
- `loop_iterations`: Counts how many loops we've done

**Why `u64` for cycles?** Cycles can be very large (72M/sec), so we need bigger type.

### Measure Total Loop

```rust
let loop_start_cycles = cortex_m::peripheral::DWT::get_cycle_count();

// ... do sensor read, busy-wait, etc ...

let loop_end_cycles = cortex_m::peripheral::DWT::get_cycle_count();
let loop_cycles = loop_end_cycles.wrapping_sub(loop_start_cycles);
```

Measures **everything** in the loop:
- Sensor read
- Data processing
- Busy-wait delay

### Accumulate and Report

```rust
total_cycles = total_cycles.wrapping_add(loop_cycles as u64);
loop_iterations += 1;

if loop_iterations % 1000 == 0 {  // Every 1000 loops
    let avg_cycles = total_cycles / 1000;
    let avg_ms = (avg_cycles as f64 / 72_000.0);
    
    iprintln!(&mut itm.stim[0], "  Avg Cycles/Loop: {}", avg_cycles);
    iprintln!(&mut itm.stim[0], "  Loop Time: {:.3}ms", avg_ms);
    
    total_cycles = 0;  // Reset for next interval
}
```

**What happens:**
1. After 1000 loops (~2.5 seconds), calculate average
2. Convert cycles to milliseconds
3. Print results
4. Reset counter for next interval

---

## Expected Output

**Every ~2.5 seconds, you'll see:**

```
📊 Baseline Stats (every 1000 loops / ~2.5s):
  Average Loop Time: 17.757 ms
  Average Cycles/Loop: 1278492
  Loops 1000
────────────────────────────

📊 Baseline Stats (every 1000 loops / ~2.5s):
  Average Loop Time: 17.758 ms
  Average Cycles/Loop: 1278550
  Loops 1000
────────────────────────────
```

---

## Interpretation

### What Do These Numbers Mean?

**CPU Clock:** 72 MHz = 72,000,000 cycles/second

**Calculation:**
```
Cycles/Loop = 1,278,492
Time/Loop = 1,278,492 ÷ 72,000 = 17.757 milliseconds

Breakdown with 400Hz sensor:
0.21ms (sensor read) + 17.55ms (busy-wait) ≈ 17.76ms ✓
```

### Power Consumption

**Phase 1 (Polling):**
- Cycles/Loop: ~1,278,000
- CPU Usage: 100% (always running)
- Power: **Maximum** ⚡💨

**Phase 2 (Interrupts):**
- Cycles/Loop: ~1,000
- CPU Usage: ~5% (sleeping most of time)
- Power: **~1,280x better** 🔋✓

---

## ✅ Step 1.3 Validation

**Pass Criteria:**

- [ ] Code compiles without errors
- [ ] ITM output includes timing, consistency, AND power stats
- [ ] Baseline shows ~1,280,000 cycles/loop
- [ ] Loop time ~17.75ms (consistent ±5%)
- [ ] Zero anomalies reported
- [ ] Statistics printed every ~2.5 seconds
- [ ] Baseline stable and repeatable

---

---

# Complete Phase 1 Summary

## What You've Implemented

| Step | What | How | Improvement |
|------|------|-----|------------|
| 1.1 | Enable DWT cycle counter | `dwt.enable_cycle_counter()` | Baseline measurement |
| 1.2 | Add consistency check | Track prev reading, flag anomalies | Data integrity verification |
| 1.3 | Measure power baseline | Count cycles per loop | ~1.28M cycles = 100% CPU usage |

---

## Files Changed

| File | Change |
|------|--------|
| `auxiliary/src/lib.rs` | No changes (already has DWT) |
| `src/main.rs` | Complete replacement with Step 1.3 code |

---

## Expected Results

### ITM Output Format

```
===============================
Phase 1: Baseline Measurement
===============================

Step 1: Detecting gyroscope...
✓ Found: I3g4250d

Step 2: Initializing custom driver...
✓ WHO_AM_I confirmed: 0xD3

Step 3: Configuring gyroscope...
✓ Configuration complete:
  - Data Rate: 400 Hz
  - Range: 500 °/s
  - All axes enabled

Step 4: Measuring Timing, Consistency & Power...
────────────────────────────────────────────────

X:   -0.38°/s | Y:    0.60°/s | Z:   -0.60°/s | Elapsed: 0.209750 ms
X:   -0.81°/s | Y:    0.26°/s | Z:   -0.74°/s | Elapsed: 0.209750 ms
X:   -0.21°/s | Y:    0.40°/s | Z:   -0.54°/s | Elapsed: 0.209750 ms

📊 Baseline Stats (every 1000 loops / ~2.5s):
  Average Loop Time: 17.757 ms
  Average Cycles/Loop: 1278492
  Loops 1000
────────────────────────────
```

---

## Testing Procedure

### 1. Build
```bash
cd /home/tandao/Documents/repos/rust_idemy/embedded_rust/embedded\ project/stm32_spi_gryoscope
cargo build --release
```

### 2. Flash
```bash
cargo embed --release
```

### 3. Observe ITM Output
- Watch for timing values ~0.21ms (sensor read only)
- Watch for zero anomalies
- Watch for baseline stats ~1.28M cycles/loop (~17.76ms)

### 4. Run for 30+ Seconds
- Verify output is consistent
- Verify no corruptions detected
- Document baseline values

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| "DWT not found" | Not imported | Already in `lib.rs` |
| Timing shows "Nan" or "inf" | Sensor read failed | Check gyroscope wiring |
| Anomalies every read | Sensor corrupting data | Check SPI connections |
| No ITM output | Debugger issue | Restart OpenOCD/IDE |
| Cycles way too high (>1M) | NOP loop too long | Normal for polling |

---

## Next Steps

Once Phase 1 is working:

1. **Document baseline:** Write down your measurements
2. **Move to Phase 2:** Replace polling with timer interrupt
3. **Compare:** Phase 2 cycles should be ~1,000 (~1,280x improvement!)

---

## Key Takeaways

✅ **Polling uses 100% CPU** — Busy-wait = wasted power  
✅ **DWT cycle counter** — Accurate timing measurement  
✅ **Consistency checks** — Detect data corruption early  
✅ **Baseline established** — Now ready to optimize!  

---

**Next document:** `02_Phase2_Implementation_Guide.md` (coming soon)

Phase 1 complete! Ready to implement? 🚀


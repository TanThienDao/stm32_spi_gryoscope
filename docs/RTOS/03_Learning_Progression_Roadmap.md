# Learning Progression Roadmap

## Overview

This document provides a **step-by-step implementation guide** from polling → bare-metal interrupts → RTIC. Each phase builds on the previous one, with clear milestones and validation checkpoints.

---

## Phase 1: Establish Baseline (Current System)

**Duration:** 1–2 hours  
**Goal:** Verify current polling system works and measure baseline metrics  
**Deliverables:** Timing data, power baseline, documented expectations

### 1.1 Add Cycle Counter to Measure Timing

**File to modify:** `auxiliary/src/lib.rs` and `src/main.rs`

**Changes:**
- Enable DWT (Data Watchpoint and Trace) cycle counter
- Measure time between sensor reads
- Log timing to ITM

**Expected output:**
```
Elapsed: 2.501ms
Elapsed: 2.502ms
Elapsed: 2.499ms
```

**Validation:**
- [ ] All readings within 2.25–2.75ms (±10% of 2.5ms)
- [ ] Output consistent over 30 seconds
- [ ] No anomalies or "Nan" values

### 1.2 Add Data Consistency Check

**File to modify:** `src/main.rs`

**Changes:**
- Track previous sensor values
- Flag if delta > 100°/s between consecutive reads
- Log anomalies

**Validation:**
- [ ] Zero anomalies over 30 seconds of normal use
- [ ] Realistic deltas (< 50°/s for stationary sensor)

### 1.3 Establish Power Baseline

**File to modify:** `src/main.rs`

**Changes:**
- Measure cycle count for each loop iteration
- Calculate average cycles/loop
- Log every 1000 iterations

**Expected baseline:**
- Busy-wait: ~900,000 cycles/loop (mostly NOP)
- Loop duration: ~12.5ms (900k cycles ÷ 72MHz)

**Validation:**
- [ ] Average cycles stable (±5%)
- [ ] Documented in `Phase1_Baseline.txt`

---

## Phase 2: Bare-Metal TIM2 Interrupt (Basic)

**Duration:** 3–4 hours  
**Goal:** Replace polling with timer interrupt, maintain same functionality  
**Deliverables:** Working interrupt-driven system, reduced power usage

### 2.1 Configure TIM2 for 400Hz Interrupt

**File to create:** `auxiliary/src/interrupt_handler.rs`  
**File to modify:** `auxiliary/src/lib.rs`, `src/main.rs`

**Implementation steps:**

1. **Add TIM2 timer configuration:**
   ```rust
   // PSC (Prescaler) = 720 - 1  (72 MHz ÷ 720 = 100 kHz)
   // ARR (Auto Reload) = 250 - 1  (100 kHz ÷ 250 = 400 Hz)
   ```

2. **Export interrupt macro helpers:**
   - Re-export `cortex_m::interrupt::*`
   - Re-export `NVIC` for unmask

3. **Main loop changes:**
   - Remove busy-wait loop
   - Add `cortex_m::asm::wfe()` (sleep)

**Validation:**
- [ ] Code compiles without errors
- [ ] No runtime crashes
- [ ] ITM output shows initial data reads

### 2.2 Implement Global Sensor Data (Unsafe)

**File to modify:** `auxiliary/src/interrupt_handler.rs`

**Implementation:**

```rust
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;

// Shared sensor data
pub static SENSOR_DATA: Mutex<RefCell<Option<(f32, f32, f32)>>> = 
    Mutex::new(RefCell::new(None));

// Global gyro reference (stored at init)
static mut GYRO_GLOBAL: Option<GyroDriver> = None;
static mut TIM2_GLOBAL: Option<pac::TIM2> = None;
```

**Validation:**
- [ ] Compiles with `#![allow(unsafe_code)]`
- [ ] No segfaults or panics

### 2.3 Write TIM2 Interrupt Handler

**File to modify:** `auxiliary/src/interrupt_handler.rs` (create this file)

**Implementation:**

```rust
#[interrupt]
fn TIM2() {
    unsafe {
        if let Some(ref mut tim2) = TIM2_GLOBAL {
            // Clear interrupt flag (must do first)
            tim2.sr.modify(|_, w| w.uif().clear_bit());
            
            // Read sensor
            if let Some(ref mut gyro) = GYRO_GLOBAL {
                if let Ok((x, y, z)) = gyro.read_angular_velocity() {
                    // Store safely
                    SENSOR_DATA.lock(|data| {
                        data.borrow_mut().replace((x, y, z));
                    });
                }
            }
        }
    }
}
```

**Validation:**
- [ ] Compiles
- [ ] `#[interrupt]` macro recognized
- [ ] Interrupt handler signature matches `cortex-m` trait

### 2.4 Update Main to Initialize Timer and Spawn Interrupt

**File to modify:** `src/main.rs` and `auxiliary/src/lib.rs`

**Implementation:**

```rust
pub fn init_timer_interrupt(dp: &pac::Peripherals) {
    let tim2 = &dp.TIM2;
    
    // Setup: PSC = 720-1, ARR = 250-1
    tim2.psc.write(|w| w.psc().bits(720 - 1));
    tim2.arr.write(|w| w.arr().bits(250 - 1));
    
    // Enable timer
    tim2.cr1.write(|w| w.cen().set_bit());
    
    // Enable interrupt on update event
    tim2.dier.write(|w| w.uie().set_bit());
    
    // Unmask in NVIC
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIM2);
    }
}
```

**Validation:**
- [ ] Timer configuration values correct (PSC=720, ARR=250)
- [ ] NVIC unmask succeeds
- [ ] Interrupt fires (~400 times/second)

### 2.5 Verify Interrupt Firing Rate

**File to modify:** `src/main.rs`

**Implementation:** Add counter in interrupt handler

```rust
static INTERRUPT_COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIM2() {
    // ... existing handler ...
    
    // Increment counter
    INTERRUPT_COUNT.lock(|count| {
        *count.borrow_mut() += 1;
    });
}

fn main() -> ! {
    // ... init ...
    
    let mut last_count = 0;
    let mut iterations = 0;
    
    loop {
        cortex_m::asm::wfe();
        
        iterations += 1;
        if iterations % 2000 == 0 {  // Every ~5 seconds
            INTERRUPT_COUNT.lock(|count| {
                let current = *count.borrow();
                let delta = current - last_count;
                let frequency = (delta as f32 / 5.0);
                iprintln!(&mut itm.stim[0], 
                    "Interrupt Freq: {:.1} Hz", frequency);
                last_count = current;
            });
        }
    }
}
```

**Expected output:**
```
Interrupt Freq: 400.2 Hz
Interrupt Freq: 399.9 Hz
Interrupt Freq: 400.1 Hz
```

**Validation:**
- [ ] Frequency within 399–401 Hz
- [ ] Consistent over time

### 2.6 Verify Data Updates

**File to modify:** `src/main.rs`

**Implementation:** Log when data is updated

```rust
loop {
    cortex_m::asm::wfe();
    
    SENSOR_DATA.lock(|data| {
        if let Some((x, y, z)) = *data.borrow() {
            iprintln!(&mut itm.stim[0], 
                "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s", 
                x, y, z);
        }
    });
}
```

**Validation:**
- [ ] Data updates every ~2.5ms
- [ ] Values coherent (no tearing)
- [ ] No "Nan" or overflow values

### 2.7 Measure Power Improvement

**File to modify:** `src/main.rs`

**Implementation:**

```rust
let mut cycles_in_loop = 0;
let mut loop_iterations = 0;

loop {
    let start = DWT::get_cycle_count();
    
    SENSOR_DATA.lock(|data| {
        if let Some((x, y, z)) = *data.borrow() {
            iprintln!(&mut itm.stim[0], "{:.2}", x);
        }
    });
    
    cortex_m::asm::wfe();  // Sleep
    
    let elapsed = DWT::get_cycle_count().wrapping_sub(start);
    cycles_in_loop += elapsed;
    loop_iterations += 1;
    
    if loop_iterations % 1000 == 0 {
        let avg = cycles_in_loop / 1000;
        iprintln!(&mut itm.stim[0], "Avg cycles/loop: {}", avg);
        cycles_in_loop = 0;
    }
}
```

**Expected improvement:**
- Phase 1: ~900,000 cycles/loop
- Phase 2: ~1,000 cycles/loop (900x improvement!)

**Validation:**
- [ ] Cycle count reduced by 99%
- [ ] Documented in `Phase2_Results.txt`

---

## Phase 3: RTIC Migration

**Duration:** 4–6 hours  
**Goal:** Rewrite system using RTIC framework for safety and scalability  
**Deliverables:** RTIC-based project, multiple concurrent tasks

### 3.1 Add RTIC Dependencies

**File to modify:** `Cargo.toml` and `auxiliary/Cargo.toml`

**Changes:**
```toml
[dependencies]
rtic = "0.5"  # Or latest version compatible with your cortex-m
cortex-m-rt = "0.7"  # Update if needed
```

**Validation:**
- [ ] Dependencies resolve
- [ ] No version conflicts

### 3.2 Create RTIC App Structure

**File to modify:** `src/main.rs`

**Implementation:**

```rust
#![no_std]
#![no_main]

use rtic::app;
use auxiliary::*;

#[app(device = stm32f3_discovery::stm32f3xx_hal::pac)]
const APP: () = {
    struct Resources {
        #[init = (0.0, 0.0, 0.0)]
        sensor_data: (f32, f32, f32),
        gyro: GyroDriver,
        itm: ITM,
    }
    
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let (mut itm, _delay, spi, cs) = init();
        let gyro = GyroDriver::new(spi, cs);
        
        // Initialize TIM2 for 400Hz
        // (RTIC macro will bind interrupt)
        
        init::LateResources {
            sensor_data: (0.0, 0.0, 0.0),
            gyro,
            itm,
        }
    }
    
    #[task(binds = TIM2, resources = [sensor_data, gyro], priority = 2)]
    fn sensor_read(cx: &mut sensor_read::Context) {
        if let Ok((x, y, z)) = cx.resources.gyro.read_angular_velocity() {
            *cx.resources.sensor_data = (x, y, z);
        }
    }
    
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfe();
        }
    }
};
```

**Validation:**
- [ ] Code compiles
- [ ] RTIC macro expands successfully
- [ ] No unsafe code needed (except in init)

### 3.3 Add Software Task for Logging

**File to modify:** `src/main.rs`

**Implementation:**

```rust
#[task(resources = [sensor_data, itm], priority = 1)]
fn log_data(cx: &mut log_data::Context) {
    let (x, y, z) = cx.resources.sensor_data;
    iprintln!(&mut cx.resources.itm.stim[0], 
        "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s", x, y, z);
}
```

### 3.4 Spawn Log Task from Sensor Task

**File to modify:** `src/main.rs`

**Implementation:**

```rust
#[task(binds = TIM2, resources = [sensor_data, gyro], priority = 2)]
fn sensor_read(cx: &mut sensor_read::Context) {
    if let Ok((x, y, z)) = cx.resources.gyro.read_angular_velocity() {
        *cx.resources.sensor_data = (x, y, z);
        
        // Spawn lower-priority logging task
        log_data::spawn().ok();
    }
}
```

**Validation:**
- [ ] `spawn()` function available
- [ ] Log task executes after sensor task
- [ ] Output matches Phase 2

### 3.5 Add Priority Inversion Test (Optional)

**File to modify:** `src/main.rs`

**Implementation:** Create a medium-priority task that logs less frequently

```rust
#[task(resources = [sensor_data, itm], priority = 1)]
fn medium_task(cx: &mut medium_task::Context) {
    // This can run between high and low priority tasks
}

#[task(binds = EXTI0, priority = 0)]
fn low_task(cx: &mut low_task::Context) {
    // This gets preempted by higher priorities
}
```

**Validation:**
- [ ] Task execution order correct
- [ ] No deadlocks or stalls

### 3.6 Compare Power Usage (Phase 3 vs Phase 2)

**Expected result:** Similar to Phase 2 (~1,000 cycles/loop)

**Validation:**
- [ ] Power usage remains low
- [ ] RTIC overhead minimal

---

## Phase 4 (Optional): Advanced Multi-Task System

**Duration:** 2–3 hours  
**Goal:** Implement data processing in separate task, demonstrate real-world scenario  
**Deliverables:** Multi-task RTIC system with task spawning and inter-task communication

### 4.1 Add Gyro Calibration Task

**File to modify:** `src/main.rs`

**Implementation:**

```rust
#[task(priority = 1)]
fn calibrate_gyro(cx: &mut calibrate_gyro::Context, samples: u32) {
    // Collect samples and compute offset
    // Runs at lower priority, doesn't block sensor reads
}

#[task(binds = TIM2, resources = [sensor_data, gyro], priority = 2)]
fn sensor_read(cx: &mut sensor_read::Context) {
    // ...
    if should_calibrate() {
        calibrate_gyro::spawn(100).ok();
    }
}
```

### 4.2 Add Statistics Computation

**File to modify:** `src/main.rs`

**Implementation:**

```rust
#[task(resources = [sensor_data], priority = 0)]
fn compute_stats(cx: &mut compute_stats::Context) {
    // RMS, variance, filtering
    // Lowest priority: only runs when nothing else needs CPU
}
```

---

## Validation Checklist

### Phase 1 ✅
- [ ] Timing accuracy: ±10% of 2.5ms
- [ ] Data consistency: No anomalies
- [ ] Baseline documented: ~900k cycles/loop
- [ ] Output readable and stable

### Phase 2 ✅
- [ ] Timer interrupt fires at 400±1 Hz
- [ ] Data updated every ~2.5ms
- [ ] Mutex prevents corruption (no anomalies)
- [ ] CPU sleeping: ~1k cycles/loop
- [ ] Power improved 99%

### Phase 3 ✅
- [ ] RTIC code compiles
- [ ] Interrupt handler works
- [ ] Task priorities execute correctly
- [ ] Software tasks spawn successfully
- [ ] Power usage maintained

### Phase 4 ✅ (Optional)
- [ ] Multiple tasks coexist
- [ ] No priority inversion
- [ ] Spawning and rescheduling works
- [ ] Real-world scenario functional

---

## Testing and Debugging Tips

### Build and Flash

```bash
# Phase 1 (polling)
cargo build --release
cargo embed --release

# Phase 2 (bare-metal interrupts)
cargo build --release
cargo embed --release

# Phase 3 (RTIC)
cargo build --release
cargo embed --release
```

### Monitor ITM Output

```bash
# Keep a terminal open to see real-time output
# In VSCode or your IDE, open ITM output viewer
```

### Breakpoints in Interrupt Handler

```gdb
# In GDB:
(gdb) break TIM2
(gdb) continue
# Will hit breakpoint ~400 times per second
# Use `c` (continue) to skip most, or set count
(gdb) break TIM2 if condition
```

### Measure Timing with Logic Analyzer

Connect to SPI pins (SCK, MOSI, MISO, CS):
```
Expected pattern:
[CS Low]──[SPI pulses]──[CS High]──[2.5ms idle]──[CS Low]...
```

---

## Troubleshooting Guide

| Issue | Cause | Solution |
|-------|-------|----------|
| "Interrupt not firing" | NVIC not unmasked | Call `NVIC::unmask(Interrupt::TIM2)` |
| "Timer fires at wrong frequency" | PSC/ARR calculation wrong | Recalculate: PSC=(72MHz/f_timer)-1, ARR=(f_timer/f_interrupt)-1 |
| "Data corruption detected" | Mutex not used | Wrap all shared data access in `Mutex::lock()` |
| "Hard Fault after first read" | Static not initialized | Use `Option<T>`, initialize in `#[init]` |
| "RTIC macro error: unknown attribute" | RTIC version mismatch | Update `Cargo.toml`: `rtic = "0.5"` |
| "CPU never sleeps (100% power)" | WFE not reached | Check interrupt handler clears flag with `sr.modify(...uif().clear_bit())` |

---

## Milestones and Checkpoints

```
Week 1:
├─ Phase 1 (Day 1): Baseline measurement ✓
├─ Phase 2 (Day 2–3): Bare-metal interrupts ✓
└─ Phase 2 (Day 4): Verification & power measurement ✓

Week 2:
├─ Phase 3 (Day 1–2): RTIC migration ✓
├─ Phase 3 (Day 3): Multi-task expansion ✓
└─ Phase 4 (Day 4): Advanced features ✓

Total: ~2 weeks for full learning path
```

---

## Next Steps

1. **Phase 1:** Review the baseline validation section above
2. **Phase 2:** Start timer configuration in `auxiliary/src/lib.rs`
3. **Phase 3:** Begin RTIC app structure when Phase 2 is complete
4. **Phase 4:** Add advanced tasks once RTIC foundation is solid

---

## References

- [RTIC Documentation](https://rtic.rs/)
- [Cortex-M Runtime](https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)
- [Cortex-M Interrupt Guide](https://docs.rs/cortex-m/latest/cortex_m/)
- [STM32F3 Reference Manual](https://www.st.com/resource/en/reference_manual/dm00043574.pdf)

---

**You are ready to begin Phase 1!** Proceed with adding timing measurements to your current system.


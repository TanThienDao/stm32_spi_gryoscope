# Verification and Testing Strategy

## Overview

This document explains how to **verify each phase works correctly**, from polling to interrupts to RTIC. You'll learn what to look for, how to measure, and what can go wrong.

---

## Phase 1: Baseline Verification (Current Polling System)

### What You Currently Have

Your gyroscope reads at ~400Hz in a polling loop with busy-wait delays.

### Baseline Measurements

**Before making changes, establish baseline metrics:**

#### 1.1 Timing Accuracy

**Goal**: Verify current polling interval matches intended 400Hz (2.5ms per read)

**Method**: Use ITM timestamps

```rust
// In your main loop, add timing checks:
use cortex_m::DWT;

let mut dwt = cp.DWT;
dwt.enable_cycle_counter();

loop {
    let start = DWT::get_cycle_count();
    
    match gyro.read_angular_velocity() {
        Ok((x, y, z)) => {
            let elapsed_cycles = DWT::get_cycle_count().wrapping_sub(start);
            let elapsed_ms = (elapsed_cycles as f64 / 72_000_000.0) * 1000.0;
            
            if counter % 4 == 0 {
                iprintln!(&mut itm.stim[0], 
                    "X: {:7.2} | Elapsed: {:.3}ms", x, elapsed_ms);
            }
        }
        Err(e) => { /* ... */ }
    }
}
```

**Expected output:**
```
X:    10.45 | Elapsed: 2.501ms
X:    10.48 | Elapsed: 2.502ms
X:    10.50 | Elapsed: 2.499ms
```

**Pass Criteria**: ±10% of 2.5ms (i.e., 2.25–2.75ms)

---

#### 1.2 Data Consistency

**Goal**: Verify sensor readings are coherent (no tearing/corruption)

**Method**: Check for logical consistency

```rust
// Monitor consecutive reads for consistency
let mut prev = (0.0, 0.0, 0.0);

loop {
    match gyro.read_angular_velocity() {
        Ok((x, y, z)) => {
            let delta_x = (x - prev.0).abs();
            let delta_y = (y - prev.1).abs();
            let delta_z = (z - prev.2).abs();
            
            // At 400Hz with bounded angular velocity, 
            // delta should be < 100°/s (adjust based on sensor)
            if delta_x > 100.0 || delta_y > 100.0 || delta_z > 100.0 {
                iprintln!(&mut itm.stim[0], 
                    "⚠️  Anomaly: dx={:.2}, dy={:.2}, dz={:.2}", 
                    delta_x, delta_y, delta_z);
            }
            
            prev = (x, y, z);
        }
        Err(e) => { /* ... */ }
    }
}
```

**Pass Criteria**: No anomalies detected during 10 seconds of normal operation

---

#### 1.3 CPU Power State

**Goal**: Establish baseline power consumption (for comparison after optimization)

**Method**: Monitor CPU cycles and sleep state

```rust
let mut cycles_in_loop = 0;
let mut loop_iterations = 0;

loop {
    let start = DWT::get_cycle_count();
    
    match gyro.read_angular_velocity() {
        Ok(_) => { /* ... */ }
        Err(_) => { /* ... */ }
    }
    
    for _ in 0..10_000 {
        cortex_m::asm::nop();  // Busy wait
    }
    
    let elapsed = DWT::get_cycle_count().wrapping_sub(start);
    cycles_in_loop += elapsed;
    loop_iterations += 1;
    
    // Report every 1000 iterations (~2.5 seconds)
    if loop_iterations % 1000 == 0 {
        let avg_cycles = cycles_in_loop / 1000;
        iprintln!(&mut itm.stim[0], 
            "Avg cycles/loop: {}", avg_cycles);
        cycles_in_loop = 0;
    }
}
```

**Expected output**: ~900,000 cycles per loop (mostly NOP busy-wait)

**Optimization target**: Reduce to < 1,000 cycles (CPU in sleep state)

---

## Phase 2: Interrupt-Based System Verification

### 2.1 Interrupt Firing Rate

**Goal**: Verify timer interrupt fires at 400Hz (±1%)

**Method**: Count interrupt firings

```rust
static INTERRUPT_COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIM2() {
    // Increment counter on every interrupt
    INTERRUPT_COUNT.lock(|count| {
        *count.borrow_mut() += 1;
    });
    
    // Clear interrupt flag
    clear_tim2_interrupt();
}

fn main() -> ! {
    init_timer_interrupt();  // Setup 400Hz timer
    
    let mut last_count = 0;
    
    loop {
        cortex_m::asm::wfe();  // Sleep
        
        // Every 5 seconds, check interrupt rate
        INTERRUPT_COUNT.lock(|count| {
            let current = *count.borrow();
            let delta = current - last_count;
            
            if current % 2000 == 0 {  // Every ~5 seconds
                let frequency = (delta as f32 / 5.0);  // Should be ~400 Hz
                iprintln!(&mut itm.stim[0], 
                    "Interrupt Freq: {:.1} Hz", frequency);
                last_count = current;
            }
        });
    }
}
```

**Expected output:**
```
Interrupt Freq: 400.2 Hz
Interrupt Freq: 399.9 Hz
Interrupt Freq: 400.1 Hz
```

**Pass Criteria**: 399–401 Hz (±0.25%)

---

### 2.2 Data Freshness

**Goal**: Verify data is updated at interrupt rate

**Method**: Track update timestamps

```rust
use core::sync::atomic::{AtomicU32, Ordering};

static DATA_UPDATE_COUNT: AtomicU32 = AtomicU32::new(0);

#[interrupt]
fn TIM2() {
    if let Ok((x, y, z)) = GYRO.read_sensor() {
        SENSOR_DATA.lock(|data| {
            data.borrow_mut().replace((x, y, z));
        });
        
        // Increment update counter
        DATA_UPDATE_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    
    clear_tim2_interrupt();
}

fn main() -> ! {
    let mut last_update_count = 0;
    
    loop {
        cortex_m::asm::wfe();
        
        let current_count = DATA_UPDATE_COUNT.load(Ordering::SeqCst);
        if current_count > last_update_count {
            iprintln!(&mut itm.stim[0], "Data updated!");
            last_update_count = current_count;
        }
    }
}
```

**Pass Criteria**: Update count increases by ~2000 every 5 seconds

---

### 2.3 No Data Corruption

**Goal**: Verify Mutex prevents race conditions

**Method**: Add checksums or range checks

```rust
static SENSOR_DATA: Mutex<RefCell<Option<(f32, f32, f32)>>> = 
    Mutex::new(RefCell::new(None));

fn main() -> ! {
    let mut corruption_detected = false;
    
    loop {
        cortex_m::asm::wfe();
        
        SENSOR_DATA.lock(|data| {
            if let Some((x, y, z)) = *data.borrow() {
                // Gyroscope range is ±500°/s (configured)
                // Any value outside is corrupted
                if x.abs() > 600.0 || y.abs() > 600.0 || z.abs() > 600.0 {
                    if !corruption_detected {
                        iprintln!(&mut itm.stim[0], 
                            "⚠️  Corruption detected: x={:.2}", x);
                        corruption_detected = true;
                    }
                }
            }
        });
    }
}
```

**Pass Criteria**: No corruption messages over 30 seconds

---

### 2.4 CPU Power Improvement

**Goal**: Verify CPU sleeping reduces power usage

**Method**: Compare cycle counts with Phase 1

```rust
loop {
    let start = DWT::get_cycle_count();
    
    // Retrieve data (usually < 100 cycles)
    SENSOR_DATA.lock(|data| {
        if let Some((x, y, z)) = *data.borrow() {
            iprintln!(&mut itm.stim[0], "X: {:.2}", x);
        }
    });
    
    cortex_m::asm::wfe();  // Sleep
    
    let elapsed = DWT::get_cycle_count().wrapping_sub(start);
    // Most time spent in WFE (sleeping), not busy-wait!
}
```

**Expected improvement**: 
- Phase 1 (polling): ~900,000 cycles/loop
- Phase 2 (interrupt): ~1,000 cycles/loop (900x improvement!)

---

## Phase 3: RTIC Verification

### 3.1 Task Priority Execution

**Goal**: Verify RTIC enforces task priorities correctly

**Method**: Log execution order with timestamps

```rust
#[rtic::app(device = stm32f3_discovery::stm32f3xx_hal::pac)]
const APP: () = {
    #[task(priority = 2)]
    fn sensor_read(_cx: &mut sensor_read::Context) {
        iprintln!(&mut itm, "[2] Sensor read task (high priority)");
    }
    
    #[task(priority = 1)]
    fn data_process(_cx: &mut data_process::Context) {
        iprintln!(&mut itm, "[1] Data process task (low priority)");
    }
    
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        iprintln!(&mut itm, "[0] Idle task");
        loop {}
    }
};
```

**Expected output:**
```
[2] Sensor read task (high priority)
[2] Sensor read task (high priority)
[1] Data process task (low priority)
[2] Sensor read task (high priority)
```

Higher priority tasks preempt lower ones.

---

### 3.2 Resource Safety

**Goal**: Verify RTIC compile-time checks prevent data races

**Method**: Attempt to cause race condition (should fail to compile)

```rust
#[rtic::app(device = stm32f3_discovery::stm32f3xx_hal::pac)]
const APP: () = {
    struct Resources {
        sensor_data: (f32, f32, f32),
    }
    
    #[task(resources = [sensor_data])]
    fn read_sensor(cx: &mut read_sensor::Context) {
        cx.resources.sensor_data = (10.0, 20.0, 30.0);
    }
    
    #[task(resources = [sensor_data])]
    fn log_data(cx: &mut log_data::Context) {
        // This will fail to compile if priorities don't allow shared access
        iprintln!(&mut itm, "{:?}", cx.resources.sensor_data);
    }
};
```

**Pass Criteria**: No race conditions; RTIC compiler prevents them or forces explicit locks.

---

## Verification Checklist

Use this checklist for each phase:

### Phase 1 (Polling)
- [ ] Timing accuracy: ±10% of 2.5ms
- [ ] Data consistency: No anomalies over 10s
- [ ] Baseline power: ~900k cycles/loop
- [ ] ITM output visible and readable

### Phase 2 (Interrupts)
- [ ] Timer fires at 400±1 Hz
- [ ] Data updates every ~2.5ms
- [ ] No corruption detected over 30s
- [ ] CPU cycles reduced to < 1k/loop
- [ ] Mutex access is crash-free

### Phase 3 (RTIC)
- [ ] Task priority execution correct
- [ ] Resource safety enforced by compiler
- [ ] Multiple tasks run concurrently
- [ ] Spawn/reschedule works reliably
- [ ] Power consumption lower than Phase 2

---

## Testing Tools

### 1. ITM Output Analysis

**Use grep to check timing patterns:**

```bash
# Count interrupt firings per second
grep "Interrupt Freq" output.log | wc -l

# Find anomalies
grep "Anomaly\|Corruption" output.log

# Analyze timing data
grep "Elapsed:" output.log | awk '{print $NF}'
```

### 2. OpenOCD Breakpoints

Break on interrupt handler to verify it's being called:

```gdb
# In GDB during debugging
break TIM2
continue
# Should hit breakpoint ~400 times per second
```

### 3. Oscilloscope/Logic Analyzer

Measure SPI clock and CS pin toggling to verify sensor reads:

```
Expected pattern at 400Hz:
[CS Low] ──[SPI Clock Pulses]── [CS High] ──[2.5ms gap]── [CS Low] ...
```

---

## Common Issues and Diagnostics

| Issue | Symptom | Diagnosis | Fix |
|-------|---------|-----------|-----|
| Interrupt not firing | ITM silent | Check NVIC unmask, TIM2 enabled | Enable in RCC, NVIC |
| Timer frequency wrong | 200Hz instead of 400Hz | PSC/ARR calculation error | Recalculate: PSC * ARR = CPU_CLK / FREQ |
| Data tearing | Alternating good/corrupted values | Mutex not used | Wrap all data access in Mutex::lock |
| CPU not sleeping | High power, ~900k cycles | WFE not reached | Check interrupt handler clears flag |
| Crashes on second read | "Hard Fault" after first sensor read | Static global not initialized | Use Option<T>, check initialization |

---

## Next Steps

→ Read **`02_RTIC_Concepts_and_Architecture.md`** to understand RTIC framework design.

→ Then follow **`03_Learning_Progression_Roadmap.md`** for step-by-step implementation.


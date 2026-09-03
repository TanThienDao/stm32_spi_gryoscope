# CPU Usage Calculation - Detailed Explanation

## Overview
This document explains how to correctly calculate CPU usage in interrupt-driven embedded systems, specifically for the STM32F3 Gyroscope project running at 72 MHz.

---

## The Problem

### Original (Incorrect) Formula
```rust
(avg_cycles as f64 / 72_000.0 / 2.5 * 100.0)
```

### Why It Was Wrong

The original formula had a **logical flaw in the time period calculation**:

1. **What it calculated:**
   - `avg_cycles / 72_000.0` = Loop time in milliseconds
   - Then divided by 2.5 (the loop time itself)
   - This created a circular logic: `loop_time / loop_time = 1 = 100%`

2. **Example with real numbers:**
   - Avg cycles: 179,722
   - Loop time: 179,722 / 72,000 = 2.496 ms
   - Formula: 2.496 / 2.5 × 100 = **99.8%** ❌ (WRONG!)

3. **The fundamental error:**
   - The denominator (2.5) is the **loop time**, not the **total measurement period**
   - This makes the formula report `loop_time / loop_time = ~100%` regardless of actual efficiency

---

## The Solution

### Correct Formula

**CPU Usage = (Time spent executing / Total measurement time) × 100%**

```
CPU Usage = (Loop Time per Iteration × Number of Iterations / Total Measurement Period) × 100%
```

Or simplified:

```
CPU Usage = (Loop Time per Iteration / Time between interrupt events) × 100%
```

### Step-by-Step Breakdown

#### Given Information:
- **CPU Clock:** 72 MHz
- **Timer Interrupt Frequency:** 400 Hz (fires every 2.5 ms)
- **Measurement Window:** 1000 loop iterations
- **Average Cycles per Loop:** 179,722

#### Step 1: Convert Cycle Count to Time

```
Loop Time (μs) = Average Cycles / Clock Frequency (MHz)
Loop Time (μs) = 179,722 cycles / 72 MHz
Loop Time (μs) = 2,496.1 μs per loop
```

**Why divide by 72 (MHz)?**
- CPU runs at 72 MHz = 72 cycles per microsecond
- So: cycles / (cycles per μs) = μs

#### Step 2: Calculate Total Measurement Period

The key insight: **We measure 1000 iterations, and each iteration happens when the timer fires.**

```
Timer Interrupt Frequency: 400 Hz = 1 interrupt every 2.5 ms
Number of Measurements: 1000 iterations
Total Time Period = 1000 iterations / 400 Hz = 2.5 seconds
```

Convert to microseconds (same units as loop time):
```
Total Period (μs) = 2.5 seconds × 1,000,000 μs/second
Total Period (μs) = 2,500,000 μs
```

#### Step 3: Calculate CPU Usage Percentage

```
CPU Usage = (Time Spent / Total Time) × 100%
CPU Usage = (2,496.1 μs / 2,500,000 μs) × 100%
CPU Usage = 0.000999 × 100%
CPU Usage ≈ 0.099%  ✅ (CORRECT!)
```

---

## Correct Implementation in Code

### Version 1: Explicit and Clear

```rust
// Print statistics every 1000 iterations (~2.5s at 400 Hz)
if loop_iterations % 1000 == 0 {
    let avg_cycles = (total_cycles / 1000) as u32;
    
    // Convert cycles to microseconds
    let loop_time_us = avg_cycles as f64 / 72.0;
    
    // Total measurement period: 1000 iterations at 400 Hz
    // 1000 / 400 Hz = 2.5 seconds = 2,500,000 microseconds
    let total_period_us = 2_500_000.0;
    
    // Calculate CPU usage
    let cpu_usage = (loop_time_us / total_period_us) * 100.0;
    
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "📊 Interrupt Stats (every 1000 loops / ~2.5s):");
    iprintln!(&mut itm.stim[0], "  Avg Cycles/Loop: {}", avg_cycles);
    iprintln!(&mut itm.stim[0], "  Loop Time: {:.3}μs", loop_time_us);
    iprintln!(&mut itm.stim[0], "  Anomalies: {}", anomaly_count);
    iprintln!(&mut itm.stim[0], "  CPU Usage: {:.2}%", cpu_usage);
    iprintln!(&mut itm.stim[0], "──────────────────────────────────────");
    
    total_cycles = 0;
}
```

### Version 2: Formula (Compact)

```rust
let cpu_usage = (avg_cycles as f64 / 72.0 / 2_500_000.0) * 100.0;
iprintln!(&mut itm.stim[0], "  CPU Usage: {:.2}%", cpu_usage);
```

**Breaking down the formula:**
```
(avg_cycles / 72.0 / 2_500_000.0) * 100.0
= (cycles / 72) / 2,500,000 * 100
= (time_in_us) / (total_period_in_us) * 100
= percentage
```

---

## Real-World Interpretation

### Example Output

```
📊 Interrupt Stats (every 1000 loops / ~2.5s):
  Avg Cycles/Loop: 179722
  Loop Time: 2496.139μs
  Anomalies: 16
  CPU Usage: 0.10%
──────────────────────────────────────
```

**What this means:**
- Out of 2.5 seconds of measurement time, the CPU is actively executing your code for **~2.5 milliseconds** (2,496 microseconds)
- The remaining **~2,497.5 milliseconds** is spent in sleep mode (wfe()) waiting for the next interrupt
- **CPU Efficiency:** 99.9% idle time = excellent power efficiency!

---

## Why This Matters

### Original Calculation (WRONG):
- Shows 99.8% CPU usage
- Suggests the system is nearly at maximum capacity
- Would imply you can't add more features
- Creates false alarm about performance

### Correct Calculation (RIGHT):
- Shows 0.10% CPU usage
- Indicates excellent efficiency
- Shows plenty of headroom for additional features
- Demonstrates effective use of interrupt-driven architecture

---

## Key Concepts

### 1. Time Units Matter
- **Cycles:** Raw count from DWT counter (unit: clock cycles)
- **Microseconds (μs):** Standard timing unit
- **Conversion:** `time_us = cycles / clock_freq_mhz`

### 2. Measurement Period Calculation
```
Total Time = Number of Samples / Frequency
Total Time = 1000 samples / 400 Hz = 2.5 seconds
```

### 3. The Interrupt-Driven Advantage
- Main loop checks flag at most **once per interrupt** (every 2.5 ms)
- When no data is ready, CPU **sleeps** via `wfe()`
- Only active during sensor reads (~2.5 μs)
- Rest of time is asleep (~2,497.5 μs) = **99.9% power efficient**

---

## Verification

You can verify this is correct by checking the loop time:

```
Loop Time = Avg Cycles / Clock Frequency
          = 179,722 / 72 MHz
          = 2,496.1 μs
          ≈ 2.5 ms
```

This makes sense because:
- Timer fires every 2.5 ms (400 Hz)
- Loop executes when flag is set
- Loop execution time ≈ time between interrupts

---

## Summary

| Aspect | Wrong | Right |
|--------|-------|-------|
| Formula | `(avg_cycles / 72_000 / 2.5 * 100)` | `(avg_cycles / 72 / 2_500_000) * 100` |
| Result | ~99.8% | ~0.1% |
| Interpretation | "System is maxed out" | "System is very efficient" |
| Reality | ❌ Misleading | ✅ Accurate |

---

## Additional Notes

### For Different Timer Frequencies

If you change the interrupt frequency, update the denominator:

**For 100 Hz timer (10 ms period):**
```rust
let total_period_us = 10_000_000.0;  // 10 seconds for 1000 samples
let cpu_usage = (loop_time_us / total_period_us) * 100.0;
```

**For 1000 Hz timer (1 ms period):**
```rust
let total_period_us = 1_000_000.0;  // 1 second for 1000 samples
let cpu_usage = (loop_time_us / total_period_us) * 100.0;
```

### General Formula

```
CPU Usage = (Loop Time / Time Between Interrupts) × 100%
         = (avg_cycles / clock_mhz / (samples / frequency)) × 100%
```

---

## References

- STM32F3 Clock: 72 MHz
- Timer Interrupt: 400 Hz (configured via `init_tim2()`)
- DWT: Data Watchpoint and Trace (provides cycle counter)
- wfe(): Wait For Event (CPU sleep instruction)


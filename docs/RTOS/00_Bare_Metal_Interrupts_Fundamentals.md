# Bare-Metal Interrupts Fundamentals

## Overview

This document explains the foundational concepts of interrupt-driven programming, transitioning from your current **polling-based** gyroscope reading to **interrupt-driven** event handling.

---

## Part 1: Polling vs Interrupt-Driven Architecture

### Current Architecture: Polling (Blocking)

Your current `main.rs` uses a **polling loop**:

```rust
loop {
    match gyro.read_angular_velocity() {
        Ok((x, y, z)) => {
            // Process data
        }
        Err(e) => { /* handle error */ }
    }
    
    // Busy-wait with NOP loops
    for _ in 0..10_000 {
        cortex_m::asm::nop();
    }
}
```

**Characteristics:**
- ✅ Simple to understand and debug
- ✅ Deterministic timing (predictable loop duration)
- ❌ **Wastes CPU cycles** spinning in busy-wait
- ❌ **Blocks main loop** — can't do anything else while waiting
- ❌ **Poor power efficiency** — CPU running at 100% even when idle
- ❌ Difficult to handle multiple concurrent events

**Timing Flow:**
```
Main Loop
│
├─ Read Sensor
├─ Process Data
├─ Busy-Wait (NOP loop) ←── CPU spinning, wasting energy!
└─ Repeat
```

---

### Interrupt-Driven Architecture

An **interrupt handler** fires at specified intervals (e.g., every 2.5ms for 400Hz data rate), triggering sensor reads **asynchronously**:

```rust
// Timer interrupt handler (fires every 2.5ms)
#[interrupt]
fn TIM2() {
    // Read sensor synchronously on interrupt
    if let Ok((x, y, z)) = GYRO.read_angular_velocity() {
        // Store data safely
        SENSOR_DATA.lock(|data| {
            data.replace((x, y, z));
        });
    }
    
    // Clear interrupt flag (clear timer pending)
    clear_tim2_interrupt();
}

// Main loop can now do useful work or sleep
fn main() -> ! {
    init_timer_interrupt();  // Setup 400Hz timer
    
    loop {
        // CPU can sleep or do other work
        cortex_m::asm::wfe();  // Wait For Event (sleep)
        
        // When interrupt fires, retrieve fresh data
        SENSOR_DATA.lock(|data| {
            if let Some((x, y, z)) = data.borrow().as_ref() {
                iprintln!(&mut itm.stim[0], "X: {}, Y: {}, Z: {}", x, y, z);
            }
        });
    }
}
```

**Characteristics:**
- ✅ **CPU can sleep** between events → better power efficiency
- ✅ **Deterministic timing** — hardware timer guarantees intervals
- ✅ **Responsive** — events trigger immediately
- ✅ **Scalable** — can add multiple interrupt sources
- ❌ Requires **synchronization primitives** (mutexes) for shared data
- ❌ More complex debugging
- ❌ Risk of **interrupt conflicts** if not carefully prioritized

**Timing Flow:**
```
Main Loop (Idle/Sleep)
│
├─ Timer Interrupt Fires (every 2.5ms)
│  ├─ Read Sensor (high priority)
│  ├─ Store Data (atomic/mutex-protected)
│  └─ Return to Main Loop
│
└─ Main Loop wakes, retrieves data, processes
```

---

## Part 2: Synchronization with `cortex_m::interrupt::Mutex`

### The Problem: Race Conditions

Without synchronization, both interrupt handler and main loop might access `SENSOR_DATA` simultaneously:

```rust
static mut SENSOR_DATA: (f32, f32, f32) = (0.0, 0.0, 0.0);

// DANGER: Race condition!
#[interrupt]
fn TIM2() {
    SENSOR_DATA = read_sensor();  // ← Interrupt context
}

fn main() -> ! {
    loop {
        let data = SENSOR_DATA;  // ← Main context
        process(data);
    }
}
```

**What can go wrong:**
- Main loop reads `x` while interrupt writes `y` → corrupted data
- Tearing: `SENSOR_DATA.x` gets old value, `.y` gets new value
- **Data inconsistency** — can be catastrophic in control systems

---

### Solution: `cortex_m::interrupt::Mutex`

**Mutex** (Mutual Exclusion) ensures only one context accesses data at a time:

```rust
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;

// Safely shared static data
static SENSOR_DATA: Mutex<RefCell<Option<(f32, f32, f32)>>> = 
    Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM2() {
    // Write data safely (disables interrupts during critical section)
    SENSOR_DATA.lock(|data| {
        data.borrow_mut().replace(read_sensor());
    });
    clear_tim2_interrupt();
}

fn main() -> ! {
    init_timer_interrupt();
    
    loop {
        // Read data safely
        SENSOR_DATA.lock(|data| {
            if let Some((x, y, z)) = *data.borrow() {
                iprintln!(&mut itm.stim[0], "X: {}", x);
            }
        });
        
        cortex_m::asm::wfe();  // Sleep until next interrupt
    }
}
```

**How it works:**
1. `Mutex::lock(|data| { ... })` temporarily disables all interrupts
2. Inside closure, data access is **exclusive** (no interrupt can preempt)
3. When closure exits, interrupts re-enable
4. `RefCell` provides **interior mutability** (borrow checking at runtime)

**Critical section duration:** Must be as short as possible (data copy, not processing)

---

## Part 3: Timer Interrupts on STM32F3

### Timer Peripherals on STM32F303VC

Your STM32F3 Discovery has multiple timers:

| Timer | Resolution | Max Frequency | Use Case |
|-------|-----------|----------------|----------|
| **SysTick** | 24-bit | ~1 kHz (typical) | OS tick, low-freq timing |
| **TIM2** | 32-bit | Up to 36 MHz | General-purpose, high-precision |
| **TIM3, TIM4** | 16-bit | Up to 36 MHz | General-purpose |
| **DWT** | 32-bit | CPU clock (~72 MHz) | Cycle counting, profiling |

### SysTick vs TIM2 for Sensor Sampling

**SysTick** (Simplest):
- ✅ Always available, no configuration
- ❌ Limited frequency (~1 kHz due to systick overhead)
- ❌ Lower precision

**TIM2** (Recommended):
- ✅ 32-bit counter, high resolution
- ✅ Scalable frequency (1 MHz to 36 MHz with prescaler)
- ✅ Can generate precise 400Hz interrupt (36 MHz ÷ prescaler ÷ period)
- ✅ More flexible

### Calculating TIM2 Interrupt for 400Hz Sampling

**Formula:**
```
Timer Frequency = CPU Clock / Prescaler
Period = Timer Frequency / Desired Interrupt Frequency

For 400Hz on STM32F3 (72 MHz CPU):
Prescaler = 720  (divide 72 MHz by 720 = 100 kHz)
Period = 100 kHz / 400 Hz = 250

So: TIM2 counts to 250, then fires interrupt
```

---

## Part 4: Interrupt Handler Structure

### Basic TIM2 Interrupt Handler

```rust
use cortex_m::interrupt::{self, Mutex};
use stm32f3_discovery::stm32f3xx_hal::pac;

// Global mutable state (interrupt-safe)
static GYRO_DATA: Mutex<RefCell<Option<(f32, f32, f32)>>> = 
    Mutex::new(RefCell::new(None));

static mut TIM2_PERIPHERAL: Option<pac::TIM2> = None;
static mut GYRO: Option<GyroDriver> = None;

// Timer interrupt handler
#[interrupt]
fn TIM2() {
    // Unsafe: We guarantee TIM2 is initialized and not borrowed elsewhere
    unsafe {
        if let Some(ref mut tim2) = TIM2_PERIPHERAL {
            // Clear interrupt flag
            tim2.sr.modify(|_, w| w.uif().clear_bit());
            
            // Read sensor
            if let Some(ref mut gyro) = GYRO {
                if let Ok((x, y, z)) = gyro.read_angular_velocity() {
                    // Store safely
                    GYRO_DATA.lock(|data| {
                        data.borrow_mut().replace((x, y, z));
                    });
                }
            }
        }
    }
}

fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs) = init();
    
    // Initialize timer and interrupt
    let cp = Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    
    // Configure TIM2 for 400Hz
    let mut tim2 = dp.TIM2;
    tim2.psc.write(|w| w.psc().bits(720 - 1));    // 100 kHz
    tim2.arr.write(|w| w.arr().bits(250 - 1));    // 400Hz
    tim2.cr1.write(|w| w.cen().set_bit());        // Enable
    tim2.dier.write(|w| w.uie().set_bit());       // Enable interrupt
    
    // Register handler (cortex-m-rt handles this)
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
    
    // Gyroscope setup
    let gyro = GyroDriver::new(spi, cs);
    unsafe {
        GYRO = Some(gyro);
        TIM2_PERIPHERAL = Some(tim2);
    }
    
    loop {
        // Read shared data safely
        GYRO_DATA.lock(|data| {
            if let Some((x, y, z)) = *data.borrow() {
                iprintln!(&mut itm.stim[0], "X: {:.2} Y: {:.2} Z: {:.2}", x, y, z);
            }
        });
    }
}
```

---

## Part 5: Key Concepts Summary

| Concept | Polling | Bare-Metal Interrupts |
|---------|---------|----------------------|
| **CPU Usage** | 100% (spinning) | ~5% (mostly sleeping) |
| **Power Consumption** | High | Low |
| **Response Time** | Varies (waits for poll) | Instant (hardware timer) |
| **Code Complexity** | Simple | Medium |
| **Synchronization** | Not needed | Mutex required |
| **Scalability** | Poor (blocked by reads) | Good (multiple interrupts) |
| **Debugging** | Easy | Harder (race conditions) |

---

## Part 6: Your Learning Path

```
Phase 1: Polling (Current)
   ↓
Phase 2: SysTick-based Interrupt (Simple)
   ↓
Phase 3: TIM2-based Interrupt with Mutex (Flexible)
   ↓
Phase 4: RTIC Framework (Deterministic, Safe)
```

**Each phase builds on the previous:**
- Phase 1 → 2: Learn hardware interrupts
- Phase 2 → 3: Understand shared state synchronization
- Phase 3 → 4: Simplify synchronization with RTIC's compiler-safe guarantees

---

## Glossary

- **ISR (Interrupt Service Routine)**: Function that executes when interrupt fires
- **Critical Section**: Code where interrupts are disabled (exclusive data access)
- **Race Condition**: When timing of events causes unpredictable results
- **Atomic Operation**: Indivisible operation that can't be interrupted mid-way
- **Prescaler**: Divider that reduces timer frequency (e.g., 72 MHz ÷ 720 = 100 kHz)
- **Context Switch**: Switching execution between interrupt and main loop

---

## Next Steps

→ Read **`01_Verification_and_Testing_Strategy.md`** to learn how to validate each phase works correctly.


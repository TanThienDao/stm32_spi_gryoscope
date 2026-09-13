# RTIC: Real-Time Interrupt-driven Concurrency

## Overview

**RTIC** is a Rust framework that brings **compile-time safety** and **deterministic scheduling** to interrupt-driven real-time systems. This document explains why RTIC is superior to bare-metal interrupts and how it works.

---

## Part 1: Why RTIC? (Problem with Bare-Metal Interrupts)

### Bare-Metal Interrupt Challenges

Even with `Mutex`, bare-metal interrupts have subtle issues:

#### 1.1 Manual Resource Management

**Problem:**

```rust
// You must manually track which interrupt accesses which resource
static SENSOR_DATA: Mutex<RefCell<Option<(f32, f32, f32)>>> = 
    Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM2() {
    // Did you disable interrupts here? Hard to verify!
    SENSOR_DATA.lock(|data| { /* ... */ });
}

#[interrupt]
fn EXTI0() {
    // What if both TIM2 and EXTI0 try to access SENSOR_DATA?
    // Potential deadlock or corruption!
    SENSOR_DATA.lock(|data| { /* ... */ });
}
```

**Question: Is this safe?** You have to reason through it manually.

#### 1.2 Difficult Synchronization Patterns

**Problem:** Different task types need different synchronization:

- **High-priority task** (e.g., sensor read) needs **immediate** access
- **Low-priority task** (e.g., logging) can **wait**
- **Main loop** should **never block** sensor task

```rust
// Bare-metal requires careful lock ordering:
// If low-priority task holds lock when high-priority fires,
// you have **priority inversion**: high-prio waits for low-prio!

#[interrupt]  // Low priority
fn EXTI0() {
    DATA.lock(|d| {  // Hold lock for long operation
        for _ in 0..1_000_000 {
            // Simulate processing
        }
    });  // Finally release
}

#[interrupt]  // High priority
fn TIM2() {
    DATA.lock(|d| {  // BLOCKED! Waiting for EXTI0!
        // This defeats the purpose of high priority
    });
}
```

#### 1.3 Easy to Make Mistakes

- Forget `clear_interrupt_flag()` → handler never fires again
- Use wrong `Mutex` type → potential deadlock
- Static initialization wrong → crashes on second call
- Interrupt priority conflict → unpredictable behavior

---

## Part 2: RTIC Solution

### RTIC Architecture: Compile-Time Safety

RTIC solves these with a **procedural macro** that generates safe code:

```rust
use rtic::app;

#[app(device = stm32f3_discovery::stm32f3xx_hal::pac)]
const APP: () = {
    struct Resources {
        // Shared data — compiler tracks access patterns
        sensor_data: (f32, f32, f32),
        gyro: GyroDriver,
    }
    
    #[task(binds = TIM2, resources = [sensor_data, gyro], priority = 2)]
    fn sensor_read(cx: &mut sensor_read::Context) {
        // Compiler verifies:
        // - Only this task can access sensor_data & gyro with this priority
        // - No possibility of priority inversion
        // - Can't forget to clear interrupt flag
        
        if let Ok((x, y, z)) = cx.resources.gyro.read_angular_velocity() {
            cx.resources.sensor_data = (x, y, z);
        }
    }
    
    #[task(resources = [sensor_data], priority = 1)]
    fn data_log(cx: &mut data_log::Context) {
        // Lower priority: sensor_read can preempt this
        let (x, y, z) = cx.resources.sensor_data;
        iprintln!(&mut itm, "Data: {}, {}, {}", x, y, z);
    }
    
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        // CPU idles here (sleeps) until interrupt
        loop {
            cortex_m::asm::wfe();
        }
    }
};
```

**RTIC generates:**

1. **Interrupt handlers** with priority-aware locking
2. **Task spawning functions** (`sensor_read::spawn()`)
3. **Safe context types** with compile-time resource checks
4. **Automatic initialization** of shared resources

---

### Key RTIC Concepts

#### 1. Tasks

```rust
#[task(binds = TIM2, priority = 2)]  // Interrupt-driven task
fn sensor_read(cx: &mut sensor_read::Context) { }

#[task(priority = 1)]  // Software task (can be spawned)
fn data_process(cx: &mut data_process::Context) { }

#[idle]  // Always running (lowest priority)
fn idle(_cx: idle::Context) -> ! { }

#[init]  // Runs once at startup
fn init(_cx: init::Context) { }
```

**Task Types:**
- **Hardware tasks** (`binds = IRQ`): Triggered by interrupt
- **Software tasks**: Spawned by other tasks programmatically
- **Idle task**: CPU at rest; runs when nothing else does
- **Init task**: Setup; runs before idle starts

---

#### 2. Resources (Shared Data)

```rust
#[app(device = ...)]
const APP: () = {
    struct Resources {
        // Owned by single task (no synchronization needed)
        timer: TIM2,
        
        // Shared between multiple tasks
        #[init = (0.0, 0.0, 0.0)]  // Initial value
        sensor_data: (f32, f32, f32),
        
        counter: u32,
    }
    
    #[task(resources = [sensor_data, counter])]
    fn my_task(cx: &mut my_task::Context) {
        // Access resources through context
        cx.resources.sensor_data.0 = 1.0;
        cx.resources.counter += 1;
    }
};
```

**RTIC's magic:**
- Resources accessed by multiple tasks → compiler generates necessary locking
- Single-task resources → no lock overhead
- **Priority-aware locking**: High-priority task never waits for low-priority

---

#### 3. Priorities and Scheduling

```rust
#[app(device = ...)]
const APP: () = {
    #[task(binds = TIM2, priority = 2)]      // Can preempt everything
    fn high_priority(cx: &mut high_priority::Context) { }
    
    #[task(binds = EXTI0, priority = 1)]     // Can be preempted by high
    fn low_priority(cx: &mut low_priority::Context) { }
    
    #[idle]                                   // Priority 0, lowest
    fn idle(_cx: idle::Context) -> ! { }
};
```

**Execution order:**
```
Idle runs
  ↓ TIM2 fires (priority 2)
High-priority executes
  ↓ (finishes)
Idle resumes
  ↓ EXTI0 fires (priority 1)
Low-priority executes
  ↓ (finishes)
Idle resumes
```

---

#### 4. Spawning and Deferred Execution

```rust
#[task(binds = TIM2, priority = 2)]
fn sensor_read(cx: &mut sensor_read::Context) {
    // High-priority task: read sensor immediately
    // ...process sensor...
    
    // Spawn lower-priority task to log later
    data_process::spawn().unwrap();  // Schedules for next available slot
}

#[task(priority = 1)]
fn data_process(_cx: &mut data_process::Context) {
    // Lower-priority task: logging, UI updates
}
```

**Benefit:** Keeps high-priority task short; defers non-critical work.

---

### Bare-Metal vs RTIC Comparison

#### Bare-Metal Interrupt

```rust
static DATA: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIM2() {
    // You must manually:
    // 1. Clear interrupt flag
    // 2. Handle locking
    // 3. Manage priority conflicts
    DATA.lock(|d| {
        *d.borrow_mut() += 1;
    });
    
    TIM2_PERIPHERAL.sr.modify(|_, w| w.uif().clear_bit());  // Easy to forget!
}
```

**Drawbacks:**
- ❌ Must manually clear interrupt flag
- ❌ Manual locking boilerplate
- ❌ Difficult to prevent priority inversion
- ❌ No compile-time safety
- ❌ Error-prone

---

#### RTIC

```rust
#[app(device = ...)]
const APP: () = {
    struct Resources {
        data: u32,
    }
    
    #[task(binds = TIM2, resources = [data], priority = 2)]
    fn sensor_read(cx: &mut sensor_read::Context) {
        // RTIC compiler:
        // 1. ✅ Generates interrupt handler
        // 2. ✅ Handles locking automatically
        // 3. ✅ Prevents priority inversion
        // 4. ✅ Clears interrupt flag automatically
        
        *cx.resources.data += 1;
    }
};
```

**Advantages:**
- ✅ Automatic interrupt flag clearing
- ✅ Zero-cost locking (priority-aware)
- ✅ Prevents priority inversion by design
- ✅ Compile-time safety (impossible to misuse)
- ✅ Less boilerplate

---

## Part 3: RTIC Advanced Concepts

### 3.1 Generics in RTIC (Shared Resource with Multiple Tasks)

**Scenario:** Multiple tasks read sensor data

```rust
#[app(device = ...)]
const APP: () = {
    struct Resources {
        #[init = (0.0, 0.0, 0.0)]
        sensor_data: (f32, f32, f32),  // Shared
    }
    
    #[task(binds = TIM2, resources = [sensor_data], priority = 2)]
    fn read_sensor(cx: &mut read_sensor::Context) {
        // High priority: write sensor data
        *cx.resources.sensor_data = (1.0, 2.0, 3.0);
    }
    
    #[task(resources = [sensor_data], priority = 1)]
    fn log_data(cx: &mut log_data::Context) {
        // Lower priority: read sensor data
        let (x, y, z) = cx.resources.sensor_data;
        iprintln!(&mut itm, "X: {}", x);
        
        // Can also spawn to defer more work
        process_data::spawn().unwrap();
    }
    
    #[task(resources = [sensor_data], priority = 0)]
    fn process_data(cx: &mut process_data::Context) {
        let (x, y, z) = cx.resources.sensor_data;
        // Process...
    }
};
```

**RTIC's locking strategy:**
- `read_sensor` (priority 2): Holds lock when accessing (no lower-priority can run)
- `log_data` (priority 1): Can run after `read_sensor` finishes
- `process_data` (priority 0): Can run after others finish

**No manual Mutex needed** — RTIC infers locking from priorities!

---

### 3.2 Software Tasks with Arguments

```rust
#[task(priority = 1)]
fn calculate(cx: &mut calculate::Context, value: u32) {
    // Spawned task receives arguments
    iprintln!(&mut itm, "Calculating {}", value);
}

#[task(binds = TIM2, priority = 2)]
fn sensor_read(cx: &mut sensor_read::Context) {
    // Spawn calculate task with data
    calculate::spawn(42).ok();
}
```

---

### 3.3 Late Resources (Init-Dependent)

```rust
#[app(device = ...)]
const APP: () = {
    struct Resources {
        timer: TIM2,  // Not initialized yet
    }
    
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let dp = cx.device;
        let timer = setup_tim2(dp.TIM2);
        
        // Return late resources
        init::LateResources { timer }
    }
    
    #[task(resources = [timer])]
    fn my_task(cx: &mut my_task::Context) {
        // timer is guaranteed to be initialized
        cx.resources.timer.sr.modify(|_, w| w.uif().clear_bit());
    }
};
```

---

## Part 4: RTIC Project Structure

### Standard RTIC Project Layout

```
your_project/
├── Cargo.toml
├── src/
│   ├── main.rs          # RTIC app macro
│   ├── lib.rs           # Shared drivers/utilities
│   ├── drivers/
│   │   ├── gyro.rs      # Gyroscope driver
│   │   └── uart.rs      # UART logging
│   └── tasks/
│       ├── sensor.rs    # Sensor reading task
│       └── logging.rs   # Logging task
└── .cargo/config.toml
```

### main.rs with RTIC

```rust
#![no_std]
#![no_main]

use rtic::app;

#[app(device = stm32f3_discovery::stm32f3xx_hal::pac)]
const APP: () = {
    struct Resources {
        // Shared data
    }
    
    #[init]
    fn init(cx: init::Context) {
        // Initialization code
    }
    
    #[task(binds = TIM2, priority = 2)]
    fn sensor_read(cx: &mut sensor_read::Context) {
        // High-priority sensor reading
    }
    
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfe();
        }
    }
};
```

---

## Part 5: RTIC vs Polling vs Bare-Metal Comparison

| Aspect | Polling | Bare-Metal Interrupts | RTIC |
|--------|---------|----------------------|------|
| **CPU Usage** | 100% | ~5% | ~5% |
| **Power Consumption** | Highest | Low | Low |
| **Code Safety** | Manual | Manual + Mutex | Compile-time guaranteed |
| **Priority Handling** | None | Manual/error-prone | Automatic |
| **Synchronization** | N/A | Complex | Automatic locking |
| **Ease of Use** | Simple | Medium | Medium (macro learning curve) |
| **Scaling (many tasks)** | Poor | Difficult | Excellent |
| **Debugging** | Easy | Hard | Medium |
| **Compiler Errors** | Few | Few | Many (until you learn) |

---

## Part 6: RTIC Learning Curve

### Common RTIC Mistakes (and How to Avoid)

**Mistake 1: Forgetting `priority` field**

```rust
// ❌ Compilation error!
#[task(resources = [data])]
fn my_task(cx: &mut my_task::Context) { }

// ✅ Correct
#[task(priority = 1, resources = [data])]
fn my_task(cx: &mut my_task::Context) { }
```

**Mistake 2: Accessing non-declared resources**

```rust
// ❌ Compilation error!
#[task(priority = 1)]
fn my_task(cx: &mut my_task::Context) {
    cx.resources.some_data;  // Not declared in #[task(...)]
}

// ✅ Correct
#[task(priority = 1, resources = [some_data])]
fn my_task(cx: &mut my_task::Context) {
    cx.resources.some_data;  // Declared in #[task(...)]
}
```

**Mistake 3: Spawning task with wrong argument type**

```rust
// ❌ Runtime error
#[task(priority = 1)]
fn calculate(cx: &mut calculate::Context, value: u32) { }

#[task(priority = 2)]
fn caller(cx: &mut caller::Context) {
    calculate::spawn("wrong type").ok();  // Should be u32
}

// ✅ Correct
#[task(priority = 2)]
fn caller(cx: &mut caller::Context) {
    calculate::spawn(42u32).ok();
}
```

---

## Part 7: When to Use RTIC

### Good Use Cases for RTIC

✅ **Real-time systems** with multiple concurrent tasks
✅ **Sensor fusion** (multiple sensors at different rates)
✅ **Communication stacks** (serial, CAN, Ethernet)
✅ **Control systems** (motors, servos, regulators)
✅ **IoT/Connected devices** (async networking + sensor reads)

### When Bare-Metal Interrupts Might Be Enough

⚠️ **Single-task systems** (one interrupt, one main loop)
⚠️ **Simple polling** (doesn't need real-time guarantees)
⚠️ **Resource-constrained** (very small RAM)

**Recommendation for your project:** Use RTIC. You have multiple tasks (sensor read, logging, potential data processing), and RTIC's safety will prevent bugs.

---

## Next Steps

→ Read **`03_Learning_Progression_Roadmap.md`** for step-by-step implementation guide.

→ Start with **Phase 1** (verify your current polling system works).

→ Move to **Phase 2** (add bare-metal TIM2 interrupt).

→ Finally, **Phase 3** (migrate to RTIC).


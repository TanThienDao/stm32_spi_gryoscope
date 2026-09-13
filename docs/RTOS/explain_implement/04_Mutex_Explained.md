# Mutex Explained: Interrupt-Safe Data Sharing for Phase 2

**What You'll Learn:**
- What a Mutex is and why you need it
- How it prevents race conditions between ISR and main loop
- How `cortex_m::interrupt::Mutex` works in embedded Rust
- Practical examples for your Phase 2 implementation

---

## 🎯 Quick Summary

**Mutex** = **Mut**ual **Ex**clusion lock

A **synchronization primitive** that ensures **only one piece of code can access a shared resource at a time**, preventing data corruption when your **ISR** and **main loop** both access the same data.

---

## ⚠️ The Problem: Race Conditions

### Scenario: Shared Sensor Data

Imagine your gyroscope sensor data is accessed by both:
1. **ISR (TIM2_Update)** — Reads sensor every 2.5ms, updates global data
2. **Main Loop** — Reads the global data to print and process

```
Time:  Main Loop              ISR (runs every 2.5ms)     Data State
──────────────────────────────────────────────────────────────────
1      read x_value
2      read y_value           
3      read z_value           
4                            [INTERRUPT FIRES!]
5                            Update x=100 ✍️
6                            Update y=200 ✍️
7                            Update z=300 ✍️
8      print x, y, z          ← CORRUPTED!
       (x=100, y=old_y, z=old_z)  Mixed old and new values!
```

### What Happens (❌ UNSAFE Code):

```rust
// Shared data WITHOUT Mutex (DANGEROUS!)
static mut SENSOR_DATA: (f32, f32, f32) = (0.0, 0.0, 0.0);

// Main Loop (unsafe because data can change mid-access)
fn main() {
    loop {
        let (x, y, z) = unsafe { SENSOR_DATA };  // ← Read mid-update!
        println!("X: {}, Y: {}, Z: {}", x, y, z);  // ← Could be corrupted
    }
}

// ISR (updates all three values)
#[interrupt]
fn TIM2_Update() {
    unsafe {
        SENSOR_DATA = (100.0, 200.0, 300.0);  // ← Race condition!
    }
}
```

**Result:** Unpredictable behavior, corrupted sensor values, hard-to-debug bugs!

---

## ✅ The Solution: Mutex with Critical Sections

### How Mutex Works

`cortex_m::interrupt::Mutex` **disables interrupts temporarily** while accessing shared data:

```rust
use cortex_m::interrupt::{Mutex, free};
use core::cell::RefCell;

// Shared data WITH Mutex (SAFE!)
static SENSOR_DATA: Mutex<RefCell<(f32, f32, f32)>> = 
    Mutex::new(RefCell::new((0.0, 0.0, 0.0)));

// ISR: Write safely
#[interrupt]
fn TIM2_Update() {
    free(|cs| {  // Enter critical section (interrupts disabled!)
        // Interrupts are DISABLED here
        let mut data = SENSOR_DATA.borrow_mut(cs);
        *data = (100.0, 200.0, 300.0);  // ← Safe write
        // Interrupts RE-ENABLED when exiting this block
    });
}

// Main Loop: Read safely
fn main() {
    loop {
        let (x, y, z) = free(|cs| {  // Disable interrupts
            *SENSOR_DATA.borrow_mut(cs)  // ← Safe read (no interrupt interference)
        });  // Re-enable interrupts
        
        println!("X: {}, Y: {}, Z: {}", x, y, z);  // Data is ALWAYS consistent!
    }
}
```

**Timeline with Mutex (✅ SAFE):**

```
Time:  Main Loop                ISR (TIM2_Update)         Status
──────────────────────────────────────────────────────────────────
1      free(|cs| {
2          read x, y, z        [BLOCKED - CS locked]      🔒 Locked
3      })
4                              free(|cs| {                
5                              write x, y, z              🔒 Locked
6                              })
7      Data is consistent!     ✅ No overlap, no corruption
```

---

## 🔧 Component Breakdown

### 1. `Mutex<T>` — The Lock Container

```rust
static SENSOR_DATA: Mutex<RefCell<(f32, f32, f32)>> = 
    Mutex::new(RefCell::new((0.0, 0.0, 0.0)));
```

- Wraps your data to prevent concurrent access
- Enforces: **"Only one reader/writer at a time"**
- In `cortex_m`, uses critical sections (interrupt disable/enable)

### 2. `RefCell<T>` — Interior Mutability

```rust
RefCell::new((0.0, 0.0, 0.0))
```

- Allows **mutable access** without declaring the static as `mut`
- Static mutables are inherently unsafe in Rust
- `RefCell` provides **runtime borrow checking** instead
- Pairs perfectly with `Mutex` for safe synchronization

### 3. `free(|cs| { ... })` — Critical Section

```rust
free(|cs| {
    // Interrupts are DISABLED here
    let data = SENSOR_DATA.borrow_mut(cs);
    *data = (100.0, 200.0, 300.0);
    // Interrupts are RE-ENABLED when exiting this block
});
```

**Components:**
- **`free()`**: Function to enter critical section
- **`|cs|`**: `cs` is a "critical section token" (proof you're in critical section)
- **`borrow_mut(cs)`**: Requires the token to access data (type safety!)
- **Automatic restoration**: Interrupts re-enabled automatically when block ends

---

## 📊 Visual Comparison

### Without Mutex (❌ UNSAFE):

```
Main Loop:              ISR:
x = read                
y = read                
                        [INTERRUPT!]
                        x = 100
                        y = 200  ← ISR mid-update!
z = read (old z)
print x, y, z           ← CORRUPTED! (mixed old/new)
```

### With Mutex (✅ SAFE):

```
Main Loop:              ISR:
free(|cs| {
  lock SENSOR_DATA
  read x, y, z                [WAITING - blocked]
})
                        free(|cs| {
                          lock SENSOR_DATA (now available)
                          write x, y, z
                        })
print x, y, z           ← CONSISTENT! (all new values)
```

---

## 💻 Real Phase 2 Example

### Step 1: Define Shared Data

```rust
use cortex_m::interrupt::{Mutex, free};
use core::cell::RefCell;

// Shared sensor reading
static SENSOR_DATA: Mutex<RefCell<(f32, f32, f32)>> = 
    Mutex::new(RefCell::new((0.0, 0.0, 0.0)));

// Flag indicating new data is available
static NEW_DATA_READY: Mutex<RefCell<bool>> = 
    Mutex::new(RefCell::new(false));

// Global gyroscope driver
static GYRO: Mutex<RefCell<Option<GyroDriver>>> = 
    Mutex::new(RefCell::new(None));
```

### Step 2: ISR Writes Safely

```rust
#[interrupt]
fn TIM2_Update() {
    free(|cs| {
        // Critical section: interrupts disabled
        
        // Clear interrupt flag
        let dp = unsafe { pac::Peripherals::steal() };
        dp.TIM2.sr.write(|w| w.uif().clear_bit());
        
        // Try to read from gyroscope
        if let Ok(mut gyro_ref) = GYRO.borrow_mut(cs).borrow_mut() {
            if let Some(ref mut gyro) = *gyro_ref {
                if let Ok((x, y, z)) = gyro.read_angular_velocity() {
                    // Update shared data (SAFE - interrupts disabled)
                    *SENSOR_DATA.borrow_mut(cs) = (x, y, z);
                    // Signal new data available
                    *NEW_DATA_READY.borrow_mut(cs) = true;
                }
            }
        }
        // Interrupts automatically re-enabled when exiting free()
    });
}
```

### Step 3: Main Loop Reads Safely

```rust
fn main() -> ! {
    // ... initialization code ...
    
    loop {
        // Check if new data is available
        let data_ready = free(|cs| {
            *NEW_DATA_READY.borrow_mut(cs)
        });
        
        if data_ready {
            // Read sensor data from shared buffer (SAFE)
            let (x, y, z) = free(|cs| {
                *SENSOR_DATA.borrow_mut(cs)
            });
            
            // Clear flag for next read
            free(|cs| {
                *NEW_DATA_READY.borrow_mut(cs) = false;
            });
            
            // Process data (no race condition!)
            println!("X: {:.2}°/s | Y: {:.2}°/s | Z: {:.2}°/s", x, y, z);
        }
        
        // If no data, CPU sleeps here (low power!)
    }
}
```

---

## 🔄 How `borrow_mut()` Works

### The Type Safety Chain

```rust
// Static definition
static SENSOR_DATA: Mutex<RefCell<(f32, f32, f32)>> = ...;
                    ^^^^^  ^^^^^^
                    |      |
                    |      └── Provides mutable access without `mut` declaration
                    └────────── Prevents concurrent access

// Inside critical section
free(|cs| {
    SENSOR_DATA.borrow_mut(cs)  // Takes critical section token
    // ↓
    // Returns: &mut RefCell<(f32, f32, f32)>
    
    .borrow_mut()  // Opens the RefCell mutably
    // ↓
    // Returns: RefMut<(f32, f32, f32)>
    
    // Can now dereference and modify:
    *data = (100.0, 200.0, 300.0);  // ← Safe write!
});
```

---

## ⚡ Performance Impact

### Overhead of Mutex

When you use `free(|cs| { ... })`:
1. **Disable interrupts** (~10 CPU cycles)
2. **Access data** (microseconds)
3. **Re-enable interrupts** (~10 CPU cycles)

**Total overhead:** Microseconds (negligible!)

### For Your Phase 2

```
Each sensor read:
├─ free(|cs| { read from SENSOR_DATA })     ← ~100 cycles
├─ Main processing                          ← ~1,000 cycles
└─ Total per loop                           ← ~1,100 cycles

At 72 MHz: ~15 microseconds per loop
CPU usage: < 0.1% ✓
```

---

## ⚠️ Common Mistakes

### ❌ Mistake 1: Forgetting `free()`

```rust
// WRONG - Compiler error!
let data = SENSOR_DATA.borrow_mut();  // ← Error: missing `cs` token
```

### ❌ Mistake 2: Holding lock too long

```rust
// WRONG - Interrupts disabled too long!
free(|cs| {
    let data = SENSOR_DATA.borrow_mut(cs);
    // 100ms of processing here... ← Interrupts stay disabled! Bad!
    do_lots_of_work();  // ← Interrupts still disabled!
});
```

**Better:**

```rust
// RIGHT - Hold lock briefly
let (x, y, z) = free(|cs| {
    *SENSOR_DATA.borrow_mut(cs)  // ← Quick read
});

// Then release lock
do_lots_of_work();  // ← Interrupts enabled now
```

### ❌ Mistake 3: Forgetting `RefCell`

```rust
// WRONG - Won't compile
static SENSOR_DATA: Mutex<(f32, f32, f32)> = ...;
                         ↑ Missing RefCell!

free(|cs| {
    SENSOR_DATA.borrow_mut(cs);  // ← Error: (f32, f32, f32) has no borrow_mut()
});
```

---

## 📚 Comparison: Mutex in Different Contexts

| Context | Mutex Type | Blocking? | Use Case |
|---------|-----------|-----------|----------|
| **Multi-threaded OS** | `std::sync::Mutex` | Yes (threads wait) | General concurrency |
| **Embedded (no ISR)** | `parking_lot::Mutex` | Yes (spinlock) | Light multi-threading |
| **Embedded (ISR safe)** | `cortex_m::interrupt::Mutex` | No (disables interrupts) | ISR ↔ Main sync |
| **Your Phase 2** | `cortex_m::interrupt::Mutex` | ✓ Best choice | Interrupt-safe data sharing |

---

## 🎯 Key Takeaways

| Aspect | Without Mutex | With Mutex |
|--------|---------------|-----------|
| **Race Conditions** | ❌ Possible | ✅ Prevented |
| **Data Consistency** | ❌ Can be corrupted | ✅ Always correct |
| **Code Safety** | ❌ Requires `unsafe` | ✅ Safe abstraction |
| **Interrupt Handling** | ❌ Manual disable/enable | ✅ Automatic management |
| **Performance** | Fast but unsafe | Microseconds overhead |
| **Predictability** | Unpredictable | Deterministic |

---

## ✅ Checklist: Using Mutex in Phase 2

- [ ] Import `use cortex_m::interrupt::{Mutex, free};`
- [ ] Import `use core::cell::RefCell;`
- [ ] Wrap shared data in `Mutex<RefCell<T>>`
- [ ] Always access data inside `free(|cs| { ... })`
- [ ] Keep critical sections brief
- [ ] Don't call blocking functions inside `free()`
- [ ] Test both ISR and main loop access patterns

---

## 🚀 Next Steps

1. **Implement Phase 2** with the Mutex pattern
2. **Test interrupt-safe data sharing**
3. **Verify no race conditions** (watch for corrupted values)
4. **Measure performance** (should be ~1,000 cycles/loop)

---

## 📖 Additional Resources

- [cortex-m crate docs](https://docs.rs/cortex-m/latest/cortex_m/interrupt/)
- [Rust Book: Interior Mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
- [ARM Cortex-M Interrupts](https://interrupt.memfault.com/)

---

**Now you're ready to implement Phase 2 with interrupt-safe data sharing! 🎉**


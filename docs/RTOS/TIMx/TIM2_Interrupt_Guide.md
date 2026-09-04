# TIM2 Interrupt Guide: Complete Implementation

## Table of Contents
* [1. Quick Start](#quick-start)
   * [The Problem We're Solving](#the-problem-were-solving)
   * [Setup](#setup)
* [2. Concepts](#concepts)
   * [What is TIM2?](#what-is-tim2)
   * [Current Architecture: Polling (Blocking)](#current-architecture-polling-blocking)
* [3. TIM2 Hardware Architecture](#tim2-hardware-architecture)
   * [Simplified Block Diagram](#simplified-block-diagram)
   * [Signal Flow in One Complete Cycle](#signal-flow-in-one-complete-cycle)
* [4. Configuration Steps](#configuration-steps)
* [5. Datasheet Reference Map](#datasheet-reference-map)
* [6. Troubleshooting](#troubleshooting)
* [7. Next Steps](#next-steps)


---

## Quick Start

### The Problem We're Solving

**Without interrupts (polling):**
```
Main loop continuously reads sensor in a spin loop
CPU: 100% busy, wastes cycles with NOP loops
```

**With TIM2 interrupts:**
```
Hardware timer fires every 2.5ms (400 Hz), triggers ISR to read sensor
CPU: ~5% busy, sleeps 95% of the time
```
### Setup

```rust
// In auxiliary/src/lib.rs (ALREADY DONE):
let mut tim2 = Tim2Guard::new();
tim2.init(&mut dp.TIM2);

// In main.rs:
let mut nvic_guard = NvicGuard::new();
nvic_guard.unmask_tim2_safe()?;

// Define ISR:
#[no_mangle]
pub extern "C" fn TIM2() {
    Tim2Guard::check_and_clear_uif();  // Clear flag
    interrupt::free(|cs| {
        *NEW_DATA_READY.borrow(cs).borrow_mut() = true;
    });
}

// Main loop waits for data:
loop {
    let should_sleep = interrupt::free(|cs| {
        let mut ready = NEW_DATA_READY.borrow(cs).borrow_mut();
        if *ready {
            *ready = false;
            // Process sensor data
            false  // Don't sleep
        } else {
            true   // Signal to sleep
        }
    });
    
    if should_sleep {
        cortex_m::asm::wfe();  // Sleep until interrupt
    }
}
```
***
## Concepts

### What is TIM2?

TIM2 is a **32-bit general-purpose timer** on STM32F303VC:
- **32-bit counter** ? Very long measurement intervals
- **Flexible prescaler** ? Divide 72 MHz to any frequency
- **Interrupt-driven** ? Hardware fires at precise intervals
- **Low CPU overhead** ? CPU sleeps while timer counts


**Why not SysTick or DWT?**

### Current Architecture: Polling (Blocking)

```rust
// Your current main.rs approach:
loop {
    match gyro.read_angular_velocity() {
        Ok((x, y, z)) => {
            // Process data
        }
        Err(e) => { /* handle error */ }
    }
    
    // Busy-wait with NOP loops (CPU wasting cycles!)
    for _ in 0..10_000 {
        cortex_m::asm::nop();
    }
}
```

**Timeline:**
```
Time:     0ms        2.5ms       5ms         7.5ms
          |          |           |           |
Main Loop READ    WAIT/NOP    READ      WAIT/NOP
          PROC    PROC/WAIT   PROC      PROC/WAIT
          WAIT    WAIT        WAIT      WAIT
          |          |           |           |
CPU Usage 100%      100%        100%       100%
```

**Problems:**
- ❌ CPU never sleeps (100% usage even when idle)
- ❌ Timing depends on NOP loops (unpredictable)
- ❌ Can't do other work while looping
- ❌ High power consumption
- ❌ Blocks on sensor I/O (SPI communication is slow)


**Why TIM2 over alternatives?**

| Feature | SysTick | TIM2 | DWT |
|---------|---------|------|-----|
| Resolution | 24-bit | 32-bit | 32-bit |
| General timing | ? | ? | ? (profiling only) |
| Flexible frequency | ? | **?** | ? |
| Best for | System tick | **Sensor sampling** | Cycle counting |

* * *

## TIM2 Hardware Architecture

### Simplified Block Diagram

```
+---------------------------------------------------------------+
|                       TIM2 (Timer 2)                          |
+---------------------------------------------------------------+
|                                                               |
|  +---------------------------------------------------------+  |
|  | 32-bit Counter (CNT Register)                           |  |
|  | Current value: 0 -> 1 -> 2 -> ... -> ARR -> 0 (reload)  |  |
|  +---------------------------------------------------------+  |
|                            |                                  |
|  +---------------------------------------------------------+  |
|  | Prescaler Divider (PSC)                                 |  |
|  | Divides clock: 72 MHz / 720 = 100 kHz                   |  |
|  | Counter increments every 10 us                          |  |
|  +---------------------------------------------------------+  |
|                                                               |
|  +---------------------------------------------------------+  |
|  | Auto-Reload Register (ARR)                              |  |
|  | Reload value: 249 (counts 0..249, then reloads)         |  |
|  | Overflow after 250 x 10 us = 2.5 ms                     |  |
|  +---------------------------------------------------------+  |
|                            |                                  |
|  +---------------------------------------------------------+  |
|  | Update Event Generator                                  |  |
|  | Fires when counter reloads                              |  |
|  | Sets Update Interrupt Flag (UIF)                        |  |
|  +---------------------------------------------------------+  |
|                            |                                  |
|  +---------------------------------------------------------+  |
|  | Interrupt Enable (DIER.UIE)                             |  |
|  | If UIE=1: Send interrupt to CPU                         |  |
|  | If UIE=0: Ignore update event                           |  |
|  +---------------------------------------------------------+  |
|                            |                                  |
|  +---------------------------------------------------------+  |
|  | NVIC (Nested Vectored Interrupt Controller)             |  |
|  | Decides if CPU should accept interrupt                  |  |
|  | (Depends on NVIC mask, priority)                        |  |
|  +---------------------------------------------------------+  |
|                            |                                  |
|              CPU Interrupt Handler                            |
|              (Your #[interrupt] fn TIM2() {...})              |
|                                                               |
+---------------------------------------------------------------+
```

### Signal Flow in One Complete Cycle

```
+---------------------------------------------------------------+
| TIME: 0 us                                                    |
| Counter = 0, starts incrementing                              |
+---------------------------------------------------------------+
|
+---------------------------------------------------------------+
| TIME: 10 us                                                   |
| Counter increments by 1 (every clock cycle at 100 kHz)        |
| Counter = 1                                                   |
+---------------------------------------------------------------+
|
[Repeat 248 more times...]
|
+---------------------------------------------------------------+
| TIME: 2500 us (2.5 ms)                                        |
| Counter reaches 249 (ARR value, which is set to 250-1)        |
| Next clock pulse causes overflow                              |
+---------------------------------------------------------------+
|
+---------------------------------------------------------------+
| UPDATE EVENT TRIGGERED!                                       |
| 1. Counter reloads to 0                                       |
| 2. Hardware sets UIF flag (Update Interrupt Flag)             |
| 3. If DIER.UIE = 1: Signal NVIC                               |
+---------------------------------------------------------------+
|
+---------------------------------------------------------------+
| CPU RECEIVES INTERRUPT!                                       |
| Saves current register state                                  |
| Jumps to #[interrupt] fn TIM2() {...}                         |
+---------------------------------------------------------------+
|
+---------------------------------------------------------------+
| YOUR ISR CODE EXECUTES                                        |
| 1. Read sensor (gyro.read_angular_velocity())                 |
| 2. Store data (with Mutex protection)                         |
| 3. Clear UIF flag (critical!)                                 |
+---------------------------------------------------------------+
|
+---------------------------------------------------------------+
| ISR RETURNS                                                   |
| CPU restores registers and continues main loop                |
| Cycle repeats every 2.5 ms                                    |
+---------------------------------------------------------------+
```

* * *
* * *
## Configuration Steps

### Overview: The 5-Step Setup

```
Step 1: Set Prescaler (PSC)         -> Divide clock frequency
        |
        v
Step 2: Set Period (ARR)            -> Set reload value
        |
        v
Step 3: Enable Timer (CR1.CEN)      -> Start counting
        |
        v
Step 4: Enable Interrupt (DIER.UIE) -> Allow interrupts
        |
        v
Step 5: Unmask in NVIC              -> CPU accepts interrupts
```

---

### Step 0: Initialize Timer (Tim2Guard)

- 9.4.8 APB1 peripheral clock enable register (RCC_APB1ENR) -> Page:155.
```
Bit 0 TIM2EN: TIM2 timer clock enable
Set and cleared by software.
0: TIM2 clock disabled
1: TIM2 clock enabled
```

- Code :
```rust
pub fn init() -> (...) {
    let dp = pac::Peripherals::take().unwrap();
    
    // Enable TIM2 clock
    dp.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());
    
    // Initialize TIM2 (sets PSC=719, ARR=249 for 400 Hz)
    let mut tim2 = Tim2Guard::new();
    tim2.init(&mut dp.TIM2);
    
    // ... return peripherals ...
}
```

**What Tim2Guard::init() does:**
0. Calculates PSC and ARR for desired frequency
1. Writes TIM_PSC (prescaler) register
2. Writes TIM_ARR (reload) register
3. Sets CR1.CEN = 1 (start counting)
4. Sets DIER.UIE = 1 (enable interrupts)

* * *

### Step 1: Calculate and Set Prescaler (PSC)

**Goal:** Convert CPU clock to a manageable frequency

**Your System:**
- CPU Clock: **72 MHz** (STM32F303VC)
- Desired Timer Frequency: **100 kHz** (good balance of precision and period)
- Formula: `Timer Frequency = CPU Clock / (Prescaler + 1)`

- **Calculation:**
```
72 MHz / (PSC + 1) = 100 kHz
72,000,000 / 100,000 = PSC + 1
720 = PSC + 1
PSC = 719
```

**Why +1?**
The hardware counts from 0, so if you write `PSC = 719`, the divider counts:
- 0, 1, 2, ..., 719 = 720 total states
- Each state = 1 clock cycle at CPU frequency
- Result: divides by 720

**Register:**
```rust
// In Rust using PAC (Peripheral Access Crate):
tim2.psc.write(|w| w.psc().bits(719));
```

**Verification:**
```
Prescaler output frequency = 72 MHz / 720 = 100 kHz ?
Time per tick = 1 / 100 kHz = 10 �s ?
```
* * *
### Step 2: Set Period/Reload Value (ARR)

**Goal:** Define when the overflow (interrupt) fires
Configuration & Implementation
**Your System:**
- Timer Frequency: 100 kHz (from step 1)
- Desired Interrupt Frequency: 400 Hz (for gyroscope sampling)
- Formula: `Period = Timer Frequency / Desired Interrupt Frequency`

**Calculation:**
```
Period = 100 kHz / 400 Hz = 250

So counter counts: 0, 1, 2, ..., 249 = 250 states
Then overflows and reloads to 0
```

**Hardware behavior:**
- Counter starts at 0 after reset/reload
- Increments by 1 every 10 us (prescaler output)
- When it reaches ARR, next clock causes overflow
- Overflow triggers reload back to 0 and fires interrupt

**Why ARR = 249 (not 250)?**
The hardware loads ARR into counter when comparing. To get 250 counts, you store `ARR = 249`:
- Counts: 0 ? 1 ? 2 ? ... ? 249 = 250 total increments
- After 249 counts, the 250th rising edge causes overflow

**Register:**
```rust
// Set ARR = 249
tim2.arr.write(|w| w.arr().bits(249));

```

**Verification:**
```
Overflow period = 250 ticks x 10 us/tick = 2500 us = 2.5 ms
Interrupt frequency = 1000 ms / 2.5 ms = 400 Hz (1s /2.5ms)
```

* * *

### Step 3: Enable the Timer (CR1 Register)

**Goal:** Start the counter counting

**Register Bits:**
```
Bit 0: CEN (Counter Enable)
  0 = Counter disabled (stopped)
  1 = Counter enabled (starts counting)
```

**Rust Code:**
```rust
// Enable counter
  tim2.cr1.write(|w| {
               w.cen().set_bit()         // Counter enabled
                .udis().clear_bit()      // Update event enabled
                .dir().clear_bit()       // Count up
                .arpe().clear_bit()      // Auto-reload preload disabled
        });

```

**What happens:**
```
Before: CEN = 0
  Counter is stopped, value remains at 0

After: CEN = 1
  Counter starts incrementing every 10 �s
  After 2.5 ms ? overflow ? interrupt fires
  -> Counter reloads to 0
  -> Repeats forever
```
* * *

### Step 4: Enable the TIM2 interrupt on update event (UIE)

Goal: Tell the TIM2 hardware to send an interrupt signal to the CPU when the counter overflow.

**What happens:**
``` text
**Without UIE (UIE = 0):**
* Counter counts and overflows normally
* Hardware sets the UIF flag (Update Interrupt Flag)
* **BUT** no interrupt signal is sent to CPU
  * CPU never knows an overflow happened

**With UIE (UIE = 1):**
* Counter counts and overflows normally
* Hardware sets the UIF flag
* Hardware **ALSO** signals NVIC (CPU's interrupt controller)
  * NVIC accepts/forwards the interrupt to CPU
  * Your ISR (`#[interrupt] fn TIM2() {...}`) executes
```

Rust code :

``` rust
//Enable TIM2 interrupt on update event (UIE)
tim2.dier.write(|w| w.uie().set_bit());
```

* * *

### Step 5: Unmask NVIC (Allow CPU to Accept Interrupts)

**Goal:** Allow CPU to accept TIM2 interrupts

The NVIC (Nested Vectored Interrupt Controller) is the CPU's interrupt manager. It can **mask** (block) or **unmask** (allow) interrupts.

**Rust Code:**
``` rust
use cortex_m::peripheral::NVIC;

// Unmask TIM2 interrupt (allow CPU to accept it)
unsafe {
    NVIC::unmask(pac::Interrupt::TIM2);
	// todo There are some issue setting priority, too lazy to fix it. 
    let mut nvic = Peripherals::steal().NVIC;
    nvic.set_priority(pac::Interrupt::TIM2, 100);
}
```

**What this does:**
- Sets the NVIC mask bit for TIM2
- unmask() tells CPU: "accept TIM2 interrupts"
   - Without this, interrupts fire but CPU ignores them
- When TIM2 fires, CPU will accept the interrupt
   - Without this, NVIC blocks TIM2 signals (interrupt never reaches CPU)

**Why `unsafe`?**
- Modifying NVIC can cause race conditions if interrupts fire while you're changing masks. Rust requires `unsafe` to acknowledge this risk.
- Solution to wrap it and init 1,  (todo) ->  **Not sure if this resilt an isse tho?** there are suggestion .
-  Better solution to use RTIC (will implement in Phase 3)
* * *
### Step 6: Define Interrupt Handler

**In main.rs:**

```rust
#[no_mangle]
pub extern "C" fn TIM2() {
    use auxiliary::interrupt_handler::NEW_DATA_READY;
    use cortex_m::interrupt;
    
    // CRITICAL: Clear the interrupt flag
    Tim2Guard::check_and_clear_uif();
    
    // Signal main loop that data is ready
    interrupt::free(|cs| {
        *NEW_DATA_READY.borrow(cs).borrow_mut() = true;
    });
}
```

**Why interrupt::free()?**
- Disables interrupts during the closure (prevents race conditions)
- Duration: ~0.1�s (very short)
- Ensures atomic read-modify-write of shared flag

---

### ⚠️ All step above os to set up TIM2 and prepare interrupt.
* * *
* * *
## Datasheet Reference Map

### ⚠️ Important: You Must Download the Official STM32F303 Reference Manual

These reference sections are from the **official STM32F303 Reference Manual (RM0316)** published by STMicroelectronics.

**✅ Correct Manual for Your Chip:**
- Document: **RM0316** ⭐ CORRECT
- Full Title: "STM32F303xB/C/D/E, STM32F303x6/8, STM32F328x8, STM32F358xC, STM32F398xE"
- Applies to: STM32F303, STM32F328, STM32F358, STM32F398
- Your Board: STM32F3 Discovery with STM32F303VC ✔️

**Download it here:**
1. Go to: https://www.st.com
2. Search: **"RM0316"** (or search "STM32F303")
3. Look for: "STM32F303xB/C/D/E Reference Manual"
4. Download the PDF (approximately 1.5 MB)


---

### Table: TIM2 Registers (From RM0316, Chapter 14) ✔️ VERIFIED

| Concept | Register | RM0316 Location | Offset | Bits | Purpose |
|---------|----------|---|--------|------|---------|
| **Counter Value** | TIM_CNT | Ch. 21.4.12 | 0x24 | [31:0] | Current counter value (read-only during counting) |
| **Prescaler Divider** | TIM_PSC | Ch. 21.4.14 | 0x28 | [15:0] | Frequency divider (Timer Freq = CPU / (PSC+1)) |
| **Auto-Reload Value** | TIM_ARR | Ch. 21.4.15 | 0x2C | [31:0] | Reload value (counter counts 0 to ARR, then wraps) |
| **Counter Enable** | TIM_CR1 | Ch. 21.4.1 | 0x00 | [0] | Start/stop counter (1=enabled) |
| **Update Interrupt Enable** | TIM_DIER | Ch. 21.4.4,| 0x0C | [0] | Enable interrupt on overflow (1=enabled) |
| **Update Interrupt Flag** | TIM_SR | Ch. 21.4.5 | 0x10 | [0] | Overflow occurred (must clear in ISR!) |
| **Clear Interrupt Flag** | TIM_SR | Ch. 21.4.5 | 0x10 | [0] | Write 0 to clear (W0C = write 0 to clear) |

### Register Detailed Descriptions

**TIM_CR1 (Control Register 1) - Offset: 0x00**
```
Bit 0: CEN (Counter ENable)
  0 = Counter disabled (stopped)
  1 = Counter enabled (counting)

Bit 1: UDIS (Update DISable)
  0 = Generate update event on overflow (standard)
  1 = Don't generate update event (rarely used)

Bit 2: URS (Update Request Source)
  0 = Update event from overflow or software write to EGR
  1 = Update event only from overflow

Bit 4: DIR (DIRection)
  0 = Counter counts up (standard for us)
  1 = Counter counts down (advanced feature)

Bits 5-7: CMS (Center-aligned Mode Select)
  0 = Edge-aligned mode (standard)
  Other values = Center-aligned (advanced)

Bit 8: ARPE (Auto-Reload Preload Enable)
  0 = ARR active immediately
  1 = ARR active after update event (safer for PWM)
```

**For your sensor sampling:**
```rust
// Typical setup:
tim2.cr1.modify(|_, w| {
    w.cen().set_bit()      // Enable counter
     .udis().clear_bit()   // Generate update events
     .urs().clear_bit()    // Standard update request
     .dir().clear_bit()    // Count up
     .arpe().clear_bit()   // Auto-reload preload disabled
});
```

---

**TIM_DIER (DMA/Interrupt Enable Register) - Offset: 0x0C**
```
Bit 0: UIE (Update Interrupt Enable)
  0 = Interrupt disabled
  1 = Interrupt enabled on update event

Bit 1: CC1IE (Capture/Compare 1 Interrupt Enable)
  0 = Disabled
  1 = Enabled (for PWM, not used in your case)

Bit 2-4: CC2IE, CC3IE, CC4IE (Capture/Compare 2-4)
  Similar to CC1IE

Bit 8: TIE (Trigger Interrupt Enable)
  For slave mode triggering (advanced)

Bit 0-5: Corresponding DMA Enable bits (similar pattern)
```

**For your sensor sampling:**
```rust
// Enable only update interrupt:
tim2.dier.modify(|_, w| {
    w.uie().set_bit()  // Enable update interrupt
});
```

---

**TIM_SR (Status Register) - Offset: 0x10**
```
Bit 0: UIF (Update Interrupt Flag)
  0 = No update event occurred
  1 = Update event occurred (counter overflowed)
  
  ?? **CRITICAL:** Must write 0 to clear this flag!
  If left SET, the interrupt stays pending and doesn't fire again.

Bit 1-4: CC1IF, CC2IF, CC3IF, CC4IF
  Capture/Compare flags (not used in your case)

Bit 6: TIF (Trigger Interrupt Flag)
  External trigger flag (not used in your case)
```

**In your ISR (MUST DO THIS!):**
```rust
let dp = unsafe { pac::Peripherals::steal() };
let uif = dp.TIM2.sr.read().uif().bit_is_set();
if uif {
    // Clear the interrupt flag
    dp.TIM2.sr.write(|w| w.uif().clear_bit());
}
```

---

**TIM_PSC (Prescaler Register) - Offset: 0x28**
```
Bits [15:0]: PSC (Prescaler value)
  Timer frequency = CPU clock / (PSC + 1)
  
  Example: PSC = 719
  Timer frequency = 72 MHz / 720 = 100 kHz
  Timer tick = 10 �s

Note: PSC is buffered! Change takes effect after next update event.
      Write PSC.UIFR = 1 to force immediate update.
```

---

**TIM_ARR (Auto-Reload Register) - Offset: 0x2C**
```
Bits [31:0]: ARR (Auto-reload value)
  Counter counts from 0 to ARR, then reloads to 0
  
  Overflow period = (ARR + 1) � Timer Tick
  
  Example: ARR = 249, Timer Tick = 10 �s
  Overflow period = 250 � 10 �s = 2.5 ms

Note: ARR is also buffered! Takes effect after update event.
      For immediate effect, write to EGR (Event Generation Register).
```

---

**TIM_CNT (Counter Register) - Offset: 0x24**
```
Bits [31:0]: CNT (Counter value)
  Current value of 32-bit counter
  
  Read-only during normal operation
  Range: 0 to ARR

Useful for debugging to see current counter value:
  uint32_t current = TIM2->CNT;  // Read counter
```

* * *
### Related Registers: NVIC (Chapter 14 of STM32 Cortex-M Datasheet)

**NVIC Interrupt Set-Enable Register (ISER)**
```
Used to unmask (enable) interrupts

Cortex-M4/F4 have up to 74 maskable interrupt channels
TIM2 interrupt line varies by STM32 variant

For STM32F303:
  TIM2 = Position 28 (Interrupt number 28)
IRQ Interrupt Requests Queue ?
```

**Rust Usage:**
```rust
use cortex_m::peripheral::NVIC;
use stm32f3_discovery::stm32f3xx_hal::pac::Interrupt;

// Unmask TIM2
unsafe {
    NVIC::unmask(interrupt::TIM2);
    //let mut nvic = Peripherals::steal().NVIC;
    //nvic.set_priority(pac::Interrupt::TIM2, 100);
}

// Mask TIM2 (disable)
NVIC::mask(Interrupt::TIM2);
```

* * *
* * *

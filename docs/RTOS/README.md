# RTOS & Real-Time Programming Learning Path

## 📚 Documentation Index

This directory contains a comprehensive learning guide for implementing real-time concepts in embedded Rust on your STM32F3 Discovery board.

### Files Overview

| File | Purpose | Read Time |
|------|---------|-----------|
| **00_Bare_Metal_Interrupts_Fundamentals.md** | Explains polling vs interrupt-driven architecture, synchronization with Mutex | 20 min |
| **01_Verification_and_Testing_Strategy.md** | How to verify each phase works; testing checklist and diagnostics | 15 min |
| **02_RTIC_Concepts_and_Architecture.md** | Deep dive into RTIC framework; why it's better than bare-metal | 25 min |
| **03_Learning_Progression_Roadmap.md** | Step-by-step implementation guide (phases 1–4) | 30 min |
| **README.md** | This file; quick reference and navigation | 5 min |

---

## 🚀 Quick Start

### For the Impatient

**TL;DR:** You have 3 phases to complete:

1. **Phase 1 (1–2 hrs):** Add timing measurements to current polling system
2. **Phase 2 (3–4 hrs):** Replace polling with TIM2 interrupt + Mutex
3. **Phase 3 (4–6 hrs):** Migrate to RTIC framework

**Total time to fluency:** ~2 weeks part-time learning

---

## 📖 Reading Order

**Start here:**

1. Read **`00_Bare_Metal_Interrupts_Fundamentals.md`**
   - Understand polling vs interrupts
   - Learn how `Mutex` prevents race conditions
   - See why RTIC is better

2. Read **`02_RTIC_Concepts_and_Architecture.md`**
   - Understand RTIC architecture and benefits
   - See code examples

3. Read **`01_Verification_and_Testing_Strategy.md`**
   - Learn how to test each phase
   - Understand what "working" looks like

4. Read **`03_Learning_Progression_Roadmap.md`**
   - Follow step-by-step implementation
   - Execute each phase

---

## 🎯 Learning Objectives

By the end of this learning path, you will:

✅ **Understand interrupt-driven architecture** — Why it's better than polling  
✅ **Master synchronization primitives** — `Mutex`, `RefCell`, critical sections  
✅ **Implement bare-metal interrupts** — Timer configuration, ISRs, flag clearing  
✅ **Migrate to RTIC framework** — Priority-based scheduling, task spawning  
✅ **Optimize for real-time** — Deterministic timing, low power usage  
✅ **Debug real-time systems** — Use ITM, DWT cycle counter, breakpoints  

---

## 🔄 Architecture Progression

```
Phase 1: Polling
┌─────────────────────────────┐
│ Main Loop                   │
│  ├─ Read Sensor             │
│  ├─ Process Data            │
│  └─ Busy-Wait (NOP loop)    │ ← CPU spinning! 💪 100% usage
└─────────────────────────────┘

              ↓ OPTIMIZE

Phase 2: Bare-Metal Interrupts
┌──────────────────────┐
│ TIM2 (400Hz)         │
│  └─ Interrupt ISR    │
│     └─ Read Sensor   │
│        └─ Store Data │
└──────────────────────┘
         │
         ↓ Fires every 2.5ms
┌──────────────────────┐
│ Main Loop            │
│  ├─ Read Data        │
│  └─ Sleep (WFE)      │ ← CPU sleeping! 💤 5% usage
└──────────────────────┘

              ↓ MODERNIZE

Phase 3: RTIC Framework
┌─────────────────────────────┐
│ RTIC App                    │
│  ├─ Task: sensor_read (p2)  │ ← High priority
│  ├─ Task: log_data (p1)     │ ← Medium priority
│  └─ Idle (p0)               │ ← Lowest priority
│                             │
│ Compiler ensures:           │
│  ✓ No race conditions       │
│  ✓ No priority inversion    │
│  ✓ Safe resource sharing    │
└─────────────────────────────┘
```

---

## 📊 Expected Improvements

### Phase 1 → Phase 2

| Metric | Phase 1 | Phase 2 | Improvement |
|--------|---------|---------|-------------|
| **CPU Usage** | 100% | ~5% | 20x better |
| **Power Consumption** | Max | Low | Huge savings |
| **Response Time** | Varies | <2.5ms (guaranteed) | Deterministic |
| **Complexity** | Simple | Medium | Worth it |

### Phase 2 → Phase 3

| Metric | Phase 2 | Phase 3 | Benefit |
|--------|---------|---------|---------|
| **CPU Usage** | ~5% | ~5% | Same |
| **Power** | Low | Low | Same |
| **Safety** | Manual | Compiler-checked | Much safer |
| **Scalability** | Difficult | Easy | Can add tasks freely |
| **Debugging** | Hard | Medium | Better error messages |

---

## 🛠️ Prerequisites

Before starting, ensure you have:

✅ Rust embedded toolchain:
```bash
rustup target add thumbv7em-none-eabihf
cargo install cargo-embed
```

✅ STM32F3 Discovery board connected

✅ ST-Link V2 debugger connected

✅ OpenOCD or cargo-embed configured

✅ ITM output viewer (in your IDE)

---

## 🔧 Hardware Requirements

| Component | Requirement |
|-----------|-------------|
| **Microcontroller** | STM32F303VC (STM32F3 Discovery) |
| **RAM** | 40 KB (sufficient for RTIC + multiple tasks) |
| **Clock** | 72 MHz (default config) |
| **Timers** | TIM2 (32-bit, for 400Hz interrupt) |
| **Peripherals** | SPI1, GPIO for CS, ITM for debugging |

---

## ⚠️ Common Pitfalls

### Phase 1 (Baseline)
- ❌ Forgot to enable DWT cycle counter → can't measure timing
- ❌ Didn't verify sensor reads are consistent → hidden bugs
- **Fix:** Follow **01_Verification_and_Testing_Strategy.md**

### Phase 2 (Interrupts)
- ❌ Forgot to clear interrupt flag → timer doesn't fire again
- ❌ Didn't use `Mutex` → race conditions/corruption
- ❌ Wrong PSC/ARR values → timer fires at wrong frequency
- **Fix:** Follow **03_Learning_Progression_Roadmap.md** step-by-step

### Phase 3 (RTIC)
- ❌ Forgot to declare resources in `#[task(...)]` → compilation error
- ❌ Wrong priority values → compiler error (all tasks need explicit priority)
- ❌ Tried to access non-existent resources → compilation error
- **Fix:** RTIC is **strict by design** — errors catch bugs early! ✓

---

## 📝 Implementation Checklist

### Phase 1 Setup
- [ ] Read: `00_Bare_Metal_Interrupts_Fundamentals.md` (Part 1–2)
- [ ] Read: `01_Verification_and_Testing_Strategy.md` (Phase 1 section)
- [ ] Implement: Cycle counter in `main.rs`
- [ ] Verify: Timing accuracy ±10%
- [ ] Verify: No data corruption
- [ ] Document: Baseline measurements

### Phase 2 Setup
- [ ] Read: `00_Bare_Metal_Interrupts_Fundamentals.md` (Part 3–6)
- [ ] Read: `01_Verification_and_Testing_Strategy.md` (Phase 2 section)
- [ ] Read: `03_Learning_Progression_Roadmap.md` (Phase 2 section)
- [ ] Create: `auxiliary/src/interrupt_handler.rs`
- [ ] Implement: TIM2 configuration
- [ ] Implement: TIM2 interrupt handler
- [ ] Implement: `Mutex<RefCell<>>` for shared data
- [ ] Verify: 400±1 Hz interrupt rate
- [ ] Verify: No data corruption
- [ ] Measure: Power reduction (900x cycles improvement)

### Phase 3 Setup
- [ ] Read: `02_RTIC_Concepts_and_Architecture.md`
- [ ] Read: `03_Learning_Progression_Roadmap.md` (Phase 3 section)
- [ ] Add: RTIC dependency in `Cargo.toml`
- [ ] Implement: RTIC app structure
- [ ] Implement: Sensor read task
- [ ] Implement: Logging task
- [ ] Verify: Tasks execute with correct priorities
- [ ] Verify: Power consumption maintained

---

## 🧪 Testing Framework

### Manual Testing (ITM Output)

```rust
// Check output format
iprintln!(&mut itm.stim[0], "Interrupt Freq: {:.1} Hz", freq);

// Capture output
grep "Interrupt Freq" openocd.log

// Analyze
awk '{print $NF}' < frequencies.txt | sort -n
```

### Automated Checks

```bash
# Build all phases
for phase in phase1 phase2 phase3; do
    cargo build --release --features=$phase
done

# Run any embedded tests
cargo test --lib
```

---

## 🔗 External References

- **[RTIC Official Documentation](https://rtic.rs/)**  
- **[Cortex-M Runtime](https://docs.rs/cortex-m-rt/)**  
- **[Embedded Rust Book](https://rust-embedded.github.io/book/)**  
- **[STM32F3 Reference Manual](https://www.st.com/resource/en/reference_manual/dm00043574.pdf)**  
- **[ARM Cortex-M Guide](https://developer.arm.com/documentation/dui0553/)**

---

## 📞 Support and Debugging

### If You Get Stuck

1. **Check diagnostics table** in `01_Verification_and_Testing_Strategy.md`
2. **Review code examples** in `03_Learning_Progression_Roadmap.md`
3. **Verify hardware setup** (pinouts, clock config)
4. **Use GDB breakpoints** to trace execution
5. **Inspect ITM output** for clues

### Common Questions

**Q: How long does each phase take?**  
A: Phase 1 (~2h), Phase 2 (~4h), Phase 3 (~5h). Total ~2 weeks part-time.

**Q: Do I need RTIC, or can I stick with bare-metal interrupts?**  
A: For a single-task system, bare-metal is fine. For multiple tasks, RTIC is **essential** for safety.

**Q: Will my gyroscope driver work without changes?**  
A: Yes! The driver is **device-agnostic**. Only the calling code changes.

**Q: Can I run all 3 phases on the same board?**  
A: Yes! Each phase overwrites the previous in the same binary. No destructive changes.

---

## 📜 License

These learning materials are part of your embedded Rust education journey.

---

**Ready to start?** Begin with **`00_Bare_Metal_Interrupts_Fundamentals.md`** → then proceed to Phase 1 in **`03_Learning_Progression_Roadmap.md`**. 🚀


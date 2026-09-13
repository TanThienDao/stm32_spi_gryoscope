# Implementation Explanation Directory

## Overview

This directory contains **detailed, step-by-step implementation guides** with complete code examples and explanations. These are meant to be read and re-read while you code.

**Unlike the conceptual docs in `RTOS/`, these guides focus on:**
- ✅ Exact code to write
- ✅ Line-by-line explanations
- ✅ Expected output examples
- ✅ Troubleshooting for common issues
- ✅ Validation checklists

---

## Files

| File | Duration | Focus | Difficulty |
|------|----------|-------|------------|
| **01_Phase1_Implementation_Guide.md** | 1-2 hours | Baseline measurement, cycle counter, consistency checks | Beginner |
| **02_Phase2_Implementation_Guide.md** | 3-4 hours | Timer interrupts, Mutex, ISR handler | Intermediate |
| **03_Phase3_Implementation_Guide.md** | 4-6 hours | RTIC app migration, task priorities | Advanced |

---

## How to Use This Directory

### Reading Strategy

1. **Read the guide fully first** — Understand what you're implementing
2. **Open your code editor** — Keep the guide in one window, code in another
3. **Copy code sections** — Follow the "File: X" headers
4. **Build and test** — Verify output matches expected results
5. **Troubleshoot** — Use the troubleshooting tables

### Example Workflow

```
Terminal 1: Editor                Terminal 2: Building
┌─────────────────────┐          ┌──────────────────┐
│ 01_Phase1_...md     │          │ cargo build --r  │
│ (reading)           │          │ cargo embed --r  │
└─────────────────────┘          └──────────────────┘

Terminal 3: Monitoring
┌──────────────────────┐
│ ITM output viewer    │
│ (watching output)    │
└──────────────────────┘
```

---

## Quick Navigation

### If You're Starting Phase 1
👉 **Read:** `01_Phase1_Implementation_Guide.md`
- Step 1.1: Enable cycle counter (20 min)
- Step 1.2: Add consistency checks (20 min)
- Step 1.3: Measure power baseline (20 min)

### If You're Starting Phase 2
👉 **Read:** `02_Phase2_Implementation_Guide.md` (when available)
- Step 2.1: Configure TIM2 timer
- Step 2.2: Write interrupt handler
- Step 2.3: Add shared data with Mutex
- Step 2.4: Verify interrupt firing

### If You're Starting Phase 3
👉 **Read:** `03_Phase3_Implementation_Guide.md` (when available)
- Step 3.1: Add RTIC dependencies
- Step 3.2: Create RTIC app structure
- Step 3.3: Define tasks and resources
- Step 3.4: Task priorities and spawning

---

## Code Organization

Each implementation guide has this structure:

```
## StepX.Y: Feature Name

### What It Does
Brief explanation of the feature

### Why It Matters
Context and benefits

### Implementation

#### File 1: path/to/file.rs
[Code block with complete code]

#### File 2: path/to/file.rs
[Code block with complete code]

### Code Breakdown
Line-by-line explanation of key parts

### Expected Output
Example of what you'll see

### ✅ Validation
Checklist of pass criteria
```

---

## Copy-Paste Strategy

Each code block is **complete and ready to paste**. 

✅ **Good approach:**
1. Read the "Code Breakdown" section first
2. Copy the entire code block
3. Paste into your file
4. Verify it compiles

❌ **Avoid:**
- Mixing code from multiple steps
- Partially copying code
- Trying to understand before copying (read first!)

---

## Expected Output Examples

Throughout these guides, "Expected Output" sections show exactly what you should see:

```
✅ GOOD - Output matches expected format
X:    10.45°/s | Y:    -2.15°/s | Z:     5.32°/s | 2.501ms
X:    10.48°/s | Y:    -2.13°/s | Z:     5.34°/s | 2.502ms

❌ BAD - Output shows errors/anomalies
⚠️  ANOMALY #1: dx=250.45, dy=500.99, dz=125.67
X:   260.90°/s | Y:   498.84°/s | Z:   131.00°/s
```

If your output doesn't match expected format, refer to the **Troubleshooting** section.

---

## Validation Checklists

Each step has a validation checklist (✅ boxes). Use these to confirm you're done:

```
✅ Step 1.1 Validation

Pass Criteria:
- [ ] Code compiles without errors
- [ ] ITM output appears and is readable
- [ ] Timing values shown (e.g., "Elapsed: 2.501ms")
- [ ] All timings between 2.25–2.75ms
- [ ] Output appears every ~100ms
```

**Don't move to next step until all boxes are checked!**

---

## Phase 1 (Baseline Measurement)

### Duration: 1-2 hours

**Goal:** Establish baseline measurements of your current polling system

**What you'll learn:**
- How to use DWT cycle counter
- How to measure timing accuracy
- How to detect data corruption
- How to calculate power consumption

**Key metrics:**
- Timing: ~2.5ms per read (±10%)
- Anomalies: 0 detected over 30 seconds
- Power: ~900k cycles/loop = 100% CPU

**Next:** Phase 2 (Interrupts)

---

## Phase 2 (Bare-Metal Interrupts)

### Duration: 3-4 hours

**Goal:** Replace polling with timer interrupt, maintain same functionality

**What you'll learn:**
- How to configure STM32F3 TIM2
- How to write interrupt handlers
- How to use Mutex for thread-safe shared data
- How to enable/disable interrupts

**Key metrics:**
- Interrupt rate: 400±1 Hz
- Power: ~1k cycles/loop (900x improvement!)

**Prerequisites:** Phase 1 complete

**Next:** Phase 3 (RTIC)

---

## Phase 3 (RTIC Framework)

### Duration: 4-6 hours

**Goal:** Migrate from bare-metal to RTIC for safety and scalability

**What you'll learn:**
- How to structure RTIC apps
- How to define tasks and priorities
- How to share resources safely
- How to spawn and reschedule tasks

**Key metrics:**
- Compiler-guaranteed safety
- Zero-cost abstractions
- Multiple concurrent tasks possible

**Prerequisites:** Phase 2 complete

---

## Common Patterns Used

### Code Blocks

Throughout these guides, code is organized in labeled blocks:

```rust
// ===== PHASE 1.1: ENABLE CYCLE COUNTER =====

let cp = cortex_m::Peripherals::take().unwrap();
let mut dwt = cp.DWT;
dwt.enable_cycle_counter();

// ===== END SETUP =====
```

**Header tells you:**
- Which phase/step the code belongs to
- What it's doing

**Use this to:**
- Find specific sections easily
- Understand code flow
- Replace sections in existing code

### Comments

Three types of comments are used:

```rust
// Single line explanation
// Quick notes about what follows

// ===== MAJOR SECTION =====
// Code blocks are organized this way

/// Documentation comment
/// Used for complex concepts
```

---

## Troubleshooting Quick Reference

| Issue | Likely Cause | Solution |
|-------|--------------|----------|
| "Compilation error: DWT not found" | Not imported | Check `lib.rs` - should already have it |
| "Code compiles but no ITM output" | Debugger not connected | Check OpenOCD/IDE connection |
| "Timing shows 'Nan'" | Sensor not responding | Verify SPI wiring and CS pin |
| "Anomalies every loop" | Data corruption | Check clock speed and SPI mode |
| "Cycles way too high" | Expected for Phase 1 | Use busy-wait delays are normal |

**For detailed troubleshooting:** See the **Troubleshooting** section in each guide.

---

## Testing Workflow

### Standard Workflow for Each Step

```bash
# 1. Edit code according to guide
vim src/main.rs

# 2. Build
cargo build --release

# 3. Flash to device
cargo embed --release

# 4. Monitor ITM output (keep running)
# Watch the terminal output...

# 5. Verify against expected output
# Check timing, anomalies, stats

# 6. If not matching, check troubleshooting section
```

---

## Tips for Success

### ✅ DO:
- Read the "What It Does" section before coding
- Copy entire code blocks (don't cherry-pick lines)
- Follow validation checklists
- Document your baseline measurements
- Test each step before moving to next

### ❌ DON'T:
- Skip the code breakdown explanations
- Mix code from different steps
- Move to next phase before passing validation
- Ignore troubleshooting errors
- Assume output is "good enough" if it doesn't match examples

---

## File Structure

```
docs/explain_implement/
├── README.md                          # This file
├── 01_Phase1_Implementation_Guide.md  # Baseline (1-2 hrs)
├── 02_Phase2_Implementation_Guide.md  # Interrupts (3-4 hrs)
└── 03_Phase3_Implementation_Guide.md  # RTIC (4-6 hrs)
```

---

## Relationship to Other Docs

| Directory | Purpose | When to Use |
|-----------|---------|------------|
| `RTOS/` | **Concepts** — Understand theory | Before implementing phases |
| `explain_implement/` | **Step-by-step code** — How to implement | While actively coding |
| Root `docs/` | **Reference** — Design decisions | For context/background |

**Recommended reading order:**
1. `RTOS/README.md` — Understand goals
2. `RTOS/00_Bare_Metal_Interrupts_Fundamentals.md` — Learn concepts
3. `explain_implement/01_Phase1_Implementation_Guide.md` — Start coding

---

## Getting Help

If something doesn't work:

1. **Check the validation checklist** — Did you complete all steps?
2. **Review expected output** — Does your output match?
3. **Consult troubleshooting** — Is your issue listed?
4. **Re-read code breakdown** — Did you understand the logic?
5. **Check ITM output** — Are there any error messages?

---

## Progress Tracking

Use this to track your learning:

```
Phase 1: Baseline Measurement
- [ ] Step 1.1: Cycle counter enabled
- [ ] Step 1.2: Consistency check implemented
- [ ] Step 1.3: Power baseline measured
- [ ] Validation: All checks passed
- [ ] Documentation: Baseline values recorded

Phase 2: Bare-Metal Interrupts
- [ ] Step 2.1: TIM2 configured
- [ ] Step 2.2: ISR handler written
- [ ] Step 2.3: Mutex shared data
- [ ] Step 2.4: Interrupt verified firing
- [ ] Validation: All checks passed
- [ ] Comparison: 900x power improvement verified

Phase 3: RTIC Migration
- [ ] Step 3.1: RTIC dependency added
- [ ] Step 3.2: App structure created
- [ ] Step 3.3: Tasks and resources defined
- [ ] Step 3.4: Multiple tasks running
- [ ] Validation: All checks passed
- [ ] Documentation: Lessons learned
```

---

## Next Steps

1. **Start with Phase 1:** `01_Phase1_Implementation_Guide.md`
2. **Follow each step** in order (1.1 → 1.2 → 1.3)
3. **Complete validation** before moving to Phase 2
4. **Document your results**
5. **Move to Phase 2** when Phase 1 is fully working

---

**Ready to start coding?** Open `01_Phase1_Implementation_Guide.md` and begin Step 1.1! 🚀


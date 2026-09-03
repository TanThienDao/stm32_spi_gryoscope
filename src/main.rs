//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use auxiliary::*;
use cortex_m_rt::entry;
use cortex_m::peripheral::DWT;
use auxiliary::interrupt_handler::NEW_DATA_READY;

// Note: Phase 2 uses timer interrupt flag polling instead of ISR
// This provides efficient timer-based updates without macro complications

#[entry]
fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs, mut dwt) = init();

    // Enable DWT cycle counter for measurements
    // DWT (Data Watchpoint and Trace) is a feature of ARM Cortex-M
    // processors that provides a cycle counter,
    // which can be used for profiling and measuring execution time.
    // Enabling the cycle counter allows you to measure how many clock
    // cycles have elapsed between two points in your code,
    // which is useful for performance analysis and debugging.
    dwt.enable_cycle_counter();

    iprintln!(&mut itm.stim[0], "===============================");
    iprintln!(&mut itm.stim[0], "I3G4250D Gyroscope");
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

    if variant != GyroVariant::I3g4250d {
        iprintln!(&mut itm.stim[0], "✗ This requires I3G4250D!");
        loop {}
    }
    // Step 2: initialize custom driver
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 2: Initializing driver...");
    let mut gyro = GyroDriver::new(spi, cs);

    match gyro.who_am_i() {
        Ok(id) => {
            if id == 0xD3 {
                iprintln!(&mut itm.stim[0], "✓ WHO_AM_I: 0x{:02X}", id);
            } else {
                iprintln!(&mut itm.stim[0], "✗ Unexpected ID: 0x{:02X}", id);
                loop {}
            }
        }
        Err(e) => {
            iprintln!(&mut itm.stim[0], "✗ WHO_AM_I failed: {}", e);
            loop {}
        }
    }

    // Initialize the gyroscope
    if let Err(e) = gyro.init() {
        iprintln!(&mut itm.stim[0], "✗ Initialization error: {}", e);
        loop {}
    }

    // Step 3: Configure gyroscope
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 3: Configuring sensor...");

    if let Err(e) = gyro.set_data_rate(DataRate::Hz400) {
        iprintln!(&mut itm.stim[0], "✗ DataRate config error: {}", e);
        loop {}
    }

    if let Err(e) = gyro.set_range(Range::DPS500) {
        iprintln!(&mut itm.stim[0], "✗ Range config error: {}", e);
        loop {}
    }

    iprintln!(&mut itm.stim[0], "✓ Configuration complete:");
    iprintln!(&mut itm.stim[0], "  - Data Rate: 400 Hz (timer interrupt)");
    iprintln!(&mut itm.stim[0], "  - Range: 500 °/s");
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 4: Starting interrupt-driven mode...");
    iprintln!(&mut itm.stim[0], "──────────────────────────────────────");
    iprintln!(&mut itm.stim[0], "");

    // ===== PHASE 2: MAIN LOOP =====

    // Enable TIM2 Interrupt safely
    let mut nvic_guard = NvicGuard::new();
    if let Err(e) = nvic_guard.unmask_tim2_safe() {
        iprintln!(&mut itm.stim[0], "✗ NVIC unmask error: {}", e);
        loop {}
    }

    let mut counter = 0u32;
    let mut prev_reading = (0.0f32, 0.0f32, 0.0f32);
    let mut anomaly_count = 0u32;
    const MAX_DELTA: f32 = 100.0;

    let mut total_cycles = 0u64;
    let mut loop_iterations = 0u32;
    let mut main_loop_start = DWT::cycle_count();

    loop {
        let should_sleep = cortex_m::interrupt::free(|cs| {

            let mut ready = NEW_DATA_READY.borrow(cs).borrow_mut();
            if *ready {
                *ready = false;  // Reset flag
                // Read sensor data from gyro
                match gyro.read_angular_velocity() {
                    Ok((x, y, z)) => {
                        // Consistency check
                        let delta_x = (x - prev_reading.0).abs();
                        let delta_y = (y - prev_reading.1).abs();
                        let delta_z = (z - prev_reading.2).abs();

                        // Flag if exceeds threshold
                        if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
                            anomaly_count += 1;
                            iprintln!(
                            &mut itm.stim[0],
                            "⚠️  ANOMALY #{}: Δx={:.2}, Δy={:.2}, Δz={:.2}",
                            anomaly_count,
                            delta_x,
                            delta_y,
                            delta_z
                        );
                        }

                        // Update previous reading
                        prev_reading = (x, y, z);

                        if counter % 4 == 0 {
                            iprintln!(
                            &mut itm.stim[0],
                            "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s",
                            x,
                            y,
                            z
                        );
                        }
                        counter += 1;

                        // Measure loop performance
                        let now = DWT::cycle_count();
                        let loop_cycles = now.wrapping_sub(main_loop_start);
                        total_cycles = total_cycles.wrapping_add(loop_cycles as u64);
                        loop_iterations += 1;
                        main_loop_start = now;

                        // Print statistics every 1000 iterations (~2.5s at 400 Hz)
                        if loop_iterations % 1000 == 0 {
                            let avg_cycles = (total_cycles / 1000) as u32;

                            // Convert cycles to microseconds (72 MHz = 72 cycles per μs)
                            let loop_time_us = avg_cycles as f64 / 72.0;

                            // Total measurement period: 1000 iterations at 400 Hz
                            // = 1000 / 400 Hz = 2.5 seconds = 2,500,000 microseconds
                            let total_period_us = 2_500_000.0;

                            // CPU Usage = (Time spent executing / Total measurement time) × 100%
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
                    }
                    Err(e) => {
                        iprintln!(&mut itm.stim[0], "✗ Read error: {}", e);
                        loop {}
                    }
                }
                false // Don't sleep if we processed data
            }
            else {
                //iprintln!(&mut itm.stim[0], "Sleeping until next timer interrupt...");
                true // Signal to sleep when no data ready
            }
        });

        if should_sleep {
            // If no data ready, just loop back (low CPU usage when sleeping)
            //iprintln!(&mut itm.stim[0], "Waiting for timer interrupt...");
            cortex_m::asm::wfe();
        }

    }
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn TIM2() {
    use auxiliary::interrupt_handler::NEW_DATA_READY;
    use cortex_m::interrupt;

    // Clear flag safely through helper
    let _ = auxiliary::Tim2Guard::check_and_clear_uif();

    interrupt::free(|cs| {
        *NEW_DATA_READY.borrow(cs).borrow_mut() = true;
    });
}

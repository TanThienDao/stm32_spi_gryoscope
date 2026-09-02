#![deny(unsafe_code)]
#![no_main]
#![no_std]

use auxiliary::*;
use core::cell::RefCell;
use cortex_m_rt::entry;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::DWT;

// Note: TIM2_PERIPHERAL and NEW_DATA_AVAILABLE are commented out for polling mode
// Uncomment when implementing interrupt-driven mode
// static mut TIM2_PERIPHERAL: Option<TIM2> = None;
// static NEW_DATA_AVAILABLE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

// Note: TIM2_PERIPHERAL and NEW_DATA_AVAILABLE are commented out for polling mode
// Uncomment when implementing interrupt-driven mode
// static mut TIM2_PERIPHERAL: Option<TIM2> = None;
// static NEW_DATA_AVAILABLE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

// Note: SENSOR_DATA is kept for future interrupt-based implementation
// Currently using polling mode - data is read directly in the main loop
#[allow(dead_code)]
static SENSOR_DATA: Mutex<RefCell<(f32, f32, f32)>> = Mutex::new(RefCell::new((0.0, 0.0, 0.0)));
// ==== End of Shared Data Buffer ====

// ==== Interrupt Handler for TIM2 ====
// Note: Interrupt support requires device-specific configuration.
// For now, gyroscope reading is done via polling in the main loop.
// Uncomment and configure when implementing RTOS-based interrupt handling.
//
// #[interrupt]
// fn TIM2() {
//     //  Enter critical section to safely access shared resources
//     // Interrupt is disabled here, so we can safely access shared resources without race conditions
//     // Interrupts are automatically re-enabled when we exit the critical section
//     free(|cs| {
//         //Step 1: Clear the interrupt flag to acknowledge the interrupt
//         // CRITICAL - Clear interrupt flag
//         // without this, the timer is blocked and won't fire again! \O_O/
//         unsafe {
//             // Clear the TIM2 interrupt flag
//             if let Some(tim2) = TIM2_PERIPHERAL.as_mut() {
//                 // Clear the update interrupt flag (UIF) to acknowledge the interrupt
//                 tim2.sr.modify(|_, w| w.uif().clear_bit());
//             }
//         }
//
//         // Step 2: Read data from shared buffer if needed
//         // Note: GYRO_DRIVER_MUTEX access removed for simplicity
//         // Reading is done in the main thread only
//     });
// }

#[entry]
fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs, mut dwt) = init();

    iprintln!(&mut itm.stim[0], "===============================");
    iprintln!(&mut itm.stim[0], "I3G4250D Gyroscope Demo");
    iprintln!(&mut itm.stim[0], "===============================");

    // Enable DWT cycle counter for precise timing
    // DWT (Data Watchpoint and Trace) is a feature of ARM Cortex-M
    // processors that provides a cycle counter,
    // which can be used for profiling and measuring execution time.
    // Enabling the cycle counter allows you to measure how many clock
    // cycles have elapsed between two points in your code,
    // which is useful for performance analysis and debugging.
    dwt.enable_cycle_counter();

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

    // Step 2: Verify it's I3G4250D and initialize custom driver
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 2: Initializing custom driver...");

    if variant != GyroVariant::I3g4250d {
        iprintln!(&mut itm.stim[0], "✗ This demo is for I3G4250D only!");
        loop {}
    }

    let mut gyro = GyroDriver::new(spi, cs);

    // Verify WHO_AM_I
    match gyro.who_am_i() {
        Ok(id) => {
            if id == 0xD3 {
                iprintln!(&mut itm.stim[0], "✓ WHO_AM_I confirmed: 0x{:02X}", id);
            } else {
                iprintln!(
                    &mut itm.stim[0],
                    "✗ Unexpected WHO_AM_I value: 0x{:02X}",
                    id
                );
                loop {}
            }
        }
        Err(e) => {
            iprintln!(&mut itm.stim[0], "✗ WHO_AM_I read failed: {}", e);
            loop {}
        }
    }

    // Step 3: Configure gyroscope
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 3: Configuring gyroscope...");

    if let Err(e) = gyro.init() {
        iprintln!(&mut itm.stim[0], "✗ Initialization error: {}", e);
        loop {}
    }

    if let Err(e) = gyro.set_data_rate(DataRate::Hz100) {
        iprintln!(&mut itm.stim[0], "✗ DataRate config error: {}", e);
        loop {}
    }

    if let Err(e) = gyro.set_range(Range::DPS245) {
        iprintln!(&mut itm.stim[0], "✗ Range config error: {}", e);
        loop {}
    }

    iprintln!(&mut itm.stim[0], "✓ Configuration complete:");
    iprintln!(&mut itm.stim[0], "  - Data Rate: 100 Hz");
    iprintln!(&mut itm.stim[0], "  - Range: 245 °/s");
    iprintln!(&mut itm.stim[0], "  - All axes enabled");

    // Step 4: Start reading sensor data
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 4: Starting sensor readings...");
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Angular Velocity (°/s):");
    iprintln!(&mut itm.stim[0], "───────────────────────");

    let mut counter: u32 = 0;
    let mut prev_reading: (f32, f32, f32) = (0.0, 0.0, 0.0);
    let mut abnomaly_counter: u32 = 0;

    // Maximum allowed change per 0.25 ms sample
    // Allow up to 100 °/s change per 0.25 ms sample (400 Hz ODR)
    const MAX_DELTA: f32 = 100.0;

    // ==== Power Mesurement variables ====
    let mut total_cycles: u64 = 0;
    let mut loop_iterations: u32 = 0;

    // ==== Main Loop: Read Gyroscope Data ====
    loop {
        // Record start time (in CPU cycles) for timing measurement
        let start_cycles = DWT::cycle_count();
        //iprintln!(&mut itm.stim[0], "Start Cycles: {}", start_cycles);
        match gyro.read_angular_velocity() {
            Ok((x, y, z)) => {
                // Calculate elapsed cycles
                let elapsed_cycles = DWT::cycle_count().wrapping_sub(start_cycles);
                //iprintln!(&mut itm.stim[0], "Elapsed Cycles: {}", elapsed_cycles);

                // Convert cycles to millisecounds
                // CPU clock is 72 MHz, so 1 cycle = 1/72,000,000 seconds
                let elapsed_ms = (elapsed_cycles as f64) / 72_000.0;
                //Calculation Check:
                //    Elapsed cycles: 15,440
                //    CPU clock: 72 MHz (72,000 cycles per ms)
                //    Time: 15,440 ÷ 72,000 = 0.214 ms ✓

                //======= Check Consistency =======
                // Calculate the change in readings from the previous reading
                let delta_x = (x - prev_reading.0).abs();
                let delta_y = (y - prev_reading.1).abs();
                let delta_z = (z - prev_reading.2).abs();

                // Flag if any change exceeds the threshold
                if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
                    abnomaly_counter += 1;
                    iprintln!(
                        &mut itm.stim[0],
                        "⚠️ Abnormal reading detected! ΔX: {:.2}, ΔY: {:.2}, ΔZ: {:.2} | Count: {}",
                        delta_x,
                        delta_y,
                        delta_z,
                        abnomaly_counter
                    );
                }

                // Update previous reading for next iteration
                prev_reading = (x, y, z);

                // Print every 4th reading (~100ms at 400Hz, every ~10.5ms per read)
                if counter % 4 == 0 {
                    iprintln!(
                        &mut itm.stim[0],
                        "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s | Elapsed: {:.6} ms",
                        x,
                        y,
                        z,
                        elapsed_ms
                    );
                }
                counter += 1;
            }
            Err(e) => {
                iprintln!(&mut itm.stim[0], "✗ Read error: {}", e);
                loop {}
            }
        }

        // Small delay between reads (approximate timing)
        // At 380Hz ODR, new data available every ~2.6ms
        // Adding extra delay for readability of ITM output
        for _ in 0..10_000 {
            cortex_m::asm::nop();
        }

        // ===== Calculate Loop time and Power Measurement =====
        let loop_end_cycles = DWT::cycle_count().wrapping_sub(start_cycles);
        total_cycles = total_cycles.wrapping_add(loop_end_cycles as u64);
        loop_iterations += 1;

        // Print statistics every 1000 iteration
        if loop_iterations % 1000 == 0 {
            let avg_cycles = total_cycles / 1000 as u64;
            let avg_ms = (avg_cycles as f64) / 72_000.0;
            iprintln!(&mut itm.stim[0], "------------------------------");
            iprintln!(
                &mut itm.stim[0],
                "📊 Baseline Stats (every 1000 loops / ~2.5s):"
            );
            iprintln!(
                &mut itm.stim[0],
                "| Average Loop Time: {:.6} ms |\n\
                | Average Cycles/Loop: {}    |\n\
                | Loops {}                      |\n",
                avg_ms,
                avg_cycles,
                loop_iterations
            );
            iprintln!(&mut itm.stim[0], "------------------------------");
            total_cycles = 0;
        }
    }
}

#![no_main]
#![no_std]

mod logging;

use rtic::app;
// ===== RTIC APP  =====

/// Shared resources (protected by RTIC)
#[app(device = stm32f3_discovery::stm32f3xx_hal::pac, dispatchers = [EXTI0])]
mod app {
    use crate::{log_info, log_init, log_stats};
    use crate::log_warn;
    use auxiliary::*;
    use rtic::Mutex;
    // ===== SHARED RESOURCES =====
    /// Resources shared between tasks (protected by RTIC)
    #[shared]
    struct Shared {
        /// ITM for logging (protected by RTIC)
        itm: ITM,
        /// Current task name for logging context
        current_task: &'static str,
        /// Shared sensor data (x, y, z angular velocities)
        sensor_data: (f32, f32, f32),
        /// Flag indicating new sensor data is ready
        new_data_ready: bool,
        /// Performane tracking for read_sensor task
        read_sensor_total_cycles: u32,
        read_sensor_count: u32,
        /// Performane tracking for process_data task
        process_data_total_cycles: u32,
        process_data_count: u32,
    }
    /// Pre-task local resources (not shared)
    #[local]
    struct Local {
        /// Local gyro driver instance
        gyro: GyroDriver<
            Spi<
                SPI1,
                (
                    PA5<Alternate<PushPull, 5>>,
                    PA6<Alternate<PushPull, 5>>,
                    PA7<Alternate<PushPull, 5>>,
                ),
            >,
            PE3<Output<PushPull>>,
        >,
        /// Data consistent tracker
        prev_data: (f32, f32, f32),
        /// Counter for tracking anomalies
        counter: u32,
        anomaly_counter: u32,
    }
    // ===== RTIC APP =====
    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        //Step 0: Enable TIM2 clock in RCC (APB1ENR)
        ctx.device.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());
        //init TIM2 for interrupt-driven updates
        init_time2(&mut ctx.device).unwrap();

        //Set up clock
        let mut dwt = ctx.core.DWT;
        let mut flash = ctx.device.FLASH.constrain();
        let mut rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        //==============================================================
        // SPI1 Configuration Start
        //==============================================================

        let mut gpioa = ctx.device.GPIOA.split(&mut rcc.ahb);
        let mut gpioe = ctx.device.GPIOE.split(&mut rcc.ahb);
        // SPI pins (PA5=SCK, PA6=MISO, PA7=MOSI)
        let sck =
            gpioa
                .pa5
                .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let miso =
            gpioa
                .pa6
                .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let mosi =
            gpioa
                .pa7
                .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

        // PE3 CS pin (in STM32F3, this is the NSS Slave Select pin)
        let cs = gpioe
            .pe3
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

        // SPI mode required by I3G4250D
        // There are four SPI modes, defined by the combination of
        // Clock Polarity (CPOL) and Clock Phase (CPHA):
        //
        // Condition: CPOL=1, CPHA=1 (Mode 3)
        // Configuration:
        //   Clock Polarity = Idle High, Active Low.
        //   Clock Phase = Capture on Second Transition, trailing edge of the clock pulse.
        let mode = Mode {
            polarity: Polarity::IdleHigh, // Clock idle state is high (CPOL=1)
            phase: Phase::CaptureOnSecondTransition, // Capture on second clock transition (falling edge for CPOL=1)
        };
        let spi = Spi::spi1(
            ctx.device.SPI1,
            (sck, miso, mosi),
            mode,
            Hertz(1_000_000), // 1 MHz Config clock baud rate, can be adjusted based on the gyroscope's datasheet
            clocks,
            &mut rcc.apb2,
        );
        //==============================================================
        // SPI1 Configuration End
        //==============================================================

        let delay = Delay::new(ctx.core.SYST, clocks);
        let mut itm = ctx.core.ITM;
        log_init!(itm, "==============================================================");
        log_init!(itm, "Phase 3: RTIC-based I3G4250D Gyroscope");
        log_init!(itm, "==============================================================");

        // Step 1: Detect Gyro driver
        log_init!(itm, "Step 1: Detect Gyro driver...");
        let mut gryo = GyroDriver::new(spi, cs);
        log_init!(itm, "✓ Gyro driver initialized.");
        match gryo.who_am_i() {
            Ok(id) => {
                if id == 0xD3 {
                    log_init!(itm, "✓ Gyro ID: 0x{:X} (I3G4250D)", id);
                } else {
                    log_init!(itm, "✗ Unexpected Gyro ID: 0x{:X}", id);
                    loop {}
                }
            }
            Err(_) => {
                log_init!(itm, "✗ Error reading Gyro ID!");
                loop {}
            }
        }

        // Step 2: Initialize Gyro driver
        log_init!(itm, "Step 2: Init Gyro driver...");
        // Initialize the gyro
        if let Err(e) = gryo.init() {
            log_init!(itm, "✗ Error initializing Gyro: {:?}", e);
            loop {}
        }
        log_init!(itm, "✓ Gyro driver initialized.");

        // Step 3: Configure Gyro
        log_init!(itm, "Step 3: Configuring Gyro...");
        if let Err(e) = gryo.set_data_rate(DataRate::Hz400) {
            log_init!(itm, "✗ Error setting data rate: {:?}", e);
            loop {}
        }
        if let Err(e) = gryo.set_range(Range::DPS500) {
            log_init!(itm, "✗ Error setting range: {:?}", e);
            loop {}
        }
        log_init!(itm, "✓ Gyro configured).");
        log_init!(itm, "✓ Configuration complete:");
        log_init!(itm, "  - Data Rate: 400 Hz (timer interrupt)");
        log_init!(itm, "  - Range: 500 °/s");
        log_init!(itm, "");

        log_init!(
            itm,
            "Enabling DWT cycle counter for performance measurement..."
        );
        dwt.enable_cycle_counter();

        log_init!(itm, "✓ RTIC system ready. Starting main loop...");
        log_init!(itm, "==============================================================");
        // shared resources initialization
        let shared = Shared {
            sensor_data: (0.0, 0.0, 0.0),
            new_data_ready: false,
            read_sensor_total_cycles: 0,
            read_sensor_count: 0,
            process_data_total_cycles: 0,
            process_data_count: 0,
            current_task: "Init",
            itm,
        };

        // Initialize local resources
        let local = Local {
            gyro: gryo,
            prev_data: (0.0, 0.0, 0.0),
            counter: 0,
            anomaly_counter: 0,
        };

        (shared, local)
    }

    // ===== IDLE TASK (LOW PRIORITY) =====
    /// Idle loop (lowest priority) for processing data
    /// Spawned process_data task when interrupt fires
    #[idle(shared = [itm])]
    fn idle(mut cx: idle::Context) -> ! {
        //Main loop: Process data (lower priority)
        //sleeps when nothing to do
        loop {
            log_info!(cx, "Idle", "Waiting for new data...");
            asm::wfi();
        }
    }

    // ===== TIM2 Interrupt Hardware Task (HIGH PRIORITY) =====
    /// TIM2 interrupt handler bound through RTIC.
    /// High priority: runs as soon as interrupt fires.
    /// Responsibility: read gyro data sensor only.
    #[task(binds = TIM2,
    shared = [sensor_data, new_data_ready, read_sensor_total_cycles, read_sensor_count,itm],
    local = [gyro])]
    fn read_sensor(mut cx: read_sensor::Context) {
        log_info!(cx, "ReadSensor", "TIM2 interrupt fired.");
        let start = DWT::cycle_count();
        // Clear the update interrupt flag (UIF) for TIM2 to acknowledge the interrupt
        let _ = timer::tim2::Tim2Guard::check_and_clear_uif();
        // Read angular velocity from the gyroscope
        let gyro = cx.local.gyro;
        // Update shared resources with new sensor data
        if let Ok((x, y, z)) = gyro.read_angular_velocity() {
            // Update shared resources
            cx.shared.sensor_data.lock(|data| {
                *data = (x, y, z);
            });
            cx.shared.new_data_ready.lock(|ready| {
                *ready = true;
            });

            // Spawn the process_data task to handle the new data
            process_data::spawn().unwrap();
        }

        // Measure performance of read_sensor task
        let end = DWT::cycle_count();
        let elapsed = end.wrapping_sub(start) as u32;
        cx.shared.read_sensor_total_cycles.lock(|total| {
            *total += elapsed;
        });
        cx.shared.read_sensor_count.lock(|count| {
            *count += 1;
        });
    }

    ///Main processing loop (lower priority than interrupt)
    /// Processes the data read from the gyroscope and checks for anomalies.
    #[task(
    priority = 1,
    shared = [sensor_data, new_data_ready,
    read_sensor_total_cycles, read_sensor_count,
    process_data_total_cycles, process_data_count,
    itm],
    local = [ prev_data, counter, anomaly_counter])]
    async fn process_data(mut cx: process_data::Context) {
        log_info!(cx, "ProcessData", "Processing data...");
        let start = DWT::cycle_count();
        // Access shared and local resources
        let prev_data = cx.local.prev_data;
        let counter = cx.local.counter;
        let anomaly_counter = cx.local.anomaly_counter;
        const MAX_DELTA: f32 = 100.0; // Maximum allowed change in angular velocity

        // Check if new data is ready
        let has_data = cx.shared.new_data_ready.lock(|ready| {
            if *ready {
                *ready = false;
                true
            } else {
                false
            }
        });
        if !has_data {
            return;
        }

        // Read the latest sensor data
        let (x, y, z) = cx.shared.sensor_data.lock(|data| *data);

        // Consistency check: Compare with previous data
        let delta_x = (x - prev_data.0).abs();
        let delta_y = (y - prev_data.1).abs();
        let delta_z = (z - prev_data.2).abs();

        if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
            *anomaly_counter += 1;
            log_warn!(
                cx,
                "ProcessData",
                "⚠️  ANOMALY #{}: Δx={:.2}, Δy={:.2}, Δz={:.2}",
                *anomaly_counter,
                delta_x,
                delta_y,
                delta_z
            );
        }

        // Update previous data for the next comparison
        *prev_data = (x, y, z);
        // Print every 4th iteration to avoid flooding the ITM
        if *counter % 4 == 0 {
            log_info!(
                cx,
                "ProcessData",
                "X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s",
                x,
                y,
                z
            );
        }
        *counter += 1;

        // Measure loop performance
        let end = DWT::cycle_count();
        let elapsed = end.wrapping_sub(start) as u32;

        // Update process_data performance metrics
        cx.shared.process_data_total_cycles.lock(|total| {
            *total += elapsed as u32;
        });
        cx.shared.process_data_count.lock(|count| {
            *count += 1;
        });

        // Print statistics every 1000 process_data iterations
        let should_print = cx
            .shared
            .process_data_count
            .lock(|count| *count % 1000 == 0);

        if should_print {
            let (process_total, process_count, read_total, read_count) =
                cx.shared.process_data_total_cycles.lock(|pdt| {
                    cx.shared.process_data_count.lock(|pdc| {
                        cx.shared.read_sensor_total_cycles.lock(|rst| {
                            cx.shared
                                .read_sensor_count
                                .lock(|rsc| (*pdt, *pdc, *rst, *rsc))
                        })
                    })
                });

            let avg_process_cycles = (process_total / 1000) as u32;
            let avg_read_cycles = if read_count > 0 {
                read_total as f64 / read_count as f64
            } else {
                0.0
            };
            // Convert cycles to microseconds (72 MHz = 72 cycles per μs)
            let process_time_us = avg_process_cycles as f64 / 72.0;
            let read_time_us = avg_read_cycles as f64 / 72.0;
            let total_time_us = process_time_us + (read_time_us * 400.0); // 400 read_sensor calls per 2.5s

            // Total measurement period: 1000 iterations at 400 Hz
            // = 1000 / 400 Hz = 2.5 seconds = 2,500,000 microseconds
            let total_period_us = 2_500_000.0;
            // CPU Usage = (Time spent executing / Total measurement time) × 100%
            let cpu_usage = (total_time_us / total_period_us) * 100.0;

            log_stats!(cx, "ProcessData", "");
            log_stats!(cx, "ProcessData", "────────────────────────────────────────────────────────");
            log_stats!(
                cx,
                "ProcessData",
                "📊 System Stats (every 1000 process_data calls / ~2.5s):"
            );
            log_stats!(
                cx,
                "ProcessData",
                "  Process Data Avg: {:.3}μs ({} calls)",
                process_time_us,
                process_count
            );
            log_stats!(
                cx,
                "ProcessData",
                "  Read Sensor Avg: {:.3}μs ({} calls)",
                read_time_us,
                read_count
            );
            log_stats!(cx, "ProcessData", "  Total Time: {:.3}μs", total_time_us);
            log_stats!(cx, "ProcessData", "  Anomalies: {}", *anomaly_counter);
            log_stats!(cx, "ProcessData", "  CPU Usage: {:.2}%", cpu_usage);
            log_stats!(cx, "ProcessData", "────────────────────────────────────────────────────────");
            log_stats!(cx, "ProcessData", "");

            // Reset counters for next measurement window
            cx.shared.process_data_total_cycles.lock(|total| {
                *total = 0;
            });
            cx.shared.process_data_count.lock(|count| {
                *count = 0;
            });
            cx.shared.read_sensor_total_cycles.lock(|total| {
                *total = 0;
            });
            cx.shared.read_sensor_count.lock(|count| {
                *count = 0;
            });
        }
    }
}

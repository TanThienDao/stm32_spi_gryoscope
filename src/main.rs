#![no_main]
#![no_std]

mod config;
mod critical_section_impl;
mod logging;
use rtic::app;
// ===== RTIC APP  =====

/// Shared resources (protected by RTIC)
#[app(device = stm32f3_discovery::stm32f3xx_hal::pac, dispatchers = [EXTI0, EXTI1])]
mod app {
    use crate::config::AppConfig;
    use crate::logging::SensorPort;
    use crate::{log_debug, log_info, log_init, log_sensor, log_stats, log_warn};
    use auxiliary::accelerometer_magnetometer_driver::driver::accel_config::{
        AccelConfig, AccelDataRate, AccelOperatingMode, AccelRange,
    };
    use auxiliary::accelerometer_magnetometer_driver::driver::accl_mag_driver::Lsm303agr;
    use auxiliary::accelerometer_magnetometer_driver::driver::mag_config::{
        MagConfig, MagDataRate, MagOperatingMode, MagPowerMode,
    };
    use auxiliary::timer::general_purpose_timers::timer_trait::TimerGuard;
    use auxiliary::*;
    use rtic::Mutex;
    use stm32f3_discovery::leds::{Direction, Leds};
    use core::f32::consts::PI;
    use micromath::F32Ext;
    use stm32f3_discovery::switch_hal::OutputSwitch;

    // ===== SHARED RESOURCES =====
    /// Resources shared between tasks (protected by RTIC)
    #[shared]
    struct Shared {
        /// ITM for logging (protected by RTIC)
        itm: ITM,
        /// Shared sensor data (x, y, z angular velocities)
        sensor_data: (f32, f32, f32),
        /// Flag indicating new sensor data is ready
        new_data_ready: bool,
        /// Performane tracking for read_sensor task
        read_sensor_total_cycles: u32,
        read_sensor_count: u32,
        /// Performane tracking for process_data task
        process_data_total_cycles: u32,//todo currently not used
        process_data_count: u32,//todo currently not used
        /// Shared sensor data for accelerometer and magnetometer
        sensor_data_accel: (f32, f32, f32),
        sensor_data_mag: (f32, f32, f32),
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
        /// Local accelerometer and magnetometer driver instance
        accel_mag_driver:
            Lsm303agr<I2c<pac::I2C1, (PB6<Alternate<OpenDrain, 4>>, PB7<Alternate<OpenDrain, 4>>)>>,
        /// Data consistent tracker
        prev_data: (f32, f32, f32),
        /// Counter for tracking anomalies
        counter: u32,
        anomaly_counter: u32,
        leds: Leds,
    }
    // ===== RTIC APP =====
    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        //Step 0: Enable TIM2 clock in RCC (APB1ENR)
        ctx.device.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());
        //init TIM2 for interrupt-driven updates
        init_time2(&mut ctx.device).unwrap();

        //Step 0: Enable TIM3 clock in RCC (APB1ENR)
        ctx.device.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());
        init_time3(&mut ctx.device).unwrap();

        //Set up clock
        let mut dwt = ctx.core.DWT;
        let mut flash = ctx.device.FLASH.constrain();
        let mut rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        //==============================================================
        // SPI1 Configuration Start
        //==============================================================

        // Note: How to know where to look for the SPI pins?
        // Check the STM32F303xB STM32F303xC datasheet, specifically the "Alternate Function Mapping" section.
        // For SPI1, the default pins are PA5 (SCK), PA6 (MISO), and PA7 (MOSI).
        // These pins are configured in alternate function mode AF5 for SPI1.
        // Table 14 Page 45 in STM32F303xB STM32F303xC datasheet.
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
        // Note: Check
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
            Hertz(1_000_000), // 1 MHz Config clock baud rate, can be adjusted based on the gyroscope's datasheet (SPI clock frequency) max is 10 MHz for I3G4250D
            clocks,
            &mut rcc.apb2,
        );
        //==============================================================
        // SPI1 Configuration End
        //==============================================================

        //==============================================================
        // I2C Configuration Start
        //==============================================================
        // Note: How to know where to look for the I2C pins?
        // Check the STM32F303xB STM32F303xC datasheet, specifically the "Alternate Function Mapping" section.
        // For I2C1, the default pins are PB6 (SCL) and PB7 (SDA).
        // These pins are configured in alternate function mode AF4 for I2C1.
        // Table 14 Page 45 in STM32F303xB STM32F303xC datasheet.
        let mut gpiob = ctx.device.GPIOB.split(&mut rcc.ahb);
        let scl =
            gpiob
                .pb6
                .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let sda =
            gpiob
                .pb7
                .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let i2c = I2c::new(
            ctx.device.I2C1,
            (scl, sda),
            Hertz(100_000), // 100 kHz standard mode, 400 kHz fast mode, 1 MHz fast mode plus.
            clocks,
            &mut rcc.apb1,
        );

        //==============================================================
        // I2C Configuration End
        //==============================================================

        //==============================================================
        // LEDs Configuration Start
        //==============================================================

        // GPIOE already split above, so we can use it to configure the LEDs
        let leds = Leds::new(
            gpioe.pe8,  // LD4 (Blue)
            gpioe.pe9,  // LD3 (Red)
            gpioe.pe10, // LD5 (Orange)
            gpioe.pe11, // LD7 (Green)
            gpioe.pe12, // LD9 (Blue)
            gpioe.pe13, // LD10 (Red)
            gpioe.pe14, // LD8 (Orange)
            gpioe.pe15, // LD6 (Green)
            &mut gpioe.moder,
            &mut gpioe.otyper,
        );

        //==============================================================
        // LEDs Configuration End
        //==============================================================

        let delay = Delay::new(ctx.core.SYST, clocks);
        let mut itm = ctx.core.ITM;
        log_init!(
            itm,
            "=============================================================="
        );
        log_init!(itm, "Phase 3: RTIC-based I3G4250D Gyroscope");
        log_init!(
            itm,
            "=============================================================="
        );

        log_init!(
            itm,
            "--------------------------------------------------------------"
        );
        log_init!(itm, "📋 Configuration Loaded:");
        log_init!(itm, " Debug Logs: {}", AppConfig::DEBUG_ENABLED);
        log_init!(itm, " Stats Enabled: {}", AppConfig::STATS_ENABLED);
        log_init!(
            itm,
            " Max Angular Velocity Change: {}°/s",
            AppConfig::MAX_ANGULAR_VELOCITY_CHANGE
        );
        log_init!(
            itm,
            " Log Frequency: Every {} iterations",
            AppConfig::LOG_FREQUENCY
        );
        log_init!(
            itm,
            " Stats Interval: Every {} iterations",
            AppConfig::STATS_INTERVAL
        );
        log_init!(
            itm,
            "--------------------------------------------------------------"
        );
        //==============================================================
        // Step 1.1: Detect Gyro driver
        //==============================================================
        log_init!(itm, "Step 1.1: Detect Gyro driver...");
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
        //==============================================================
        // Step 1.2: Detect Magnetometer driver
        //==============================================================
        log_init!(itm, "Step 1.2: Detect Magnetometer driver...");
        let mut accel_mag_driver = Lsm303agr::new(
            i2c,
            AccelConfig::new(
                AccelRange::G2,
                AccelOperatingMode::HighResolution,
                AccelDataRate::Hz100,
            ),
            MagConfig::new(
                MagOperatingMode::Continuous,
                MagDataRate::Hz100,
                MagPowerMode::HighResolution,
            ),
        );
        match accel_mag_driver.verify_device_identity() {
            Ok(id) => {
                if id.0 == 0x33 && id.1 == 0x40 {
                    log_init!(itm, "✓ Accelerometer ID: 0x{:X} (LSM303AGR)", id.0);
                    log_init!(itm, "✓ Magnetometer ID: 0x{:X} (LSM303AGR)", id.1);
                } else {
                    log_init!(
                        itm,
                        "✗ Unexpected Magnetometer ID: 0x{:X} and 0x{:X}",
                        id.0,
                        id.1
                    );
                    loop {}
                }
            }
            Err(_) => {
                log_init!(itm, "✗ Error reading Magnetometer ID!");
                loop {}
            }
        }
        log_init!(itm, "");
        //==============================================================
        // Detect I3G4250D and LSM303AGR driver complete
        //==============================================================

        //==============================================================
        // Step 2.1: Initialize Gyro driver
        //==============================================================
        log_init!(itm, "Step 2.1: Init Gyro driver...");
        // Initialize the gyro
        if let Err(e) = gryo.init() {
            log_init!(itm, "✗ Error initializing Gyro: {:?}", e);
            loop {}
        }
        log_init!(itm, "✓ Gyro driver initialized.");

        //Configure Gyro
        log_init!(itm, " Configuring Gyro...");
        if let Err(e) = gryo.set_data_rate(DataRate::Hz400) {
            log_init!(itm, "✗ Error setting data rate: {:?}", e);
            loop {}
        }
        if let Err(e) = gryo.set_range(Range::DPS500) {
            log_init!(itm, "✗ Error setting range: {:?}", e);
            loop {}
        }
        log_init!(itm, "✓ Gyro configured.");
        log_init!(itm, "✓ Configuration Gyroscope complete:");
        log_init!(itm, "  - Data Rate: 100 Hz (timer interrupt)");
        log_init!(itm, "  - Range: 500 °/s");
        log_init!(itm, "");

        //==============================================================
        // Step 2.2: Initialize Accelerometer and Magnetometer driver
        //==============================================================
        log_init!(
            itm,
            "Step 2.2: Init Accelerometer and Magnetometer driver..."
        );
        if let Err(e) = accel_mag_driver.init() {
            log_init!(
                itm,
                "✗ Error initializing Accelerometer and Magnetometer: {:?}",
                e
            );
            loop {}
        }
        log_init!(itm, "✓ Accelerometer and Magnetometer driver initialized.");
        log_init!(
            itm,
            "✓ Configuration Accelerometer and Magnetometer complete:"
        );
        log_init!(itm, "  - Accelerometer: ±2g, High-Resolution, 100 Hz");
        log_init!(itm, "  - Magnetometer: Continuous, 100 Hz, High-Resolution");
        log_init!(itm, "");

        log_init!(
            itm,
            "Enabling DWT cycle counter for performance measurement..."
        );
        dwt.enable_cycle_counter();
        log_init!(itm, "");

        log_init!(itm, "✓ RTIC system ready. Starting main loop...");
        log_init!(
            itm,
            "=============================================================="
        );
        //==============================================================
        // End initialization for both Gyroscope and Accelerometer/Magnetometer
        //==============================================================

        // shared resources initialization
        let shared = Shared {
            // Shared sensor data (x, y, z angular velocities)
            sensor_data: (0.0, 0.0, 0.0),
            new_data_ready: false,
            read_sensor_total_cycles: 0,
            read_sensor_count: 0,
            process_data_total_cycles: 0,
            process_data_count: 0,
            //------------------------------------------------
            itm,
            // Shared sensor data for accelerometer and magnetometer
            sensor_data_accel: (0.0, 0.0, 0.0),
            sensor_data_mag: (0.0, 0.0, 0.0),
            //------------------------------------------------
        };

        // Initialize local resources
        let local = Local {
            // Local gyro driver instance
            gyro: gryo,
            prev_data: (0.0, 0.0, 0.0),
            counter: 0,
            anomaly_counter: 0,
            // Local accelerometer and magnetometer driver instance
            accel_mag_driver: accel_mag_driver,
            leds
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
        log_info!(cx, "Idle", "Waiting for new data...");
        loop {
            if AppConfig::DEBUG_ENABLED {
                log_debug!(cx, "Idle", "CPU is idle, waiting for new data...");
            }
            asm::wfi();
        }
    }

    // ===== TIM2 Interrupt Hardware Task (HIGH PRIORITY) =====
    /// TIM2 interrupt handler bound through RTIC.
    /// High priority: runs as soon as interrupt fires.
    /// Responsibility: read gyro data sensor only.
    #[task(binds = TIM2,
    shared = [sensor_data, new_data_ready, read_sensor_total_cycles, read_sensor_count,itm],
    local = [gyro,prev_data, counter, anomaly_counter])]
    fn gyroscope_task(mut cx: gyroscope_task::Context) {
        if AppConfig::DEBUG_ENABLED {
            log_debug!(cx, "Gyroscope", "🔴 TIM2 interrupt fired!");
        }
        const MAX_DELTA: f32 = AppConfig::MAX_ANGULAR_VELOCITY_CHANGE; // Maximum allowed change in angular velocity
        if AppConfig::DEBUG_ENABLED {
            log_debug!(
                cx,
                "Gyroscope",
                "TIM2 interrupt fired, reading sensor data..."
            );
        }
        let start = DWT::cycle_count();
        // Clear the update interrupt flag (UIF) for TIM2 to acknowledge the interrupt
        let _ = timer::general_purpose_timers::tim2::Tim2Guard::clear_update_interrupt_flag();
        // Read angular velocity from the gyroscope
        let gyro = cx.local.gyro;

        // Update shared resources with new sensor data
        if let Ok((x, y, z)) = gyro.read_angular_velocity() {
            log_info!(
                cx,
                "Gyroscope",
                "✓ Read sensor data: X: {:7.2}°/s | Y: {:7.2}°/s | Z: {:7.2}°/s",
                x,
                y,
                z
            );
            // Update shared resources
            cx.shared.sensor_data.lock(|data| {
                *data = (x, y, z);
            });
            cx.shared.new_data_ready.lock(|ready| {
                *ready = true;
            });
        }

        // Consistency check: Compare with previous data
        let delta_x = (cx.shared.sensor_data.lock(|data| data.0) - cx.local.prev_data.0).abs();
        let delta_y = (cx.shared.sensor_data.lock(|data| data.1) - cx.local.prev_data.1).abs();
        let delta_z = (cx.shared.sensor_data.lock(|data| data.2) - cx.local.prev_data.2).abs();
        if delta_x > MAX_DELTA || delta_y > MAX_DELTA || delta_z > MAX_DELTA {
            *cx.local.anomaly_counter += 1;
            log_warn!(
                cx,
                "Gyroscope",
                "⚠️  ANOMALY #{}: Δx={:.2}, Δy={:.2}, Δz={:.2}",
                *cx.local.anomaly_counter,
                delta_x,
                delta_y,
                delta_z
            );
            *cx.local.anomaly_counter += 1;
        }
        // Update previous data for the next comparison
        *cx.local.prev_data = cx.shared.sensor_data.lock(|data| *data);

        // Increment the counter for tracking performance metrics
        *cx.local.counter += 1;
        if AppConfig::DEBUG_ENABLED {
            log_debug!(
                cx,
                "Gyroscope",
                "Read sensor data complete, counter: {}",
                *cx.local.counter
            );
        }

        // Measure performance of read_sensor task
        if AppConfig::DEBUG_ENABLED {
            log_info!(cx, "Gyroscope", "⏱️  Measuring performance...");
        }
        let end = DWT::cycle_count();
        let elapsed = end.wrapping_sub(start) as u32;
        /// Total cucles for 1000 records, average cycles per read_sensor call, and CPU usage percentage
        let process_total = cx.shared.read_sensor_total_cycles.lock(|total| {
            *total += elapsed;
            *total
        });
        /// Total count of read_sensor calls
        let count = cx.shared.read_sensor_count.lock(|count| {
            *count += 1;
            *count
        });
        /// Print statistics every 1000 process_data iterations
        let should_print = *cx.local.counter % AppConfig::STATS_INTERVAL == 0;
        if should_print {
            log_info!(
                cx,
                "Gyroscope",
                "📊 Read_sensor performance: total_cycles={}, count={}",
                process_total,
                AppConfig::STATS_INTERVAL
            );
            /// Average cycles per read_sensor call and CPU usage percentage
            let avg_cycles = (process_total / AppConfig::STATS_INTERVAL) as u32;
            let avg_read_cycles = if count > 0 {
                process_total as f64 / count as f64
            } else {
                0.0
            };
            /// Convert cycles to microseconds (72 MHz = 72 cycles per μs)
            let read_time_us = avg_read_cycles as f64 / 72.0;
            /// Total measurement period: 1000 iterations at 400 Hz
            /// = 1000 / 400 Hz = 2.5 seconds = 2,500,000 microseconds
            /// Convert cycles to microseconds (72 MHz = 72 cycles per μs)
            let total_period_us = 2_500_000.0;
            /// CPU Usage = (Time spent executing / Total measurement time) × 100%
            let cpu_usage = (read_time_us / total_period_us) * 100.0; // Total measurement period: 2.5 seconds
            log_stats!(cx, "Gyroscope", "");
            log_stats!(
                cx,
                "Gyroscope",
                "────────────────────────────────────────────────────────"
            );
            log_stats!(
                cx,
                "Gyroscope",
                "📊 Read_sensor Gyroscope Stats (every {} calls / ~2.5s):",
                AppConfig::STATS_INTERVAL
            );
            log_stats!(cx, "Gyroscope", "  Total Cycles: {}", process_total);
            log_stats!(
                cx,
                "Gyroscope",
                "  Abnormal Count: {}",
                *cx.local.anomaly_counter
            );
            log_stats!(
                cx,
                "Gyroscope",
                "  Read Sensor Avg: {:.3}μs ({} calls)",
                read_time_us,
                count
            );
            log_stats!(cx, "Gyroscope", "  CPU Usage: {:.2}%", cpu_usage);
            log_stats!(
                cx,
                "Gyroscope",
                "────────────────────────────────────────────────────────"
            );
            /// Reset counters for next measurement window
            cx.shared.read_sensor_total_cycles.lock(|total| {
                *total = 0;
            });
            cx.shared.read_sensor_count.lock(|count| {
                *count = 0;
            });
            *cx.local.anomaly_counter = 0;
        }

        if AppConfig::DEBUG_ENABLED {
            log_debug!(
                cx,
                "Gyroscope",
                "✅ Read sensor data complete, counter: {}, elapsed cycles: {}",
                *cx.local.counter,
                elapsed
            );
        }
    }

    // ==== TIM3 Interrupt Hardware Task (HIGH PRIORITY) ====
    /// TIM3 interrupt handler bound through RTIC.
    /// High priority: runs as soon as interrupt fires.
    /// Responsibility: read accelerometer and magnetometer data sensor only.
    #[task(binds = TIM3,
    shared = [sensor_data_accel, sensor_data_mag, itm],
    local = [accel_mag_driver, leds])]
    fn accel_mag_task(mut cx: accel_mag_task::Context) {
        if AppConfig::DEBUG_ENABLED {
            log_debug!(cx, "ReadSensor", "🔴 TIM3 interrupt fired!");
        }
        let start = DWT::cycle_count();
        // Clear the update interrupt flag (UIF) for TIM3 to acknowledge the interrupt
        let _ = timer::general_purpose_timers::tim3::Tim3Guard::clear_update_interrupt_flag();
        // Read accelerometer and magnetometer data
        let accel_mag_driver = cx.local.accel_mag_driver;
        if let Ok((accel_data)) = accel_mag_driver.read_accel_g() {
            log_info!(cx, "Accelerometer", "✓ Read accelerometer data: X: {:7.2}g | Y: {:7.2}g | Z: {:7.2}g",
                accel_data.0, accel_data.1, accel_data.2);
            // Update shared resources
            cx.shared.sensor_data_accel.lock(|data| {
                *data = accel_data;
            });
        } else {
            log_warn!(cx, "Accelerometer", "✗ Error reading accelerometer and magnetometer data!");
        }
        if let Ok((mag_data)) = accel_mag_driver.read_mag_gauss() {
            log_info!(cx, "Magnetometer", "✓ Read magnetometer data: X: {:7.2}G | Y: {:7.2}G | Z: {:7.2}G",
                mag_data.0, mag_data.1, mag_data.2);
            // Update shared resources
            cx.shared.sensor_data_mag.lock(|data| {
                *data = mag_data;
                // 1. Determine direction from X and Y.
                let direction = calculate_compass_direction(mag_data.0, mag_data.1);
                // 2. Turn off all LEDs then illuminate the LED corresponding to the direction.
                for led in cx.local.leds.iter_mut() {
                    led.off().ok();
                }
                cx.local.leds.for_direction(direction).on().ok();
            });
        } else {
            log_warn!(cx, "Magnetometer", "✗ Error reading magnetometer data!");
        }
        let end = DWT::cycle_count();
        let elapsed = end.wrapping_sub(start) as u32;
        if AppConfig::DEBUG_ENABLED {
            log_debug!(cx, "ReadSensor", "✅ Read accelerometer and magnetometer data complete, elapsed cycles: {}", elapsed);
        }
    }

    fn calculate_compass_direction(x: f32, y: f32) -> Direction {
        // atan2 return radians in range [-π, π]
        let theta = y.atan2(x); // Calculate the angle in radians
        if theta < -7.0 * PI / 8.0 || theta > 7.0 * PI / 8.0 {
            Direction::North
        } else if theta < -5.0 * PI / 8.0 {
            Direction::NorthWest
        } else if theta < -3.0 * PI / 8.0 {
            Direction::West
        } else if theta < -1.0 * PI / 8.0 {
            Direction::SouthWest
        } else if theta < 1.0 * PI / 8.0 {
            Direction::South
        } else if theta < 3.0 * PI / 8.0 {
            Direction::SouthEast
        } else if theta < 5.0 * PI / 8.0 {
            Direction::East
        } else if theta < 7.0 * PI / 8.0 {
            Direction::NorthEast
        } else {
            Direction::North
        }
    }





}

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use auxiliary::*;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs) = init();

    iprintln!(&mut itm.stim[0], "===============================");
    iprintln!(&mut itm.stim[0], "I3G4250D Gyroscope Demo");
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

    if let Err(e) = gyro.set_data_rate(DataRate::Hz400) {
        iprintln!(&mut itm.stim[0], "✗ DataRate config error: {}", e);
        loop {}
    }

    if let Err(e) = gyro.set_range(Range::DPS500) {
        iprintln!(&mut itm.stim[0], "✗ Range config error: {}", e);
        loop {}
    }

    iprintln!(&mut itm.stim[0], "✓ Configuration complete:");
    iprintln!(&mut itm.stim[0], "  - Data Rate: 400 Hz");
    iprintln!(&mut itm.stim[0], "  - Range: 500 °/s");
    iprintln!(&mut itm.stim[0], "  - All axes enabled");

    // Step 4: Start reading sensor data
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Step 4: Starting sensor readings...");
    iprintln!(&mut itm.stim[0], "");
    iprintln!(&mut itm.stim[0], "Angular Velocity (°/s):");
    iprintln!(&mut itm.stim[0], "───────────────────────");

    let mut counter = 0u32;

    loop {
        match gyro.read_angular_velocity() {
            Ok((x, y, z)) => {
                // Print every 4th reading (~100ms at 400Hz, every ~10.5ms per read)
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
    }
}

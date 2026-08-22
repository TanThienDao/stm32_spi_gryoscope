#![deny(unsafe_code)]
#![no_main]
#![no_std]

use auxiliary::*;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let (mut itm, _delay, mut spi, mut cs) = init();

    iprintln!(&mut itm.stim[0], "Gyroscope initialization starting...");

    // Example: Read the WHO_AM_I register (0x0F) from the gyroscope
    // Method 1
    match identify_gryoscope(&mut spi, &mut cs) {
        Ok(device) => {
            // Successfully identified the gyroscope
            iprintln!(&mut itm.stim[0], "Found Device: {}", device);
        }
        Err(e) => {
            // Handle the error
            iprintln!(&mut itm.stim[0], "Error: {}", e);
        }
    }
    //Method 2
    match detect_gyroscope(&mut spi, &mut cs) {
        Ok(variant) => {
            iprintln!(&mut itm.stim[0], "Found Gyroscope Variant: {:?}", variant);
        }
        Err(_) => {
            iprintln!(&mut itm.stim[0], "Error detecting gyroscope variant");
        }
    }

    // We found out that our gyroscope is I3G4250D, so we can use the I3G4250D driver to read data from it.

    loop {}
}

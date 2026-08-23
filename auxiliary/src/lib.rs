#![no_std]

#[allow(unused_extern_crates)]
extern crate panic_itm;

pub use cortex_m::{
    asm, interrupt, iprint, iprintln,
    peripheral::{Peripherals, DWT, ITM, NVIC, SYST},
};
pub use cortex_m_rt::entry;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;
pub use stm32f3_discovery::stm32f3xx_hal::{
    delay::Delay,
    pac::{self, SPI1},
    prelude::*,
    spi::{MisoPin, Mode, MosiPin, Phase, Polarity, SckPin, Spi},
    time::rate::Hertz,
};

pub fn init() -> (
    ITM,
    Delay,
    Spi<SPI1, (impl SckPin<SPI1>, impl MisoPin<SPI1>, impl MosiPin<SPI1>)>,
    impl OutputPin,
) {
    let cp = Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    //==============================================================
    // SPI1 Configuration Start
    //==============================================================

    // Configure GPIOA and GPIOE for SPI and CS pin
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    // SPI pins (PA5=SCK, PA6=MISO, PA7=MOSI)
    let sck = gpioa
        .pa5
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let miso = gpioa
        .pa6
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mosi = gpioa
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
        dp.SPI1,
        (sck, miso, mosi),
        mode,
        Hertz(1_000_000), // 1 MHz Config clock baud rate, can be adjusted based on the gyroscope's datasheet
        clocks,
        &mut rcc.apb2,
    );
    //==============================================================
    // SPI1 Configuration End
    //==============================================================

    let delay = Delay::new(cp.SYST, clocks);

    (cp.ITM, delay, spi, cs)
}
pub fn identify_gryoscope<PINS, CS>(
    spi: &mut Spi<SPI1, PINS>,
    cs: &mut CS,
) -> Result<&'static str, &'static str>
where
    CS: stm32f3_discovery::stm32f3xx_hal::prelude::_embedded_hal_digital_OutputPin,
{
    // Read the WHO_AM_I register (0x0F)
    // For SPI read, set MSB to 1: 0x0F | 0x80 = 0x8F
    const WHO_AM_I: u8 = 0x0F;
    let mut buffer = [0u8; 2];

    // Set CS low before starting the SPI transaction (in STM32F3 this is NSS Slave Select pin, active low)
    cs.set_low().ok();

    // Prepare buffer: first byte is address, second byte will contain response
    buffer[0] = WHO_AM_I | 0x80;

    // Identify the gyroscope by reading the WHO_AM_I register
    let result = match spi.transfer(&mut buffer) {
        Ok(_) => {
            // Set CS high after the SPI transaction
            cs.set_high().ok();

            // buffer[1] contains the response
            match buffer[1] {
                0xD4 => Ok("L3GD20"),   // L3GD20 is 11010100 is D4h
                0xD3 => Ok("I3G4250D"), // I3G4250D is 11010011 is D3h
                0xD7 => Ok("L3GD20H"),  // L3GD20H is 11010111 is D7h
                _ => Err("Unknown gyroscope device"),
            }
        }
        Err(_) => {
            // Set CS high after the SPI transaction
            cs.set_high().ok();
            Err("Failed to read WHO_AM_I register")
        }
    };
    result
}

/// Identified Gyroscope Variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GyroVariant {
    I3g4250d,
    L3gd20,
    L3gd20h,
    Unknown(u8),
}

/// Reads the WHO_AM_I register (0x0F) via SPI and identifies the gyroscope model.
///
/// * `spi` - Reference to configured SPI peripheral (Mode 3)[cite: 1]
/// * `cs`  - Active-low Chip Select output pin (PE3)[cite: 1]
pub fn detect_gyroscope<SPI, CS, E>(spi: &mut SPI, cs: &mut CS) -> Result<GyroVariant, E>
where
    SPI: Transfer<u8, Error = E>,
    CS: OutputPin,
{
    // SPI Read Protocol: Bit 0 = 1 (Read mode) | Bit 1 = 0 (Single byte) | Bits 2-7 = Reg address (0x0F)[cite: 1]
    let mut buffer = [0x0F | 0x80, 0x00];

    // Execute SPI transaction
    cs.set_low().ok();
    let result = spi.transfer(&mut buffer);
    cs.set_high().ok();

    // Return the error if SPI transfer failed
    result?;

    // Read response byte from full-duplex transfer buffer[cite: 1]
    let who_am_i = buffer[1];

    // Identify variant based on WHO_AM_I byte
    match who_am_i {
        0xD3 => Ok(GyroVariant::I3g4250d), // I3G4250D is 11010011 is D3h
        0xD4 => Ok(GyroVariant::L3gd20),   // L3GD20 is 11010100 is D4h
        0xD7 => Ok(GyroVariant::L3gd20h),  // L3GD20H is 11010111 is D7h
        other => Ok(GyroVariant::Unknown(other)),
    }
}

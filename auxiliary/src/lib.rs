#![no_std]

#[allow(unused_extern_crates)]
extern crate panic_itm;

pub use cortex_m::{
    self,
    asm, iprint, iprintln,
    //interrupt::{self, Mutex,free},
    peripheral::{Peripherals, DWT, ITM, NVIC, SYST}
};
pub use cortex_m_rt::{entry};
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;
pub use stm32f3_discovery::stm32f3xx_hal::{
    delay::Delay,
    pac::{self, SPI1,TIM2},
    prelude::*,
    spi::{MisoPin, Mode, MosiPin, Phase, Polarity, SckPin, Spi},
    time::rate::Hertz,
    interrupt::{self},

};

// Custom driver module for I3G4250D
pub mod gyro_driver;
pub use gyro_driver::{GyroDriver, DataRate, Range};

// Interrupt handler module for Phase 2
pub mod interrupt_handler;

pub fn init() -> (
    ITM,
    Delay,
    Spi<SPI1, (impl SckPin<SPI1>, impl MisoPin<SPI1>, impl MosiPin<SPI1>)>,
    impl OutputPin,
    DWT,
) {
    let cp = Peripherals::take().unwrap();
    let mut dp = pac::Peripherals::take().unwrap();
    // Enable TIM2 clock in RCC (APB1ENR)
    dp.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());
    let dwt = cp.DWT;
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
    //init TIM2 for interrupt-driven updates
    init_tim2(&mut dp.TIM2, 72_000_000, 100_000, 400);

    (cp.ITM, delay, spi, cs,dwt)
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
    // Check section 5.2 of the I3G4250D datasheet for details on the SPI read protocol..
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
    // Check section 5.2 of the I3G4250D datasheet for details on the SPI read protocol..
    let mut buffer = [0x0F | 0x80, 0x00]; // Master's data to send: [WHO_AM_I | 0x80 (Read mode), dummy byte for response]

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

/// Calculate prescaler and period values for desired interrupt frequency base on the timer clock.
///  return psc and arr values
pub fn calculate_timer_values(
    cpu_clock_hz: u32,
    target_timer_freq_hz: u32,
    target_interrupt_freq_hz: u32,
) -> (u16,u32){
    // PSC (CPU_clock / (target_timer_freq)) - 1
    let psc = (cpu_clock_hz / target_timer_freq_hz) - 1;

    // ARR (target_timer_freq / target_interrupt_freq) - 1
    let arr = (target_timer_freq_hz / target_interrupt_freq_hz) - 1;
    (psc as u16, arr)
}
/// Initialize TIM2 with calculated prescaler and auto-reload values.
/// Set PSC and ARR, enable counter, and configure update interrupt.
/// Setup NVIC to unmask TIM2 interrupt.
pub fn config_tim2(tim2: &mut TIM2, psc: u16,arr:u32) -> &mut TIM2{
    // ====== TIM2 Configuration Start ======
    //Step 1:  Set PSC: Prescaler (divides input clock frequency)
    tim2.psc.write(|w| w.psc().bits(psc));
    //Step 2: Set ARR: Auto-reload register (defines the period of the timer)
    tim2.arr.write(|w| w.arr().bits(arr));
    //Step 3: Enable counter and update event generation
    tim2.cr1.write(|w| {
        w.cen().set_bit()         // Counter enabled
            .udis().clear_bit()          // Update event enabled
            .dir().clear_bit()           // Count up
            .arpe().clear_bit()                  // Auto-reload preload disabled
    });
    //Step 4: Enable TIM2 interrupt on update event (UIE)
    tim2.dier.write(|w| w.uie().set_bit());
    tim2
}

pub fn init_tim2(
    tim2: &mut TIM2,
    cpu_clock_hz: u32,
    target_timer_freq_hz: u32,
    target_interrupt_freq_hz: u32,
) -> & TIM2 {

    // Calculate prescaler and auto-reload values
    let (psc, arr) = calculate_timer_values(cpu_clock_hz, target_timer_freq_hz, target_interrupt_freq_hz);
    config_tim2(tim2, psc, arr)
}

pub fn enable_tim2_interrupt() {
    // Unmask TIM2 interrupt in NVIC (allow CPU to accept TIM2 interrupts)
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
}
#[derive(Debug)]
pub struct Tim2Guard{
    initialized: bool,
}

impl Tim2Guard {
    /// Initialize TIM2 once safely
    pub fn new() -> Self {
        Tim2Guard { initialized: false }
    }
    pub fn init(&mut self) -> Option<()> {
        if self.initialized {
            return None;
        }
        self.initialized = true;
        // This should be called only one from main
        cortex_m::interrupt::free(|_| {
            Some(())
        })
    }
    pub fn check_and_clear_uif() -> bool {
        let dp = unsafe { pac::Peripherals::steal() };
        let uif = dp.TIM2.sr.read().uif().bit_is_set();
        if uif {
            // Clear the interrupt flag
            dp.TIM2.sr.write(|w| w.uif().clear_bit());
        }
        uif
    }
}
#[derive(Debug)]
pub struct NvicGuard{
    // This struct is used to ensure that NVIC unmasking is done safely and only once.
    initialized: bool,
}

impl NvicGuard {
    pub fn new() -> Self {
        NvicGuard { initialized: false }
    }
    pub fn unmask_tim2_safe(&mut self) -> Result<(), &'static str> {
        if self.initialized {
            return Err("Already initialized"); // Already initialized, do nothing
        }
        unsafe {
            NVIC::unmask(pac::Interrupt::TIM2);
        }
        self.initialized = true;
        Ok(())
    }
}
//! TIM2 specific timer implement

use super::{
    TimerConfig,
    TimerResult,
};
use stm32f3_discovery::stm32f3xx_hal::pac::TIM2;
use crate::pac;

/// TIM2 timer group for STM32F3 series microcontrollers.
pub struct Tim2Guard {
    config: TimerConfig,
    initialized: bool,
}

impl Tim2Guard {
    /// Create a new TIM2 guard with configuration.
    pub fn new(config: TimerConfig) -> TimerResult<Self> {
        config.validate()?;
        Ok(Self {
            config,
            initialized: false,
        })
    }

    /// Initialize TIM2 peripheral
    pub fn init(&mut self, tim2: &mut TIM2) -> TimerResult<()> {
        if self.is_initialized() {
            return Err("TIM2 already initialized");
        }
        let (psc, arr) = self.config.calculate_timer_values();
        cortex_m::interrupt::free(|_| {
            self.config_tim2(tim2, psc, arr);
            self.initialized = true;
            Ok(())
        })
    }

    /// Configure TIM2 with calculated prescaler and auto-reload values.
    pub fn config_tim2(&self, tim2: &mut TIM2, psc: u16, arr: u32) -> TimerResult<()> {
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
        Ok(())
    }

    /// Check if Update Interrupt Flag (UIF) is set, indicating an update event has occurred.
    pub fn is_update_interrupt_flag_set(&self, tim2: &TIM2) -> bool {
        tim2.sr.read().uif().bit_is_set()
    }

    /// Check and clear UIF
    pub fn check_and_clear_uif() -> bool {
        let dp = unsafe { pac::Peripherals::steal() };
        let uif = dp.TIM2.sr.read().uif().bit_is_set();
        if uif {
            dp.TIM2.sr.write(|w| w.uif().clear_bit());
        }
        uif
    }

    /// Get current configuration of TIM2
    pub fn get_config(&self) -> &TimerConfig {
        &self.config
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tim2_guard_initialization() {
        let config = TimerConfig::new(100_000, 400);
        let mut tim2_guard = Tim2Guard::new(config).unwrap();
        assert!(!tim2_guard.is_initialized());
    }
    #[test]
    fn test_tim2_guard_create() {
        let config = TimerConfig::new(100_000, 400);
        let mut tim2_guard = Tim2Guard::new(config);
        assert!(tim2_guard.is_ok());
    }
    #[test]
    fn test_tim2_get_config() {
        let config = TimerConfig::new(100_000, 400);
        let tim2_guard = Tim2Guard::new(config).unwrap();
        let retrieved_config = tim2_guard.get_config();
        assert_eq!(retrieved_config.cpu_clock_hz, 72_000_000);
        assert_eq!(retrieved_config.target_timer_clock_hz, 100_000);
        assert_eq!(retrieved_config.interrupt_enable_hz, 400);
    }
}
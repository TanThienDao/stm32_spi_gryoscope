//! =====================================================================
//!
//! General purpose timer TIM2
//! This module provides a guard for the TIM2 timer peripheral,
//! allowing for safe and controlled access to its configuration and operation.
//! The `Tim2Guard` struct encapsulates the TIM2 peripheral and provides methods to
//! initialize, start, stop, reset, and manage interrupts for the timer.
//!
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use crate::timer::{
    general_purpose_timers::timer_trait::{TimXResult, TimerError, TimerGuard},
    TimerConfig,
};

use crate::pac::TIM2;

pub struct Tim2Guard {
    config: TimerConfig,
    initialized: bool,
}

impl Tim2Guard {
    /// Create a new instance of Tim2Guard with the provided configuration.
    pub fn new(config: TimerConfig) -> TimXResult<Self> {
        Ok(Self {
            config,
            initialized: false,
        })
    }
    /// Get a reference to the current configuration of the timer.
    pub fn get_config(&self) -> &TimerConfig {
        &self.config
    }
    /// Check if the timer has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// Set the initialized state of the timer.
    pub fn set_initialized(&mut self, state: bool) {
        self.initialized = state;
    }
}

impl TimerGuard for Tim2Guard {
    /// Asscociated type for the specific timer peripheral (e.g., TIM2, TIM3, etc.)
    /// This ensure the guard is bound to exactly one timer at compile time.
    type Peripheral = TIM2;

    /// Initialize the timer with clock, prescaler, and auto-reload values based on the provided configuration.
    /// Return result to catch configuration errors.
    fn init(&mut self, tim2: &mut TIM2) -> TimXResult<()> {
        if self.is_initialized() {
            return Err(TimerError::AlreadyInitialized("TIM2 already initialized"));
        }
        self.config
            .validate()
            .map_err(|e| TimerError::InvalidConfiguration(e))?;
        let (psc, arr) = self.config.calculate_timer_values();
        //Step 1:  Set PSC: Prescaler (divides input clock frequency)
        tim2.psc.write(|w| w.psc().bits(psc));
        //Step 2: Set ARR: Auto-reload register (defines the period of the timer)
        tim2.arr.write(|w| w.arr().bits(arr)); // ARR is 32-bit for TIM2
        // Step 3: Enable counter and update event generation
        tim2.cr1.write(|w| {
            w.cen()
                .set_bit() // Counter enabled
                .udis()
                .clear_bit() // Update event enabled
                .dir()
                .clear_bit() // Count up
                .arpe()
                .set_bit() // Auto-reload preload enable
        });
        // Step 4: Enable TIM2 interrupt on update event (UIE)
        tim2.dier.write(|w| w.uie().set_bit());
        self.set_initialized(true);
        Ok(())
    }
    /// Start the timer and enable interrupts.
    fn start(&mut self, tim2: &mut TIM2) -> TimXResult<()> {
        if !self.is_initialized() {
            return Err(TimerError::NotInitialized);
        }
        tim2.cr1.modify(|_, w| w.cen().set_bit());
        Ok(())
    }
    /// Stop the timer and disable interrupts.
    fn stop(&mut self, tim2: &mut TIM2) -> TimXResult<()> {
        if !self.is_initialized() {
            return Err(TimerError::NotInitialized);
        }
        tim2.cr1.modify(|_, w| w.cen().clear_bit());
        Ok(())
    }
    /// Reset the timer counter to zero
    fn reset(&mut self, tim2: &mut TIM2) -> TimXResult<()> {
        if !self.is_initialized() {
            return Err(TimerError::NotInitialized);
        }
        tim2.cnt.write(|w| w.cnt().bits(0));
        Ok(())
    }
    /// Get current counter value.
    /// TIM2 is a 32-bit timer, so the counter value is returned as u32.
    fn get_counter(&mut self, tim2: &mut TIM2) -> TimXResult<u32> {
        if !self.is_initialized() {
            return Err(TimerError::NotInitialized);
        }
        Ok(tim2.cnt.read().cnt().bits())
    }
    /// Check if update interrupt flag (UIF) is set, indicating an update event has occurred.
    fn is_update_interrupt_flag_set(&mut self, tim2: &TIM2) -> TimXResult<bool> {
        if !self.is_initialized() {
            return Err(TimerError::NotInitialized);
        }
        Ok(tim2.sr.read().uif().bit_is_set())
    }
    /// Clear the update interrupt flag (UIF) after handling the interrupt.
    fn clear_update_interrupt_flag() -> TimXResult<()> {
        let dp = unsafe { crate::pac::Peripherals::steal() };
        dp.TIM2.sr.write(|w| w.uif().clear_bit());
        Ok(())
    }
    /// Set the update interrupt flag (UIF) to trigger an update event.
    fn set_update_interrupt_flag(&mut self, tim2: &mut TIM2) -> TimXResult<()> {
        if !self.is_initialized() {
            return Err(TimerError::NotInitialized);
        }
        tim2.sr.write(|w| w.uif().set_bit());
        Ok(())
    }
}

/// Unit tests for the Tim2Guard struct and its methods.
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
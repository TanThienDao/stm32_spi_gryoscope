//! =====================================================================
//!
//! General purpose timer TIM3
//! This module provides a guard for the TIM3 timer peripheral,
//! allowing for safe and controlled access to its configuration and operation.
//! The `Tim3Guard` struct encapsulates the TIM3 peripheral and provides methods to
//! initialize, start, stop, reset, and manage interrupts for the timer.
//!
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use crate::timer::{
    general_purpose_timers::timer_trait::{TimXResult, TimerError, TimerGuard},
    TimerConfig,
};

use crate::pac::TIM3;

pub struct Tim3Guard {
    config: TimerConfig,
    initialized: bool,
}

impl Tim3Guard {
    /// Create a new instance of Tim3Guard with the provided configuration.
    pub fn new(config: TimerConfig) -> TimXResult<Self> {
        Ok(Self {
            config,
            initialized: false,
        })
    }
}

impl TimerGuard for Tim3Guard {
    /// Asscociated type for the specific timer peripheral (e.g., TIM2, TIM3, etc.)
    /// This ensure the guard is bound to exactly one timer at compile time.
    type Peripheral = TIM3;

    /// Initialize the timer with clock, prescaler, and auto-reload values based on the provided configuration.
    /// Return result to catch configuration errors.
    fn init(&mut self,tim3: &mut TIM3) -> TimXResult<()> {
        if self.initialized {
            return Err(TimerError::AlreadyInitialized("TIM3 already initialized"));
        }
        self.config
            .validate()
            .map_err(|e| TimerError::InvalidConfiguration(e))?;
        let (psc, arr) = self.config.calculate_timer_values();
        //Step 1:  Set PSC: Prescaler (divides input clock frequency)
        tim3.psc.write(|w| w.psc().bits(psc));
        //Step 2: Set ARR: Auto-reload register (defines the period of the timer)
        tim3.arr.write(|w| w.arr().bits(arr as u16));
        // Step 3: Enable counter and update event generation
        tim3.cr1.write(|w| {
            w.cen()
                .set_bit() // Counter enabled
                .udis()
                .clear_bit() // Update event enabled
                .dir()
                .clear_bit() // Count up
                .arpe()
                .clear_bit() // Auto-reload preload disabled
        });
        //Step 4: Enable TIM3 interrupt on update event (UIE)
        tim3.dier.write(|w| w.uie().set_bit());
        self.initialized = true;
        Ok(())
    }
    /// Start the timer and enable interrupts.
    fn start(&mut self,tim3: &mut TIM3) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        tim3.cr1.modify(|_, w| w.cen().set_bit());
        Ok(())
    }

    /// Stop the timer and disable interrupts.
    fn stop(&mut self,tim3: &mut TIM3) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        tim3.cr1.modify(|_, w| w.cen().clear_bit());
        Ok(())
    }

    /// Reset the timer counter to zero
    fn reset(&mut self,tim3: &mut TIM3) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        tim3.cnt.write(|w| w.cnt().bits(0));
        Ok(())
    }

    /// Tim3 is a 16-bit timer, so the counter value is u16
    fn get_counter(&mut self,tim3: &mut TIM3) -> TimXResult<u32> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        Ok(tim3.cnt.read().cnt().bits() as u32)
    }

    /// Check if update interrupt flag (UIF) is set, indicating an update event has occurred.
    fn is_update_interrupt_flag_set(&mut self,tim3: &mut TIM3) -> TimXResult<bool> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        Ok(tim3.sr.read().uif().bit_is_set())
    }

    /// Clear the update interrupt flag (UIF) after handling the interrupt.
    fn clear_update_interrupt_flag() -> TimXResult<()> {
        let dp = unsafe { crate::pac::Peripherals::steal() };
        dp.TIM3.sr.modify(|_, w| w.uif().clear_bit());
        Ok(())
    }
    /// Set the update interrupt flag (UIF) start the timer and enable interrupts.
    fn set_update_interrupt_flag(&mut self,tim3: &mut TIM3) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        tim3.sr.modify(|_, w| w.uif().set_bit());
        Ok(())
    }
}

// TODO Re-do the unit test. 

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
    tim3: &'static TIM3,
    config: TimerConfig,
    initialized: bool,
}

impl TimerGuard for Tim3Guard {
    /// Asscociated type for the specific timer peripheral (e.g., TIM2, TIM3, etc.)
    /// This ensure the guard is bound to exactly one timer at compile time.
    type Peripheral = TIM3;

    /// Initialize the timer with clock, prescaler, and auto-reload values based on the provided configuration.
    /// Return result to catch configuration errors.
    fn init(&mut self, config: &TimerConfig) -> TimXResult<()> {
        if self.initialized {
            return Err(TimerError::AlreadyInitialized("TIM3 already initialized"));
        }
        config
            .validate()
            .map_err(|e| TimerError::InvalidConfiguration(e))?;
        let (psc, arr) = config.calculate_timer_values();
        //Step 1:  Set PSC: Prescaler (divides input clock frequency)
        self.tim3.psc.write(|w| w.psc().bits(psc));
        //Step 2: Set ARR: Auto-reload register (defines the period of the timer)
        self.tim3.arr.write(|w| w.arr().bits(arr as u16));
        // Step 3: Enable counter and update event generation
        self.tim3.cr1.write(|w| {
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
        self.tim3.dier.write(|w| w.uie().set_bit());
        self.initialized = true;
        Ok(())
    }
    /// Start the timer and enable interrupts.
    fn start(&mut self) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        self.tim3.cr1.modify(|_, w| w.cen().set_bit());
        Ok(())
    }

    /// Stop the timer and disable interrupts.
    fn stop(&mut self) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        self.tim3.cr1.modify(|_, w| w.cen().clear_bit());
        Ok(())
    }

    /// Reset the timer counter to zero
    fn reset(&mut self) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        self.tim3.cnt.write(|w| w.cnt().bits(0));
        Ok(())
    }

    /// Tim3 is a 16-bit timer, so the counter value is u16
    fn get_counter(&mut self) -> TimXResult<u32> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        Ok(self.tim3.cnt.read().cnt().bits() as u32)
    }

    /// Check if update interrupt flag (UIF) is set, indicating an update event has occurred.
    fn is_update_interrupt_flag_set(&mut self) -> TimXResult<bool> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        Ok(self.tim3.sr.read().uif().bit_is_set())
    }

    /// Clear the update interrupt flag (UIF) after handling the interrupt.
    fn clear_update_interrupt_flag(&mut self) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        self.tim3.sr.modify(|_, w| w.uif().clear_bit());
        Ok(())
    }
    /// Set the update interrupt flag (UIF) start the timer and enable interrupts.
    fn set_update_interrupt_flag(&mut self) -> TimXResult<()> {
        if !self.initialized {
            return Err(TimerError::NotInitialized);
        }
        self.tim3.sr.modify(|_, w| w.uif().set_bit());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::TimerConfig;
    use cortex_m::peripheral::ITM;

    #[derive(Debug)]
    struct MockHardWare {
        psc: u16,
        arr: u32,
        cen: bool,
        uie: bool,
        uif: bool,
        cnt: u16,
        initialized: bool,
    }

    impl MockHardWare {
        fn new() -> Self {
            MockHardWare {
                psc: 0,
                arr: 0,
                cen: false,
                uie: false,
                uif: false,
                cnt: 0,
                initialized: false,
            }
        }
    }

    impl TimerGuard for MockHardWare {
        type Peripheral = TIM3;

        fn init(&mut self, config: &TimerConfig) -> TimXResult<()> {
            if self.initialized {
                return Err(TimerError::AlreadyInitialized(
                    "MockHardWare already initialized",
                ));
            }
            config
                .validate()
                .map_err(|e| TimerError::InvalidConfiguration(e))?;
            let (psc, arr) = config.calculate_timer_values();
            self.psc = psc;
            self.arr = arr;
            self.cen = true;
            self.uie = true;
            self.initialized = true;
            Ok(())
        }

        fn start(&mut self) -> TimXResult<()> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            self.cen = true;
            Ok(())
        }

        fn stop(&mut self) -> TimXResult<()> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            self.cen = false;
            Ok(())
        }

        fn reset(&mut self) -> TimXResult<()> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            self.cnt = 0;
            Ok(())
        }

        fn get_counter(&mut self) -> TimXResult<u32> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            Ok(self.cnt as u32)
        }

        fn is_update_interrupt_flag_set(&mut self) -> TimXResult<bool> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            Ok(self.uif)
        }

        fn clear_update_interrupt_flag(&mut self) -> TimXResult<()> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            self.uif = false;
            Ok(())
        }

        fn set_update_interrupt_flag(&mut self) -> TimXResult<()> {
            if !self.initialized {
                return Err(TimerError::NotInitialized);
            }
            self.uif = true;
            Ok(())
        }
    }

    #[test]
    fn test_init_twice_returns_error() {
        let mut hw = MockHardWare::new();
        let config = TimerConfig::new(100_000, 400);
        assert!(hw.init(&config).is_ok());
        assert!(matches!(
            hw.init(&config),
            Err(TimerError::AlreadyInitialized(_))
        ));
    }
    #[test]
    fn test_start_before_init_returns_not_initialized_error() {
        let mut hw = MockHardWare::new();
        assert!(matches!(hw.start(), Err(TimerError::NotInitialized)));
    }
    #[test]
    fn test_stop_before_init_returns_not_initialized_error() {
        let mut hw = MockHardWare::new();
        assert!(matches!(hw.stop(), Err(TimerError::NotInitialized)));
    }
    #[test]
    fn test_reset_before_init_returns_not_initialized_error() {
        let mut hw = MockHardWare::new();
        assert!(matches!(hw.reset(), Err(TimerError::NotInitialized)));
    }
    #[test]
    fn test_get_counter_before_init_returns_not_initialized_error() {
        let mut hw = MockHardWare::new();
        assert!(matches!(hw.get_counter(), Err(TimerError::NotInitialized)));
    }
    #[test]
    fn test_is_update_interrupt_flag_set_before_init_returns_not_initialized_error() {
        let mut hw = MockHardWare::new();
        assert!(matches!(
            hw.is_update_interrupt_flag_set(),
            Err(TimerError::NotInitialized)
        ));
    }
    #[test]
    fn test_clear_update_interrupt_flag_before_init_returns_not_initialized_error() {
        let mut hw = MockHardWare::new();
        assert!(matches!(
            hw.clear_update_interrupt_flag(),
            Err(TimerError::NotInitialized)
        ));
    }
    #[test]
    fn test_uif_flow() {
        let mut hw = MockHardWare::new();
        let config = TimerConfig::new(100_000, 400);
        hw.init(&config).unwrap();
        assert!(hw.init(&config).is_err());
        assert!(hw.start().is_ok());
        assert_ne!(hw.is_update_interrupt_flag_set().unwrap(), true);
        assert!(hw.clear_update_interrupt_flag().is_ok());
        hw.clear_update_interrupt_flag().unwrap();
        assert_eq!(hw.is_update_interrupt_flag_set().unwrap(), false);
        assert!(hw.set_update_interrupt_flag().is_ok());
        assert_eq!(hw.is_update_interrupt_flag_set().unwrap(), true);
        assert!(hw.clear_update_interrupt_flag().is_ok());
        assert_eq!(hw.is_update_interrupt_flag_set().unwrap(), false);
        assert!(hw.stop().is_ok());
    }
}

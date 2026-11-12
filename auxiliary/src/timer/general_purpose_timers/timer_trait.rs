//! =====================================================================
//!
//! TimerGuard peripheral trait for generic timer configuration support.
//!
//! Copyright (c) 2026, Tan Dao.
//! =====================================================================

use crate::pac::{TIM2, TIM3};
use crate::timer::TimerConfig;
use core::fmt::Display;

/// Associated type for timer peripherals.
pub trait TimerPeripheral {
    fn instance() -> &'static Self;
}

/// TimerGuard peripheral trait for generic timer configuration support.
pub trait TimerGuard {
    /// Asscociated type for the specific timer peripheral (e.g., TIM2, TIM3, etc.)
    /// This ensure the guard is bound to exactly one timer at compile time.
    type Peripheral: TimerPeripheral;
    /// Initialize the timer with clock, prescaler, and auto-reload values based on the provided configuration.
    /// Return result to catch configuration errors.
    fn init(&mut self,tim3: &mut TIM3) -> TimXResult<()>;

    /// Start the timer and enable interrupts.
    fn start(&mut self,tim3: &mut TIM3) -> TimXResult<()>;

    /// Stop the timer and disable interrupts.
    fn stop(&mut self,tim3: &mut TIM3) -> TimXResult<()>;

    /// Reset the timer counter to zero
    fn reset(&mut self,tim3: &mut TIM3) -> TimXResult<()>;

    /// Get current counter value.
    fn get_counter(&mut self,tim3: &mut TIM3) -> TimXResult<u32>;

    /// Check if update interrupt flag (UIF) is set, indicating an update event has occurred.
    fn is_update_interrupt_flag_set(&mut self,tim3: &mut TIM3) -> TimXResult<bool>;

    /// Clear the update interrupt flag (UIF) after handling the interrupt.
    fn clear_update_interrupt_flag() -> TimXResult<()>;
    /// Set the update interrupt flag (UIF) start the timer and enable interrupts.
    fn set_update_interrupt_flag(&mut self,tim3: &mut TIM3) -> TimXResult<()>;
}
/// Peripheral marker implementation
impl TimerPeripheral for TIM2 {
    fn instance() -> &'static Self {
        unsafe { &*TIM2::ptr().cast::<TIM2>() }
    }
}

impl TimerPeripheral for TIM3 {
    fn instance() -> &'static Self {
        unsafe { &*TIM3::ptr().cast::<TIM3>() }
    }
}
pub type TimXResult<T> = Result<T, TimerError>;
#[derive(Debug, Clone, Copy)]
pub enum TimerError {
    AlreadyInitialized(&'static str),
    NotInitialized,
    InvalidConfiguration(&'static str),
    HardwareError,
}

impl Display for TimerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TimerError::AlreadyInitialized(msg) => write!(f, "Timer already initialized: {}", msg),
            TimerError::NotInitialized => write!(f, "Timer not initialized"),
            TimerError::InvalidConfiguration(msg) => {
                write!(f, "Invalid timer configuration: {}", msg)
            }
            TimerError::HardwareError => write!(f, "Hardware error occurred"),
        }
    }
}

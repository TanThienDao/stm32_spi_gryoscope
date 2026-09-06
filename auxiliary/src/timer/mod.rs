//! Timer module for stm32f3
//!
//! Manager TIM2 configuration and interrupt handling

pub mod tim2;

/// Timer congiguration parameter
#[derive(Debug, Clone, Copy)]
pub struct TimerConfig {
    /// System clock frequency in Hz( typically 72MHz for STM32F3).
    pub cpu_clock_hz: u32,
    /// Timer clock frequency target
    pub target_timer_clock_hz: u32,
    /// Desired interrupt frequency in Hz (e.g., 1000 for 1kHz).
    pub interrupt_enable_hz: u32,
}

impl TimerConfig {
    /// Create a new TimerConfig instance.
    ///
    /// # Arguments
    /// * `target_timer_clock_hz` - The desired timer clock frequency in Hz. (100 Khz for example)
    /// * `interrupt_enable_hz` - The desired interrupt frequency in Hz. (e.g., 400 Hz)
    /// Set up for TIM2 to generate interrupts at the specified frequency.
    pub fn new(target_timer_clock_hz: u32, interrupt_enable_hz: u32) -> Self {
        TimerConfig {
            cpu_clock_hz: 72_000_000,
            target_timer_clock_hz,
            interrupt_enable_hz,
        }
    }
    /// Create default configuration for 400 hz interrupt with 100 Khz timer clock
    pub fn default() -> Self {
        Self::new(100_000, 400)
    }

    /// Validate Configuration parameters
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.cpu_clock_hz == 0 {
            return Err("CPU clock frequency must be greater than 0");
        }
        if self.target_timer_clock_hz == 0 {
            return Err("Target timer clock frequency must be greater than 0");
        }
        if self.interrupt_enable_hz == 0 {
            return Err("Interrupt enable frequency must be greater than 0");
        }
        if self.cpu_clock_hz <= self.target_timer_clock_hz {
            return Err("CPU clock frequency must be NOT less than or equal to target timer clock frequency");
        }
        if self.interrupt_enable_hz >= self.target_timer_clock_hz {
            return Err("Interrupt enable frequency must be less than or equal to target timer clock frequency");
        }
        Ok(())

    }

    pub fn calculate_timer_values(&self) -> (u16, u32) {
        // PSC (CPU_clock / (target_timer_freq)) - 1
        let psc = (self.cpu_clock_hz / self.target_timer_clock_hz) - 1;

        // ARR (target_timer_freq / target_interrupt_freq) - 1
        let arr = (self.target_timer_clock_hz / self.interrupt_enable_hz) - 1;
        (psc as u16, arr)
    }
}

/// Result type for timer operations
pub type TimerResult<T> = Result<T, &'static str>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_config_validation() {
        let valid_config = TimerConfig::new(100_000, 400);
        assert!(valid_config.validate().is_ok());
    }
    #[test]
    fn test_timer_config_invalid() {
        let invalid_config = TimerConfig::new(0, 0);
        assert!(invalid_config.validate().is_err());
    }
    #[test]
    fn test_timer_values_calculation() {
        let config = TimerConfig::new(100_000, 400);
        let (psc, arr) = config.calculate_timer_values();
        assert_eq!(psc, 719);
        assert_eq!(arr, 249);
    }
    #[test]
    fn test_timer_values_calculation_edge_case() {
        let config = TimerConfig::new(1_000_000, 1_000);
        let (psc, arr) = config.calculate_timer_values();
        assert_eq!(psc, 71);
        assert_eq!(arr, 999);
    }
    #[test]
    fn test_timer_values_calculation_large_values() {
        let config = TimerConfig::new(10_000_000, 10_000);
        let (psc, arr) = config.calculate_timer_values();
        assert_eq!(psc, 6);
        assert_eq!(arr, 999);
    }
}
//! TimerX peripheral trait for generic timer configuration support.

use crate::pac;


/// TimerX peripheral trait for generic timer configuration support.
pub trait TimerX {
    /// Initialize the timer with the given configuration.
    fn init(&self, config: &crate::timer::TimerConfig) -> crate::timer::TimerResult<()>;

    /// Start the timer.
    fn start(&self) -> crate::timer::TimerResult<()>;

    /// Stop the timer.
    fn stop(&self) -> crate::timer::TimerResult<()>;

    /// Reset the timer.
    fn reset(&self) -> crate::timer::TimerResult<()>;
}

//! Logging module for the application with task-aware formatting.
//! All logging operations are protected by RTIC's Mutex for thread safety.

use cortex_m::peripheral::DWT;

/// Color codes for terminal output
#[derive(Debug, Clone, Copy)]
pub enum LogColor {
    /// Task-specific colors
    /// Cyan - for Init/Setup
    Cyan,
    /// CHARTREUSE - for ReadSensor (data acquisition)
    CHARTREUSE,
    /// SKY - for ProcessData (processing)
    SKY,
    /// Honey - for Errors/Anomalies
    Honey,
    /// Magenta - for Stats/Performance
    Magenta,

    /// Custom for logging levels
    /// Orange - for Debugging
    Orange,
    /// Blue - for Info
    Blue,
    /// Yellow - for Warnings
    Yellow,
    /// Red - for Error
    Red,

    /// No color (default)
    None,
}
impl LogColor {
    /// Returns the ANSI escape code for the color
    pub fn to_ansi_code(&self) -> &'static str {
        match self {
            LogColor::Cyan => "\x1b[36m",         // Cyan
            LogColor::CHARTREUSE => "\x1b[32;1m", // Bright Green (for ReadSensor)
            LogColor::SKY => "\x1b[34;1m",        // Bright Blue (for ProcessData)
            LogColor::Honey => "\x1b[33;1m",      // Bright Yellow/Orange (for errors)
            LogColor::Magenta => "\x1b[35m",      // Magenta (for stats)
            LogColor::Orange => "\x1b[38;5;208m", // Orange (for debugging - 256 color)
            LogColor::Blue => "\x1b[34m",         // Blue (for info)
            LogColor::Yellow => "\x1b[33m",       // Yellow (for warnings)
            LogColor::Red => "\x1b[31m",          // Red (for errors)
            LogColor::None => "",                 // No color
        }
    }
    /// Returns the ANSI escape code to reset the color
    pub fn reset() -> &'static str {
        "\x1b[0m" // Reset
    }
    pub fn get_from_task(task_name: &str) -> LogColor {
        match task_name {
            "Init" => LogColor::Cyan,
            "ReadSensor" => LogColor::CHARTREUSE,
            "ProcessData" => LogColor::SKY,
            "Idle" => LogColor::Honey,
            _ => LogColor::None, // Default color for unknown tasks
        }
    }
    pub fn get_from_log_level(level: &str) -> LogColor {
        match level {
            "INFO" => LogColor::Blue,
            "DEBUG" => LogColor::Orange,
            "WARN" => LogColor::Yellow,
            "ERROR" => LogColor::Red,
            "STATS" => LogColor::Magenta,
            _ => LogColor::None, // Default color for unknown levels
        }
    }
}
/// Helper function to format timestamp
pub fn format_timestamp() -> (u64, u64, u64, u64, u64) {
    let cycles = DWT::cycle_count();
    let total_seconds = cycles / 72_000_000;
    let millis = (cycles / 72_000) % 1000;
    let micros = (cycles / 72_000) % 72;

    let hours = (total_seconds / 3600) as u64;
    let minutes = ((total_seconds % 3600) / 60) as u64;
    let seconds = (total_seconds % 60) as u64;

    (hours, minutes, seconds, millis as u64, micros as u64)
}

/// Helper to print padded task name
pub fn get_padded_task(task: &str) -> &'static str {
    match task {
        "Init" => "Init        ",        // 12 chars total
        "ReadSensor" => "ReadSensor  ",  // 12 chars total
        "ProcessData" => "ProcessData ", // 12 chars total
        "Idle" => "Idle        ",        // 12 chars total
        _ => "Unknown     ",             // 12 chars total
    }
}

/// Standard info log with auto-detected color based on task
#[macro_export]
macro_rules! log_info {
    ($cx:expr,$task:expr, $($arg:tt)*) => {
        {
            let (h, m, s, ms, us) = $crate::logging::format_timestamp();
            let color = $crate::logging::LogColor::get_from_task($task);
            let padded_task = $crate::logging::get_padded_task($task);
            $cx.shared.itm.lock(|itm| {
                iprintln!(
                    &mut itm.stim[0],
                    "{}[{:02}:{:02}:{:02}.{:03}.{:03}] - [{}] [INFO ] {}{}",
                    color.to_ansi_code(), h, m, s, ms, us, padded_task,
                    format_args!($($arg)*),
                    $crate::logging::LogColor::reset()
                );
            });
        }
    };
}

/// Debug log with dim color (for verbose output)
#[macro_export]
macro_rules! log_debug {
    ($cx:expr,$task:expr, $($arg:tt)*) => {
        {
            let (h, m, s, ms, us) = $crate::logging::format_timestamp();
            let color = $crate::logging::LogColor::get_from_log_level("DEBUG");
            let padded_task = $crate::logging::get_padded_task($task);

            $cx.shared.itm.lock(|itm| {
                iprintln!(
                    &mut itm.stim[0],
                        "{}[{:02}:{:02}:{:02}.{:03}.{:03}] - [{}] [DEBUG] {}{}",
                        color.to_ansi_code(), h, m, s, ms, us, padded_task,
                        format_args!($($arg)*),
                        $crate::logging::LogColor::reset()
                    );
            });
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($cx:expr,$task:expr, $($arg:tt)*) => {
        {
            let (h, m, s, ms, us) = $crate::logging::format_timestamp();
            let color = $crate::logging::LogColor::get_from_log_level("WARN");
            let padded_task = $crate::logging::get_padded_task($task);

            $cx.shared.itm.lock(|itm| {
                iprintln!(
                    &mut itm.stim[0],
                        "{}[{:02}:{:02}:{:02}.{:03}.{:03}] - [{}] [WARN ] {}{}",
                        color.to_ansi_code(), h, m, s, ms, us, padded_task,
                        format_args!($($arg)*),
                        $crate::logging::LogColor::reset()
                    );

            });
        }
    };
}
#[macro_export]
macro_rules! log_error {
    ($cx:expr,$task:expr, $($arg:tt)*) => {
        {
            let (h, m, s, ms, us) = $crate::logging::format_timestamp();
            let color = $crate::logging::LogColor::get_from_log_level("ERROR");
            let padded_task = $crate::logging::get_padded_task($task);

            $cx.shared.itm.lock(|itm| {
                iprintln!(
                    &mut itm.stim[0],
                    "{}[{:02}:{:02}:{:02}.{:03}.{:03}] - [{}] [ERROR] {}{}",
                        color.to_ansi_code(), h, m, s, ms, us, padded_task,
                        format_args!($($arg)*),
                        $crate::logging::LogColor::reset()
                    );
            });
        }
    };
}
/// Stats log
#[macro_export]
macro_rules! log_stats {
    ($cx:expr, $task:expr, $($arg:tt)*) => {
        {
            let (h, m, s, ms, us) = $crate::logging::format_timestamp();
            let color = $crate::logging::LogColor::get_from_log_level("STATS");
            let padded_task = $crate::logging::get_padded_task($task);

            $cx.shared.itm.lock(|itm| {
                iprintln!(
                    &mut itm.stim[0],
                    "{}[{:02}:{:02}:{:02}.{:03}.{:03}] - [{}] [Stats] {}{}",
                    color.to_ansi_code(), h, m, s, ms, us, padded_task,
                    format_args!($($arg)*),
                    $crate::logging::LogColor::reset()
                );
            });
        }
    };
}
// In logging.rs
/// Init-specific log macro (for use in init task only)
#[macro_export]
macro_rules! log_init {
    ($itm:expr, $($arg:tt)*) => {
        {
            let (h, m, s, ms, us) = $crate::logging::format_timestamp();
            let color = $crate::logging::LogColor::get_from_log_level("INFO");
            let padded_task = $crate::logging::get_padded_task("Init");
            iprintln!(
                &mut $itm.stim[0],
                "{}[{:02}:{:02}:{:02}.{:03}.{:03}] - [{}] [Info ] {}{}",
                color.to_ansi_code(), h, m, s, ms, us, padded_task,
                format_args!($($arg)*),
                $crate::logging::LogColor::reset()
            );
        }
    };
}

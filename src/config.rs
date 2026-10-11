//! Application Configuration Module.
//!
//! This module defines all the configuration parameters for the application.
//! Configuration is determined at compile time using Cargo features.
//!
//! Build with different features to enable or disable specific functionalities:
//! - cargo build --release --feature production
//! - cargo build --release --feature development
//! - cargo build --release --feature "debug-logs,enable-stats"

/// Main application configuration structure.
#[derive(Debug, Clone, Copy)]
pub struct AppConfig;

impl AppConfig {
    // ========================Logging Configuration=========================
    /// Enable debug-level logging.
    ///
    /// When true, log_debug!() macros will output messages to the console.
    /// Set to false in production builds to reduce log verbosity and improve performance.
    pub const DEBUG_ENABLED: bool = cfg!(feature = "debug-logs");

    /// Enable statistics and performance metrics collection. 
    ///
    /// When true, CPU usage and performance stats are printed to the console.
    pub const STATS_ENABLED: bool = cfg!(feature = "enable-stats");

    // =========================Sensor Configuration=========================

    /// Maximum allowed change in angular velocity (in degrees per second)
    /// between consecutive readings.
    ///
    /// If the change exceeds this threshold, an anomaly is logged.
    pub const MAX_ANGULAR_VELOCITY_CHANGE: f32 = 100.0; // degrees per second   
    
    /// How often to print the info logs during normal operation.
    /// Prints every Nth iteration of the process_data. 
    /// Adjust based on your sensor's expected behavior and noise characteristics.
    pub const LOG_FREQUENCY: u32 = 4; // Print info logs every 4 iterations
    
    /// Statics collection interval.
    /// 
    /// Print performance stats every Nth iteration of the process_data.
    pub const STATS_INTERVAL: u32 = 1000; // Print stats every 1000 iterations
}

/// Helper functions to check configuration at compile time. 
impl AppConfig {
    /// Check if debugging is enabled.
    pub const fn is_debug_enabled() -> bool {
        Self::DEBUG_ENABLED
    }
    
    /// Check if statistics collection is enabled.
    pub const fn is_stats_enabled() -> bool {
        Self::STATS_ENABLED
    }
    
    /// Check if anomaly detection should be
    pub const fn anomaly_detection_threshold() -> f32 {
        Self::MAX_ANGULAR_VELOCITY_CHANGE
    }
}

// ======================== COMPLETE-TIME ASSERTIONS =========================
// These  ensure configuration makes sense 

#[cfg(all(feature = "production", feature = "debug-logs"))]
compile_error!("Cannot enable debug logs in production build. Use --feature development or --feature debug-logs instead.");

/// This constant wil be optimized away by the compiler 
/// but serves as a "poof" that config load correcly.
pub const _CONFIG_LOADED: () = ();
//! Interrupt handler module for Phase 2 - Timer-based implementation
//!
//! This module provides shared data structures for interrupt handlers.
//! The actual interrupt handler is defined in src/main.rs at the binary level.

#![allow(unsafe_code)]

use cortex_m::interrupt::Mutex;
use core::cell::RefCell;

// ===== SHARED DATA STRUCTURES =====

/// Shared sensor data (x, y, z angular velocities)
pub static SENSOR_DATA: Mutex<RefCell<(f32, f32, f32)>> =
    Mutex::new(RefCell::new((0.0, 0.0, 0.0)));

/// Flag indicating new sensor data is ready
pub static NEW_DATA_READY: Mutex<RefCell<bool>> =
    Mutex::new(RefCell::new(false));




//! Custom critical section implementation for Cortex-M
//! This provides the _critical_section_1_0_acquire and _critical_section_1_0_release
//! symbols that the critical-section crate expects.

use cortex_m::interrupt;

/// Acquire a critical section
#[no_mangle]
pub unsafe extern "C" fn _critical_section_1_0_acquire() -> u8 {
    // Disable all interrupts and return the previous state
    let primask = cortex_m::register::primask::read();
    interrupt::disable();
    primask.is_active() as u8
}

/// Release a critical section
#[no_mangle]
pub unsafe extern "C" fn _critical_section_1_0_release(was_active: u8) {
    // Restore the previous interrupt state
    if was_active != 0 {
        interrupt::enable();
    }
}


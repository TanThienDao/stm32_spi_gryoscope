//! NVIC (Nested Vectored Interrupt Controller) management for Cortex-M microcontrollers.
//! 
//! Ensure safe interrupt unmasking

pub use crate::{NVIC,interrupt};

#[derive(Debug)]
pub struct NvicGuard{
    // This struct is used to ensure that NVIC unmasking is done safely and only once.
    is_tim2_unmasked: bool,
}

impl NvicGuard {
    pub fn new() -> Self {
        NvicGuard { is_tim2_unmasked: false }
    }
    pub fn unmask_tim2_safe(&mut self) -> Result<(), &'static str> {
        if self.is_tim2_unmasked {
            return Err("TIM2 interrupt already unmasked"); // Already initialized, do nothing
        }
        unsafe {
            NVIC::unmask(interrupt::TIM2);
            //let mut nvic = Peripherals::steal().NVIC;
            //nvic.set_priority(pac::Interrupt::TIM2, 100);
        }
        self.is_tim2_unmasked = true;
        Ok(())
    }
    
    pub fn is_tim2_unmask_status(&self) -> bool {
        self.is_tim2_unmasked
    }
}
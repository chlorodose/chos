#![no_std]
extern crate alloc;
use core::arch::asm;

include!("../interface.rs");

pub struct Riscv64;
pub type Arch = Riscv64;

impl ArchImpl for Riscv64 {
    fn halt() -> ! {
        loop {
            unsafe {
                asm!("wfi", options(nomem, nostack));
            }
        }
    }
}
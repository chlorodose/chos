use core::arch::asm;

pub struct Arch;
impl super::ArchImpl for Arch {
    fn halt() -> ! {
        loop {
            unsafe {
                asm!("wfi", options(nomem, nostack));
            }
        }
    }
}
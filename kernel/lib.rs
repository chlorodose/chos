#![no_std]

use arch::Arch;

pub struct BootParms;

pub fn start_kernel(_parms: BootParms) -> ! {
    Arch::halt()
}
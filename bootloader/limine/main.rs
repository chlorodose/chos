#![no_std]
#![no_main]
extern crate alloc;

use kernel::BootParms;

#[unsafe(export_name = "_start")]
pub extern "system" fn entry() -> ! {
    let parms = todo!();
    kernel::start_kernel(parms);
}
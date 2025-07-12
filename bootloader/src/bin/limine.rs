#![no_std]
#![no_main]
extern crate kernel;

#[unsafe(export_name = "_start")]
pub extern "system" fn entry() -> ! {
    loop {}
}
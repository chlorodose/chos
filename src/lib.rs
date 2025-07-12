#![no_std]
extern crate alloc;

use arch::Arch;
use arch::ArchImpl as _;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

unsafe extern "system" {
    #[link_name = "___nonexist_symbol___"]
    safe fn unreachable_comptime() -> !;
}
struct PesudoGlobalAlloctator;
unsafe impl alloc::alloc::GlobalAlloc for PesudoGlobalAlloctator {
    unsafe fn alloc(&self, _: alloc::alloc::Layout) -> *mut u8 {
        unreachable_comptime()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: alloc::alloc::Layout) {
        unreachable_comptime()
    }
}
#[global_allocator]
static ALLOCATOR: PesudoGlobalAlloctator = PesudoGlobalAlloctator;


pub struct BootParms;

pub fn start_kernel(_parms: BootParms) -> ! {
    Arch::halt()
}
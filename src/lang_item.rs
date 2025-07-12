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

use crate::arch::Arch;
use crate::arch::ArchImpl;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panic occurred: {info}");

    Arch::halt()
}

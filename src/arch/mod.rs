pub mod page;
mod test;

use alloc::fmt::Debug;

use page::PhyPageNumber;

use crate::arch::page::{PageTableEntry, PagingMode};

pub trait ArchImpl: Debug + Clone + Copy + Default + Send + Sync {
    /// Halts the CPU indefinitely.
    fn halt() -> ! {
        #[allow(clippy::empty_loop)]
        loop {}
    }

    /// Flushes the MMU for the given address space and address.
    fn flush_mmu(addr_space: Option<usize>, addr: Option<*const ()>);

    /// Sets the MMU with the given address space, paging mode, and root paging table.
    /// Returns true if successful, false otherwise.
    /// # Safety
    /// Arguments must be valid and aligned according to the architecture's requirements.
    /// Caller must ensure proper fence is used after this call.
    unsafe fn set_mmu(addr_space: u16, mode: PagingMode, root_paging: PhyPageNumber) -> bool;

    /// Returns the maximum address space supported by the architecture.
    fn get_max_address_space() -> u16;

    /// Returns the default paging mode for the architecture.
    fn get_default_paging_mode() -> PagingMode;

    /// Returns the architecture-specific random number. Maybe low-quality.
    fn arch_rand() -> usize;

    /// Converts a page table entry to a number.
    fn pte_to_num(pte: PageTableEntry) -> usize;

    /// Converts a number to a page table entry.
    fn num_to_pte(num: usize) -> PageTableEntry;
}

mod riscv64;
cfg_if::cfg_if! {
    if #[cfg(test)] {
        pub use test::Arch;
    } else if #[cfg(target_arch = "riscv64")] {
        // mod riscv64;
        pub use riscv64::Arch;
    } else {
        pub use test::Arch;
    }
}

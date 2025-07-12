#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(
    raw_slice_split,
    ptr_metadata,
    slice_ptr_get,
    array_ptr_get,
    slice_as_array,
    ptr_as_ref_unchecked,
    allocator_api,
    exact_div,
    pointer_is_aligned_to,
    ptr_as_uninit,
    step_trait,
    new_range_api,
    array_try_from_fn,
    generic_const_exprs,
    strict_overflow_ops
)]
extern crate alloc;

#[cfg(not(test))]
mod lang_item;

pub mod arch;
pub use arch::{Arch, ArchImpl};
pub mod page;
pub use arch::page::{PhyPageNumber, VirtPageNumber};
pub use page::Page;
pub mod rng;
pub use rng::Rng;

use crate::page::PhysicalPageAccessor;

pub trait BootParms {
    /// Returns the initial random number generator.
    fn take_rng(&mut self) -> Rng;

    /// Returns a physical page accessor.
    fn take_phy_page_accessor(&mut self) -> impl PhysicalPageAccessor;

    /// Accesses the physical memory map provided by the bootloader.
    fn make_memory_map_accessor(
        &self,
    ) -> impl Iterator<Item = (PhyPageNumber, usize, MemoryMapType)> + '_;

    /// Returns the required extra memory map on paging.
    fn extra_map_iter(&self) -> impl Iterator<Item = (PhyPageNumber, VirtPageNumber, usize)> + '_;

    /// Get the kernel address for both physical and virtual.
    /// First element is for main kernel, second is for the bootloader droppable.
    fn kernel_address(&self) -> KernelAddress;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhyVirtMap {
    pub phy_base: PhyPageNumber,
    pub virt_base: VirtPageNumber,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct KernelAddress {
    pub text: PhyVirtMap,
    pub ro: PhyVirtMap,
    pub data: PhyVirtMap,
    pub bl: PhyVirtMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryMapType {
    /// Memory that is not used by the kernel.
    Unused,
    // Memory should not accessed.
    Reserved,
    /// Memory that is used by the bootloader.
    BootloaderReserved,
}

/// Starts the kernel.
pub fn start_kernel<P: BootParms>(parms: &mut P) -> ! {
    log::info!("Starting kernel...");
    let _rng = parms.take_rng();
    todo!("Kernel ended, more development needed!");
}

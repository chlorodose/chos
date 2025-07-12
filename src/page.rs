use crate::{
    Arch, ArchImpl, PhyPageNumber, VirtPageNumber,
    arch::page::{PageTable, PagingMode},
};
use core::{error::Error, fmt::Display, mem::MaybeUninit, ptr, range::Step};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalPageAllocError;
impl Display for PhysicalPageAllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Failed to allocate physical page")
    }
}
impl Error for PhysicalPageAllocError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(align(4096))]
/// Represents a single page of memory.
pub struct Page(pub [u8; 4096]);

/// Accessor for physical pages.
pub trait PhysicalPageAccessor {
    /// Accesses the physical memory at the given address.
    /// Returns a guard that allows access to the physical page.
    /// # Safety
    /// The caller must ensure that the physical page is valid and accessible.
    fn access_phy_page(&self, phy_page_number: PhyPageNumber) -> impl PhysicalPageAccessGuard + '_;
}

/// A guard for accessing a physical page.
pub trait PhysicalPageAccessGuard {
    /// Returns a mutable pointer to the page.
    fn get_mut_ptr(&self) -> *mut Page;
}

pub(crate) unsafe fn access_phy<T, R>(
    guard: &mut impl PhysicalPageAccessGuard,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    let ptr = guard.get_mut_ptr().cast::<T>();
    f(unsafe { &mut *ptr })
}

pub(crate) unsafe fn drop_phy<T>(guard: &mut impl PhysicalPageAccessGuard) {
    let ptr = guard.get_mut_ptr().cast::<T>();
    unsafe { ptr::drop_in_place(ptr) };
}

/// Physical page allocator.
pub trait PhysicalPageAllocator {
    /// Try to allocate a physical page.
    /// # Errors
    /// Returns an error if the allocation fails.
    fn allocate(&self) -> Result<PhyPageNumber, PhysicalPageAllocError> {
        self.allocate_contiguous(1)
    }
    /// Allocate count number of physical pages.
    /// Returns the physical page number of the first page.
    /// # Errors
    /// Returns an error if the allocation fails.
    fn allocate_contiguous(&self, count: usize) -> Result<PhyPageNumber, PhysicalPageAllocError>;
    /// Deallocate a physical page.
    /// # Safety
    /// The caller must ensure that the page is not in use and that it was allocated by this allocator.
    /// The page must not be accessed after deallocation.
    unsafe fn deallocate(&self, page: PhyPageNumber);
    /// Deallocate multiple physical pages.
    /// # Safety
    /// The caller must ensure that the pages are not in use and that they were allocated by this allocator.
    /// The pages must not be accessed after deallocation.
    unsafe fn deallocate_contiguous(&self, page: PhyPageNumber, count: usize) {
        for i in 0..count {
            unsafe { self.deallocate(page + i) };
        }
    }
}

#[derive(Debug)]
/// A page mapping tree.
pub struct PageTree<C: PhysicalPageAccessor, A: PhysicalPageAllocator> {
    phy_accessor: C,
    root_ppn: PhyPageNumber,
    allocator: A,
    mode: PagingMode,
}
impl<C, A> PageTree<C, A>
where
    C: PhysicalPageAccessor,
    A: PhysicalPageAllocator,
{
    /// Set this page tree as the current MMU.
    /// Returns true if succeeded.
    /// # Safety
    /// The caller must ensure that the page tree is valid and proper fence will be used.
    pub unsafe fn set_mmu(&self, addr_space: u16, mode: PagingMode) -> bool {
        unsafe { Arch::set_mmu(addr_space, mode, self.root_ppn) }
    }

    /// Create a new page tree.
    /// # Panics
    /// Panics if allocator fails to allocate the root page.
    /// # Errors
    /// Returns an error if the allocation fails.
    pub fn new(
        phy_accessor: C,
        allocator: A,
        mode: PagingMode,
    ) -> Result<Self, PhysicalPageAllocError> {
        let root_ppn = allocator.allocate()?;
        let mut root = phy_accessor.access_phy_page(root_ppn);
        unsafe {
            access_phy::<MaybeUninit<PageTable>, ()>(&mut root, |table| {
                table.write(PageTable::default());
            });
        }
        drop(root);
        Ok(PageTree {
            phy_accessor,
            root_ppn,
            allocator,
            mode,
        })
    }

    pub fn map(
        &self,
        phy_page_number: PhyPageNumber,
        virt_page_number: VirtPageNumber,
        len: usize,
    ) -> Result<(), PhysicalPageAllocError> {
        assert!(
            virt_page_number.is_valid(self.mode)
                && VirtPageNumber::forward_checked(virt_page_number, len)
                    .is_some_and(|v| v.is_valid(self.mode)),
            "Virtual page number is not valid: {virt_page_number:?}, len: {len}"
        );
        assert!(
            PhyPageNumber::forward_checked(phy_page_number, len).is_some(),
            "Physical page number is not valid: {phy_page_number:?}, len: {len}"
        );
        let mut root = self.phy_accessor.access_phy_page(self.root_ppn);
        todo!();
    }
    pub fn unmap(
        &self,
        virt_page_number: VirtPageNumber,
        len: usize,
    ) -> Result<(), PhysicalPageAllocError> {
        assert!(
            virt_page_number.is_valid(self.mode)
                && VirtPageNumber::forward_checked(virt_page_number, len)
                    .is_some_and(|v| v.is_valid(self.mode)),
            "Virtual page number is not valid: {virt_page_number:?}, len: {len}"
        );
        let mut root = self.phy_accessor.access_phy_page(self.root_ppn);
        todo!();
    }
}
impl<C, A> Drop for PageTree<C, A>
where
    C: PhysicalPageAccessor,
    A: PhysicalPageAllocator,
{
    fn drop(&mut self) {
        todo!()
    }
}
impl<C, A> Clone for PageTree<C, A>
where
    C: PhysicalPageAccessor + Clone,
    A: PhysicalPageAllocator + Clone,
{
    fn clone(&self) -> Self {
        todo!()
    }
}

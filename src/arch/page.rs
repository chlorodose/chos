use core::array;
use core::range::Step;
use core::sync::atomic::Ordering;
use core::{
    ops::{Add, Sub},
    ptr,
    sync::atomic::AtomicUsize,
};

use crate::{Arch, ArchImpl, Page};

#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable([AtomicUsize; Self::COUNT]);
impl Default for PageTable {
    fn default() -> Self {
        PageTable(array::from_fn(|_| {
            AtomicUsize::new(Arch::pte_to_num(PageTableEntry::default()))
        }))
    }
}
impl PageTable {
    pub const COUNT: usize = Page::SIZE.exact_div(size_of::<AtomicUsize>());

    pub fn iter(&self) -> impl Iterator<Item = PageTableEntry> + '_ {
        self.0
            .iter()
            .map(|entry| Arch::num_to_pte(entry.load(Ordering::Relaxed)))
    }

    #[must_use = "Always check the result to see if update fails"]
    /// Update the page table entry at the given index atomicity.
    /// # Safety
    /// The caller must ensure change this page table does not violate the architecture's requirements.
    /// # Errors
    /// Returns a Result of `Ok(previous_value)` if the function returned Some(_), else `Err(previous_value)`.
    pub unsafe fn update_at(
        &self,
        index: usize,
        mut f: impl FnMut(PageTableEntry) -> Option<PageTableEntry>,
    ) -> Result<PageTableEntry, PageTableEntry> {
        self.0[index]
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |num| {
                f(Arch::num_to_pte(num)).map(Arch::pte_to_num)
            })
            .map(Arch::num_to_pte)
            .map_err(Arch::num_to_pte)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhyPageNumber(usize);
impl Step for PhyPageNumber {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start.0 > end.0 {
            return (0, None);
        }
        let steps = end.0 - start.0;
        (steps, Some(steps))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        start.0.checked_add(count).map(PhyPageNumber)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        start.0.checked_sub(count).map(PhyPageNumber)
    }
}
impl From<usize> for PhyPageNumber {
    fn from(value: usize) -> Self {
        PhyPageNumber(value)
    }
}
impl From<PhyPageNumber> for usize {
    fn from(val: PhyPageNumber) -> Self {
        val.0
    }
}
impl Add<usize> for PhyPageNumber {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        PhyPageNumber(self.0 + rhs)
    }
}
impl Sub<usize> for PhyPageNumber {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        PhyPageNumber(self.0 - rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtPageNumber(usize);
impl Step for VirtPageNumber {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start.0 > end.0 {
            return (0, None);
        }
        let steps = end.0 - start.0;
        (steps, Some(steps))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        start.0.checked_add(count).map(VirtPageNumber)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        start.0.checked_sub(count).map(VirtPageNumber)
    }
}
impl VirtPageNumber {
    pub const MIN: Self = VirtPageNumber(0);
    pub const MAX: Self = VirtPageNumber(usize::MAX >> Page::BITS);
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Actually would not panic
    pub fn is_valid(&self, paging_mode: PagingMode) -> bool {
        if self.0 == 0 {
            return false;
        }
        if self.0 >= (1 << (usize::try_from(usize::BITS).unwrap() - paging_mode.virt_size())) - 1 {
            return false;
        }
        if [
            0,
            ((1 << (usize::try_from(usize::BITS).unwrap() - paging_mode.virt_size())) - 1),
        ]
        .contains(&(self.0 >> (paging_mode.virt_size() - 1)))
        {
            return false;
        }
        true
    }
}
impl From<usize> for VirtPageNumber {
    fn from(value: usize) -> Self {
        VirtPageNumber(value)
    }
}
impl From<VirtPageNumber> for usize {
    fn from(val: VirtPageNumber) -> Self {
        val.0
    }
}
impl From<VirtPageNumber> for *mut Page {
    fn from(value: VirtPageNumber) -> Self {
        ptr::with_exposed_provenance_mut(value.0 * Page::SIZE)
    }
}
impl Add<usize> for VirtPageNumber {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        VirtPageNumber(self.0 + rhs)
    }
}
impl Sub<usize> for VirtPageNumber {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        VirtPageNumber(self.0 - rhs)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PageTableEntry {
    Pointer(PointerPageTableEntry),
    Leaf(LeafPageTableEntry),
    Invalid(InvalidPageTableEntry),
}
impl Default for PageTableEntry {
    fn default() -> Self {
        PageTableEntry::Invalid(InvalidPageTableEntry(ptr::null_mut()))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InvalidPageTableEntry(*mut ());
impl From<InvalidPageTableEntry> for usize {
    fn from(entry: InvalidPageTableEntry) -> Self {
        entry.0.expose_provenance()
    }
}
impl From<usize> for InvalidPageTableEntry {
    fn from(value: usize) -> Self {
        InvalidPageTableEntry(ptr::with_exposed_provenance_mut(value))
    }
}
impl<P> From<*mut P> for InvalidPageTableEntry
where
    [(); align_of::<P>() - 2]: Sized,
{
    fn from(ptr: *mut P) -> Self {
        assert!(ptr.is_aligned());
        InvalidPageTableEntry(ptr.cast())
    }
}
impl<P> From<InvalidPageTableEntry> for *mut P
where
    [(); align_of::<P>() - 2]: Sized,
{
    fn from(entry: InvalidPageTableEntry) -> Self {
        debug_assert!(entry.0.is_aligned_to(2));
        entry.0.cast()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointerPageTableEntry {
    pub to: PhyPageNumber,
    pub global: bool,
    pub reserved: bool,
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct LeafPageTableEntry {
    pub to: PhyPageNumber,
    pub privilege: PagePrivilege,
    pub cache: PageCache,
    pub global: bool,
    pub user: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub reserved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Paging modes(length of virtual address space).
pub enum PagingMode {
    Layer3,
    Layer4,
    Layer5,
}
impl PagingMode {
    pub const MAX_LAYERS: usize = 6;
    #[must_use]
    pub const fn virt_size(self) -> usize {
        match self {
            PagingMode::Layer3 => 39,
            PagingMode::Layer4 => 48,
            PagingMode::Layer5 => 57,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Represents the privilege of a page.
pub enum PagePrivilege {
    #[default]
    ReadOnly,
    ExecuteOnly,
    ReadExecute,
    ReadWrite,
    ReadWriteExecute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Represents the cache type of a page.
pub enum PageCache {
    #[default]
    Cacheable,
    NonCacheable,
    IO,
}

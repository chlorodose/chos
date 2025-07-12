use crate::arch::page::{
    InvalidPageTableEntry, LeafPageTableEntry, PageTableEntry, PointerPageTableEntry,
};

use super::page::{PageCache, PagePrivilege, PagingMode, PhyPageNumber};
use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Arch;
impl super::ArchImpl for Arch {
    fn halt() -> ! {
        log::error!("Halting the CPU indefinitely.");
        loop {
            unsafe {
                asm!("wfi", options(nomem, nostack));
            }
        }
    }
    fn flush_mmu(addr_space: Option<usize>, addr: Option<*const ()>) {
        log::trace!("Flushing MMU for address space: {addr_space:?}, address: {addr:?}");
        match (addr_space, addr) {
            (Some(space), Some(address)) => unsafe {
                asm!(
                    "sfence.vma {}, {}",
                    in(reg) space,
                    in(reg) address
                );
            },
            (Some(space), None) => unsafe {
                asm!(
                    "sfence.vma {}, x0",
                    in(reg) space
                );
            },
            (None, Some(address)) => unsafe {
                asm!(
                    "sfence.vma x0, {}",
                    in(reg) address
                );
            },
            (None, None) => unsafe {
                asm!("sfence.vma x0, x0");
            },
        }
    }
    unsafe fn set_mmu(addr_space: u16, mode: PagingMode, root_table: PhyPageNumber) -> bool {
        log::trace!(
            "Setting MMU with address space: {addr_space}, mode: {mode:?}, root paging table: {root_table:?}"
        );
        let satp: usize = ((match mode {
            PagingMode::Layer3 => 8,
            PagingMode::Layer4 => 9,
            PagingMode::Layer5 => 10,
        }) << 60)
            | (usize::from(addr_space) << 44)
            | usize::from(root_table);
        let result: usize;
        // Attempt to set the SATP register and check if it was successful
        unsafe {
            asm!(r#"
                    j 1f;
                    .align 12 // Ensure instructions do not cross a page boundary
                    1:
                    csrrw {old}, satp, {x};
                    csrr {new}, satp;

                    mv {x}, x0;
                    beq {x}, {new}, 1f;
                    csrw satp, {old};
                    addi {x}, x0, 1;
                    1:
                    sfence.vma {asid}, x0;
                "#, 
                x = inout(reg) satp => result,
                asid = in(reg) addr_space,
                old = out(reg) _,
                new = out(reg) _,
                options(nostack)
            );
        };
        result == 0
    }
    fn get_max_address_space() -> u16 {
        let max_space: usize;
        unsafe {
            // Test the maximum address space by setting the SATP register
            asm!(r#"
                    j 1f;
                    .align 11 // Ensure instructions do not cross a page boundary
                    1:
                    csrrs {old}, satp, {x};
                    csrrw {x}, satp, {old};
                    sfence.vma x0, x0;
                "#,
                x = inout(reg) (usize::from(u16::MAX) << 44) => max_space,
                old = out(reg) _,
                options(nostack, nomem)
            );
        }
        ((max_space >> 44) & 0xFFFF).try_into().unwrap()
    }
    fn get_default_paging_mode() -> PagingMode {
        PagingMode::Layer3
    }
    fn arch_rand() -> usize {
        let rand: usize;
        unsafe {
            asm!(r#"
                    rdcycle {x};
                    rdtime {t};
                    xor {x}, {x}, {t};
                    rdinstret {x};
                    xor {x}, {x}, {t};
                "#, 
                x = out(reg) rand,
                t = out(reg) _,
                options(nomem, nostack)
            );
        }
        rand
    }

    fn pte_to_num(pte: PageTableEntry) -> usize {
        match pte {
            PageTableEntry::Pointer(pointer) => {
                BASE_VALID
                    | ((usize::from(pointer.to) & len_to_mask(PPN_LEN)) << PPN_OFFSET)
                    | (usize::from(pointer.reserved) << RESERVED_OFFSET)
                    | (usize::from(pointer.global) << GLOBAL_OFFSET)
            }
            PageTableEntry::Leaf(entry) => {
                BASE_VALID
                    | ((usize::from(entry.to) & len_to_mask(PPN_LEN)) << PPN_OFFSET)
                    | (privilege_to_number(entry.privilege) & len_to_mask(PRIVILEGE_LEN))
                        << PRIVILEGE_OFFSET
                    | ((cache_to_number(entry.cache) & len_to_mask(CACHE_LEN)) << CACHE_OFFSET)
                    | (usize::from(entry.reserved) << RESERVED_OFFSET)
                    | (usize::from(entry.global) << GLOBAL_OFFSET)
                    | (usize::from(entry.user) << USER_OFFSET)
                    | (usize::from(entry.accessed) << ACCESS_OFFSET)
                    | (usize::from(entry.dirty) << DIRTY_OFFSET)
            }
            PageTableEntry::Invalid(ptr) => {
                assert!((usize::from(ptr) & 1) == 0);
                ptr.into()
            }
        }
    }

    fn num_to_pte(num: usize) -> PageTableEntry {
        match (
            num & 1 == 0,
            (num >> PRIVILEGE_OFFSET) & len_to_mask(PRIVILEGE_LEN),
        ) {
            (false, _) => PageTableEntry::Invalid(InvalidPageTableEntry::from(num)),
            (true, 0) => PageTableEntry::Pointer(PointerPageTableEntry {
                to: PhyPageNumber::from((num >> PPN_OFFSET) & len_to_mask(PPN_LEN)),
                global: (num >> GLOBAL_OFFSET) & 1 != 0,
                reserved: (num >> RESERVED_OFFSET) & 1 != 0,
            }),
            (true, _) => PageTableEntry::Leaf(LeafPageTableEntry {
                to: PhyPageNumber::from((num >> PPN_OFFSET) & len_to_mask(PPN_LEN)),
                privilege: number_to_privilege(
                    (num >> PRIVILEGE_OFFSET) & len_to_mask(PRIVILEGE_LEN),
                ),
                cache: number_to_cache((num >> CACHE_OFFSET) & len_to_mask(CACHE_LEN)),
                global: (num >> GLOBAL_OFFSET) & 1 != 0,
                user: (num >> USER_OFFSET) & 1 != 0,
                accessed: (num >> ACCESS_OFFSET) & 1 != 0,
                dirty: (num >> DIRTY_OFFSET) & 1 != 0,
                reserved: (num >> RESERVED_OFFSET) & 1 != 0,
            }),
        }
    }
}

const BASE_VALID: usize = 1;

const PPN_OFFSET: usize = 10;
const PPN_LEN: usize = 50;
const RESERVED_OFFSET: usize = 8;
const GLOBAL_OFFSET: usize = 5;
const USER_OFFSET: usize = 4;
const ACCESS_OFFSET: usize = 6;
const DIRTY_OFFSET: usize = 7;
const CACHE_OFFSET: usize = 61;
const CACHE_LEN: usize = 2;
const PRIVILEGE_OFFSET: usize = 1;
const PRIVILEGE_LEN: usize = 3;

const fn privilege_to_number(p: PagePrivilege) -> usize {
    match p {
        PagePrivilege::ReadOnly => 1,
        PagePrivilege::ExecuteOnly => 4,
        PagePrivilege::ReadExecute => 5,
        PagePrivilege::ReadWrite => 3,
        PagePrivilege::ReadWriteExecute => 7,
    }
}
fn number_to_privilege(num: usize) -> PagePrivilege {
    match num {
        1 => PagePrivilege::ReadOnly,
        4 => PagePrivilege::ExecuteOnly,
        5 => PagePrivilege::ReadExecute,
        3 => PagePrivilege::ReadWrite,
        7 => PagePrivilege::ReadWriteExecute,
        _ => panic!("Invalid privilege code: {num:b}"),
    }
}
fn number_to_cache(num: usize) -> PageCache {
    match num {
        0 => PageCache::Cacheable,
        1 => PageCache::NonCacheable,
        2 => PageCache::IO,
        _ => panic!("Invalid cache code: {num:b}"),
    }
}
const fn cache_to_number(c: PageCache) -> usize {
    match c {
        PageCache::Cacheable => 0,
        PageCache::NonCacheable => 1,
        PageCache::IO => 2,
    }
}
const fn len_to_mask(len: usize) -> usize {
    (1usize << (len + 1)).overflowing_sub(1).0
}

use crate::{
    PhyPageNumber,
    arch::page::{PageTableEntry, PagingMode},
};

use super::ArchImpl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(unused)]
pub struct Arch;
impl ArchImpl for Arch {
    fn flush_mmu(_addr_space: Option<usize>, _addr: Option<*const ()>) {
        unimplemented!()
    }
    unsafe fn set_mmu(_addr_space: u16, _mode: PagingMode, _root_paging: PhyPageNumber) -> bool {
        unimplemented!()
    }
    fn get_max_address_space() -> u16 {
        u16::MAX
    }
    fn get_default_paging_mode() -> PagingMode {
        PagingMode::Layer3
    }
    fn arch_rand() -> usize {
        0
    }
    fn pte_to_num(_pte: PageTableEntry) -> usize {
        unimplemented!()
    }
    fn num_to_pte(_num: usize) -> PageTableEntry {
        unimplemented!()
    }
}

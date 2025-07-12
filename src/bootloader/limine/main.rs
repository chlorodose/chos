#![no_std]
#![no_main]
#![feature(ptr_metadata, exact_div, pointer_is_aligned_to, ptr_as_uninit)]
extern crate kernel;

use core::{
    arch::asm,
    ffi::c_void,
    fmt,
    ptr::{self, addr_of},
};
use kernel::{
    Page,
    page::{PhysicalPageAccessGuard, PhysicalPageAccessor},
};

#[allow(clippy::wildcard_imports)]
use limine::{BaseRevision, request::*};

#[unsafe(link_section = ".limine_reqs")]
#[used]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static BOOTLOADER_INFO: BootloaderInfoRequest = BootloaderInfoRequest::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static BOOT_DATE: DateAtBootRequest = DateAtBootRequest::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static KERNEL_ADDRESS: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static FIRMWARE_TYPE: FirmwareTypeRequest = FirmwareTypeRequest::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static HHDM: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static MEMORY_MAP: MemoryMapRequest = MemoryMapRequest::new();

#[unsafe(link_section = ".limine_reqs")]
#[used]
static STACK_SIZE: StackSizeRequest = StackSizeRequest::new().with_size((Page::SIZE * 512) as u64);

unsafe extern "C" {
    #[link_name = "__KERNEL_TEXT_START"]
    static KERNEL_TEXT_START: c_void;
    #[link_name = "__KERNEL_TEXT_RO_BOUNDARY"]
    static KERNEL_TEXT_RO_BOUNDARY: c_void;
    #[link_name = "__KERNEL_RO_DATA_BOUNDARY"]
    static KERNEL_RO_DATA_BOUNDARY: c_void;
    #[link_name = "__KERNEL_DATA_BL_BOUNDARY"]
    static KERNEL_DATA_BL_BOUNDARY: c_void;
    #[link_name = "__KERNEL_BL_END"]
    static KERNEL_BL_END: c_void;
}

fn to_kernel_memory_type(ty: limine::memory_map::EntryType) -> Option<kernel::MemoryMapType> {
    match ty {
        limine::memory_map::EntryType::USABLE => Some(kernel::MemoryMapType::Unused),
        limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE => {
            Some(kernel::MemoryMapType::BootloaderReserved)
        }
        limine::memory_map::EntryType::RESERVED => Some(kernel::MemoryMapType::Reserved),
        _ => None,
    }
}

struct BootParms {
    rng: Option<kernel::rng::Rng>,
    memory_map: &'static [&'static limine::memory_map::Entry],
    hhdm_offset: usize,
    kernel_vbase: usize,
    kernel_pbase: usize,
}

impl kernel::BootParms for BootParms {
    fn take_rng(&mut self) -> kernel::rng::Rng {
        self.rng.take().expect("try to take RNG more than once")
    }

    fn make_memory_map_accessor(
        &self,
    ) -> impl Iterator<Item = (kernel::PhyPageNumber, usize, kernel::MemoryMapType)> + '_ {
        self.memory_map.iter().filter_map(|entry| {
            to_kernel_memory_type(entry.entry_type).map(|ty| {
                (
                    usize::try_from(entry.base).unwrap().exact_div(4096).into(),
                    usize::try_from(entry.length).unwrap().exact_div(4096),
                    ty,
                )
            })
        })
    }

    fn extra_map_iter(
        &self,
    ) -> impl Iterator<Item = (kernel::PhyPageNumber, kernel::VirtPageNumber, usize)> + '_ {
        self.memory_map.iter().filter_map(|entry| {
            if entry.entry_type != limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE {
                return None;
            }
            Some((
                usize::try_from(entry.base).unwrap().exact_div(4096).into(),
                (self.kernel_vbase + usize::try_from(entry.base).unwrap())
                    .exact_div(4096)
                    .into(),
                usize::try_from(entry.length).unwrap().exact_div(4096),
            ))
        })
    }

    fn take_phy_page_accessor(&mut self) -> impl kernel::page::PhysicalPageAccessor {
        struct Accessor<'a> {
            rf: &'a BootParms,
        }
        impl PhysicalPageAccessor for Accessor<'_> {
            fn access_phy_page(
                &self,
                phy_page_number: kernel::PhyPageNumber,
            ) -> impl PhysicalPageAccessGuard {
                struct Guard<'a> {
                    _rf: &'a BootParms,
                    ptr: *mut Page,
                }
                impl PhysicalPageAccessGuard for Guard<'_> {
                    fn get_mut_ptr(&self) -> *mut Page {
                        self.ptr
                    }
                }
                Guard {
                    _rf: self.rf,
                    ptr: ptr::with_exposed_provenance_mut::<Page>(
                        self.rf.hhdm_offset + usize::from(phy_page_number) * Page::SIZE,
                    ),
                }
            }
        }
        Accessor { rf: self }
    }

    fn kernel_address(&self) -> kernel::KernelAddress {
        assert_eq!(self.kernel_vbase, addr_of!(KERNEL_TEXT_START).addr());
        let pbase = self.kernel_pbase.exact_div(Page::SIZE);
        let vbase = self.kernel_vbase.exact_div(Page::SIZE);

        let text_offset = 0;
        let text_length =
            (addr_of!(KERNEL_TEXT_RO_BOUNDARY).addr() - self.kernel_vbase).exact_div(Page::SIZE);
        let ro_offset = text_offset + text_length;
        let ro_length = (addr_of!(KERNEL_RO_DATA_BOUNDARY).addr()
            - addr_of!(KERNEL_TEXT_RO_BOUNDARY).addr())
        .exact_div(Page::SIZE);
        let data_offset = ro_offset + ro_length;
        let data_length = (addr_of!(KERNEL_DATA_BL_BOUNDARY).addr()
            - addr_of!(KERNEL_RO_DATA_BOUNDARY).addr())
        .exact_div(Page::SIZE);
        let bl_offset = data_offset + data_length;
        let bl_length = (addr_of!(KERNEL_BL_END).addr() - addr_of!(KERNEL_DATA_BL_BOUNDARY).addr())
            .exact_div(Page::SIZE);

        kernel::KernelAddress {
            text: kernel::PhyVirtMap {
                phy_base: (pbase + text_offset).into(),
                virt_base: (vbase + text_offset).into(),
                len: text_length,
            },
            ro: kernel::PhyVirtMap {
                phy_base: (pbase + ro_offset).into(),
                virt_base: (vbase + ro_offset).into(),
                len: ro_length,
            },
            data: kernel::PhyVirtMap {
                phy_base: (pbase + data_offset).into(),
                virt_base: (vbase + data_offset).into(),
                len: data_length,
            },
            bl: kernel::PhyVirtMap {
                phy_base: (pbase + bl_offset).into(),
                virt_base: (vbase + bl_offset).into(),
                len: bl_length,
            },
        }
    }
}

struct SBILog;
static SBILOG: SBILog = SBILog;
impl log::Log for SBILog {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use fmt::Write;
        struct Writer;
        impl Write for Writer {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                s.bytes().for_each(|b| unsafe {
                    asm!("ecall",
                    in("a0") usize::from(b),
                    in("a6") 2usize,
                    in("a7") 0x4442434Eusize);
                });
                Ok(())
            }
        }
        let _ = writeln!(
            Writer,
            "[{}] ({}) -- {}",
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}

#[unsafe(export_name = "_start")]
/// Entry point for the bootloader.
/// # Panics
/// This function will panic if it fails to receive the required bootloader information.
pub extern "system" fn entry() -> ! {
    unsafe {
        log::set_max_level_racy(log::LevelFilter::Trace);
        log::set_logger_racy(&SBILOG).expect("failed to set logger");
    };
    let hhdm = HHDM
        .get_response()
        .expect("not receiving HHDM from bootloader(limine)");
    let memory_map = MEMORY_MAP
        .get_response()
        .expect("not receiving memory map from bootloader(limine)");
    let boot_date = BOOT_DATE
        .get_response()
        .expect("not receiving boot date from bootloader(limine)");
    let kernel_address = KERNEL_ADDRESS
        .get_response()
        .expect("not receiving kernel address from bootloader(limine)");

    log::info!("Successfully collected bootloader information");

    let mut rng = kernel::rng::Rng::default();
    rng.feed(&hhdm.offset().to_ne_bytes());
    rng.feed(&boot_date.timestamp().as_micros().to_ne_bytes());

    let mut parms = BootParms {
        rng: Some(rng),
        memory_map: memory_map.entries(),
        hhdm_offset: hhdm.offset().try_into().unwrap(),
        kernel_vbase: kernel_address.virtual_base().try_into().unwrap(),
        kernel_pbase: kernel_address.physical_base().try_into().unwrap(),
    };

    kernel::start_kernel(&mut parms)
}

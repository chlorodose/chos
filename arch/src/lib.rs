#![no_std]

pub trait ArchImpl {
    fn halt() -> ! {
        loop {}
    }
}

pub struct TestArch;
impl ArchImpl for TestArch {}

pub type Arch = TestArch;
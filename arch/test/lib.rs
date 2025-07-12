#![no_std]
extern crate alloc;

include!("../interface.rs");

pub struct TestArch;
pub type Arch = TestArch;

impl ArchImpl for TestArch { }
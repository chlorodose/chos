pub trait ArchImpl {
    fn halt() -> ! {
        loop {}
    }
}
pub type DoBlocksFn = unsafe fn(adler: &mut u32, sum2: &mut u32, buf: &[u8]) -> usize;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        pub mod x86;
        pub use self::x86::accelerated_do_blocks_if_supported;
    } else {
        pub fn accelerated_do_blocks_if_supported() -> Option<DoBlocksFn> {
            None
        }
    }
}

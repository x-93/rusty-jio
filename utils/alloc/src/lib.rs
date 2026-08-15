//! Memory allocator helpers and custom global allocators for rusty-jio.

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub fn init_allocator() {
    // Custom allocator initialization if necessary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_init() {
        init_allocator();
    }
}

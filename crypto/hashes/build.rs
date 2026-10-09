fn main() {
    #[cfg(feature = "asm")]
    {
        use std::env;
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

        if target_arch == "x86_64" {
            let mut build = cc::Build::new();
            if target_os == "macos" || target_os == "ios" {
                build.file("src/keccakf1600_x86-64-osx.s");
            } else if target_os != "windows" {
                build.file("src/keccakf1600_x86-64.s");
            }
            let _ = build.try_compile("keccakf1600");
        }
    }
}
